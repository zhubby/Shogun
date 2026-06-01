use crate::agent::{
    Agent, AgentProvider, AgentRunConfig, AgentRunStatus, FileAgentCheckpointStore, ToolRegistry,
    complete_task_tool,
};
use crate::game::{
    ChildGenerationContext, GameState, GeneratedOfficer, Officer, OfficerGenerationProvider,
    RuleBasedChildGenerator,
};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::tools::get_officer_info_tool;

const CHILD_NAME_AGENT_INSTRUCTIONS: &str = r#"
You name newborn children in a Three Kingdoms strategy game.

Rules:
- You must call get_officer_info for both parents before naming the child.
- Generate exactly one Chinese full name for the child.
- The name must use the father's family name.
- The full name must be 2 to 4 Chinese characters.
- Do not reuse either parent's exact name.
- Prefer historically plausible late-Han / Three Kingdoms given names.
- When done, call complete_task with result {"name":"..."}.
"#;

#[derive(Clone)]
pub struct AgentChildNameGenerator<P>
where
    P: AgentProvider + Clone + Send + Sync + 'static,
{
    provider: P,
    model: String,
    timeout: Duration,
    checkpoint_dir: Option<PathBuf>,
    fallback: RuleBasedChildGenerator,
}

impl<P> AgentChildNameGenerator<P>
where
    P: AgentProvider + Clone + Send + Sync + 'static,
{
    pub fn new(provider: P, model: impl Into<String>, timeout: Duration) -> Self {
        Self {
            provider,
            model: model.into(),
            timeout,
            checkpoint_dir: None,
            fallback: RuleBasedChildGenerator,
        }
    }

    pub fn with_checkpoint_dir(mut self, checkpoint_dir: impl Into<PathBuf>) -> Self {
        self.checkpoint_dir = Some(checkpoint_dir.into());
        self
    }
}

impl<P> OfficerGenerationProvider for AgentChildNameGenerator<P>
where
    P: AgentProvider + Clone + Send + Sync + 'static,
{
    fn generate_child(&self, context: ChildGenerationContext<'_>) -> GeneratedOfficer {
        let fallback = self.fallback.generate_child(ChildGenerationContext {
            state: context.state,
            father: context.father,
            mother: context.mother,
            sequence: context.sequence,
        });
        let Some(name) = generate_child_name_with_timeout(
            self.provider.clone(),
            ChildNameAgentRequest {
                model: self.model.clone(),
                timeout: self.timeout,
                state: context.state.clone(),
                father: context.father.clone(),
                mother: context.mother.clone(),
                sequence: context.sequence,
                fallback_name: fallback.name.clone(),
                checkpoint_dir: self.checkpoint_dir.clone(),
            },
        ) else {
            return fallback;
        };

        GeneratedOfficer { name, ..fallback }
    }
}

struct ChildNameAgentRequest {
    model: String,
    timeout: Duration,
    state: GameState,
    father: Officer,
    mother: Officer,
    sequence: u64,
    fallback_name: String,
    checkpoint_dir: Option<PathBuf>,
}

fn generate_child_name_with_timeout<P>(
    provider: P,
    request: ChildNameAgentRequest,
) -> Option<String>
where
    P: AgentProvider + Send + Sync + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let timeout = request.timeout;
    thread::spawn(move || {
        let father_name = request.father.name.clone();
        let mother_name = request.mother.name.clone();
        let name = generate_child_name(
            provider,
            request.model,
            request.state,
            request.father,
            request.mother,
            request.sequence,
            request.checkpoint_dir,
        )
        .and_then(|candidate| {
            validate_child_name(
                &candidate,
                &request.fallback_name,
                &father_name,
                &mother_name,
            )
        });
        let _ = sender.send(name);
    });
    receiver.recv_timeout(timeout).ok().flatten()
}

fn generate_child_name<P>(
    provider: P,
    model: String,
    state: GameState,
    father: Officer,
    mother: Officer,
    sequence: u64,
    checkpoint_dir: Option<PathBuf>,
) -> Option<String>
where
    P: AgentProvider,
{
    let mut tools = ToolRegistry::new();
    tools.register(get_officer_info_tool(state));
    tools.register(complete_task_tool());
    let config = AgentRunConfig {
        session_id: format!("child-name-{}-{sequence}", father.id),
        agent_type: "child_name".to_string(),
        model,
        instructions: CHILD_NAME_AGENT_INSTRUCTIONS.to_string(),
        user_goal: format!(
            "Name the newborn child of father {} ({}) and mother {} ({}).",
            father.name, father.id, mother.name, mother.id
        ),
        max_iterations: 4,
        max_output_tokens: Some(600),
        temperature: Some(0.5),
        tasks: Vec::new(),
    };
    let result = if let Some(checkpoint_dir) = checkpoint_dir {
        let agent = Agent::with_checkpoint_store(
            &provider,
            tools,
            FileAgentCheckpointStore::new(checkpoint_dir),
        );
        agent.run(config).ok()?
    } else {
        let agent = Agent::new(&provider, tools);
        agent.run(config).ok()?
    };
    if result.status != AgentRunStatus::Success {
        return None;
    }
    result
        .result
        .as_ref()
        .and_then(|result| result.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

fn validate_child_name(
    candidate: &str,
    fallback_name: &str,
    father_name: &str,
    mother_name: &str,
) -> Option<String> {
    let name: String = candidate.chars().filter(|ch| !ch.is_whitespace()).collect();
    let surname = fallback_name.chars().next()?;
    let char_count = name.chars().count();
    let valid = (2..=4).contains(&char_count)
        && name.starts_with(surname)
        && name != father_name
        && name != mother_name
        && name.chars().all(is_cjk_unified_ideograph);
    valid.then_some(name)
}

fn is_cjk_unified_ideograph(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AgentProviderRequest, AgentProviderResponse};
    use crate::game::{HistoricalCatalog, OfficerGender, SqliteHistoricalCatalog};
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct FakeProvider {
        responses: Arc<Mutex<Vec<AgentProviderResponse>>>,
    }

    impl FakeProvider {
        fn new(responses: Vec<AgentProviderResponse>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into_iter().rev().collect())),
            }
        }
    }

    impl AgentProvider for FakeProvider {
        fn respond(
            &self,
            _request: AgentProviderRequest,
        ) -> Result<AgentProviderResponse, crate::agent::AgentError> {
            self.responses
                .lock()
                .unwrap()
                .pop()
                .ok_or_else(|| crate::agent::AgentError::Provider("missing response".to_string()))
        }
    }

    fn parent_context_game() -> (GameState, Officer, Officer) {
        let mut game = SqliteHistoricalCatalog::in_memory_from_seed()
            .unwrap()
            .build_game("ad200", "liu_bei")
            .unwrap();
        let father = game.officers["liu_bei"].clone();
        let mut mother = father.clone();
        mother.id = "lady_gan".to_string();
        mother.name = "甘夫人".to_string();
        mother.gender = OfficerGender::Female;
        game.officers.insert(mother.id.clone(), mother.clone());
        (game, father, mother)
    }

    #[test]
    fn child_name_agent_uses_valid_completed_name() {
        let provider = FakeProvider::new(vec![
            AgentProviderResponse {
                output_items: vec![json!({
                    "type": "function_call",
                    "call_id": "call_father",
                    "name": "get_officer_info",
                    "arguments": {"officer_id": "liu_bei"}
                })],
                output_text: None,
            },
            AgentProviderResponse {
                output_items: vec![json!({
                    "type": "function_call",
                    "call_id": "call_mother",
                    "name": "get_officer_info",
                    "arguments": {"officer_id": "lady_gan"}
                })],
                output_text: None,
            },
            AgentProviderResponse {
                output_items: vec![json!({
                    "type": "function_call",
                    "call_id": "call_done",
                    "name": "complete_task",
                    "arguments": {
                        "status": "success",
                        "summary": "named child",
                        "result": {"name": "刘承"}
                    }
                })],
                output_text: None,
            },
        ]);
        let (game, father, mother) = parent_context_game();

        let name = generate_child_name_with_timeout(
            provider,
            ChildNameAgentRequest {
                model: "model".to_string(),
                timeout: Duration::from_secs(1),
                state: game,
                father,
                mother,
                sequence: 1,
                fallback_name: "刘安".to_string(),
                checkpoint_dir: None,
            },
        );

        assert_eq!(name.as_deref(), Some("刘承"));
    }

    #[test]
    fn child_name_validation_rejects_wrong_surname_and_non_chinese() {
        assert_eq!(
            validate_child_name("刘承", "刘安", "刘备", "甘夫人").as_deref(),
            Some("刘承")
        );
        assert_eq!(validate_child_name("关承", "刘安", "刘备", "甘夫人"), None);
        assert_eq!(
            validate_child_name("Liu Cheng", "刘安", "刘备", "甘夫人"),
            None
        );
        assert_eq!(
            validate_child_name("刘承安平远", "刘安", "刘备", "甘夫人"),
            None
        );
        assert_eq!(validate_child_name("刘备", "刘安", "刘备", "甘夫人"), None);
    }

    #[test]
    fn agent_child_name_generator_falls_back_when_agent_name_is_invalid() {
        let provider = FakeProvider::new(vec![AgentProviderResponse {
            output_items: vec![json!({
                "type": "function_call",
                "call_id": "call_done",
                "name": "complete_task",
                "arguments": {
                    "status": "success",
                    "summary": "named child",
                    "result": {"name": "关承"}
                }
            })],
            output_text: None,
        }]);
        let (game, father, mother) = parent_context_game();
        let fallback = RuleBasedChildGenerator.generate_child(ChildGenerationContext {
            state: &game,
            father: &father,
            mother: &mother,
            sequence: 1,
        });

        let generator = AgentChildNameGenerator::new(provider, "model", Duration::from_secs(1));
        let generated = generator.generate_child(ChildGenerationContext {
            state: &game,
            father: &father,
            mother: &mother,
            sequence: 1,
        });

        assert_eq!(generated, fallback);
    }
}
