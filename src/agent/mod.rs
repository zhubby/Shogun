use crate::ai::{
    AiApiError, OpenAiClient, OpenAiConfig, OpenAiResponseInput, OpenAiResponseInputItem,
    OpenAiResponseRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod child_name;
pub mod tools;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentRunConfig {
    pub session_id: String,
    pub agent_type: String,
    pub model: String,
    pub instructions: String,
    pub user_goal: String,
    pub max_iterations: u32,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tasks: Vec<AgentTask>,
}

impl AgentRunConfig {
    pub fn new(
        session_id: impl Into<String>,
        agent_type: impl Into<String>,
        model: impl Into<String>,
        instructions: impl Into<String>,
        user_goal: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            agent_type: agent_type.into(),
            model: model.into(),
            instructions: instructions.into(),
            user_goal: user_goal.into(),
            max_iterations: 4,
            max_output_tokens: Some(800),
            temperature: Some(0.4),
            tasks: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentRunResult {
    pub status: AgentRunStatus,
    pub summary: String,
    pub result: Option<Value>,
    pub iterations: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentRunStatus {
    Success,
    Partial,
    Blocked,
    MaxIterations,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTask {
    pub id: String,
    pub description: String,
    pub status: AgentTaskStatus,
    pub notes: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentTaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentCheckpoint {
    pub session_id: String,
    pub agent_type: String,
    pub input_items: Vec<Value>,
    pub iteration_count: u32,
    pub tasks: Vec<AgentTask>,
    pub custom_state: BTreeMap<String, Value>,
    pub saved_at_unix: u64,
}

pub trait AgentCheckpointStore {
    fn save(&self, checkpoint: &AgentCheckpoint) -> Result<(), AgentError>;
    fn load(&self, session_id: &str) -> Result<Option<AgentCheckpoint>, AgentError>;
    fn clear(&self, session_id: &str) -> Result<(), AgentError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoopCheckpointStore;

impl AgentCheckpointStore for NoopCheckpointStore {
    fn save(&self, _checkpoint: &AgentCheckpoint) -> Result<(), AgentError> {
        Ok(())
    }

    fn load(&self, _session_id: &str) -> Result<Option<AgentCheckpoint>, AgentError> {
        Ok(None)
    }

    fn clear(&self, _session_id: &str) -> Result<(), AgentError> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FileAgentCheckpointStore {
    base_dir: PathBuf,
}

impl FileAgentCheckpointStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    fn checkpoint_path(&self, session_id: &str) -> PathBuf {
        self.base_dir.join(format!("{session_id}.json"))
    }
}

impl AgentCheckpointStore for FileAgentCheckpointStore {
    fn save(&self, checkpoint: &AgentCheckpoint) -> Result<(), AgentError> {
        fs::create_dir_all(&self.base_dir).map_err(AgentError::CheckpointIo)?;
        let body = serde_json::to_string_pretty(checkpoint).map_err(AgentError::CheckpointJson)?;
        fs::write(self.checkpoint_path(&checkpoint.session_id), body)
            .map_err(AgentError::CheckpointIo)
    }

    fn load(&self, session_id: &str) -> Result<Option<AgentCheckpoint>, AgentError> {
        let path = self.checkpoint_path(session_id);
        if !path.exists() {
            return Ok(None);
        }
        let body = fs::read_to_string(path).map_err(AgentError::CheckpointIo)?;
        serde_json::from_str(&body)
            .map(Some)
            .map_err(AgentError::CheckpointJson)
    }

    fn clear(&self, session_id: &str) -> Result<(), AgentError> {
        let path = self.checkpoint_path(session_id);
        if path.exists() {
            fs::remove_file(path).map_err(AgentError::CheckpointIo)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct AgentProviderRequest {
    pub model: String,
    pub instructions: String,
    pub input_items: Vec<Value>,
    pub tools: Vec<Value>,
    pub max_output_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct AgentProviderResponse {
    pub output_items: Vec<Value>,
    pub output_text: Option<String>,
}

pub trait AgentProvider {
    fn respond(&self, request: AgentProviderRequest) -> Result<AgentProviderResponse, AgentError>;
}

#[derive(Clone)]
pub struct OpenAiResponsesAgentProvider {
    client: OpenAiClient,
}

impl OpenAiResponsesAgentProvider {
    pub fn new(config: OpenAiConfig) -> Result<Self, AgentError> {
        Ok(Self {
            client: OpenAiClient::new(config).map_err(AgentError::AiApi)?,
        })
    }
}

impl AgentProvider for OpenAiResponsesAgentProvider {
    fn respond(&self, request: AgentProviderRequest) -> Result<AgentProviderResponse, AgentError> {
        let mut body = OpenAiResponseRequest::new(
            request.model,
            OpenAiResponseInput::Items(
                request
                    .input_items
                    .into_iter()
                    .map(OpenAiResponseInputItem::Other)
                    .collect(),
            ),
        );
        body.instructions = Some(request.instructions);
        body.tools = Some(request.tools);
        body.max_output_tokens = request.max_output_tokens;
        body.temperature = request.temperature;
        body.parallel_tool_calls = Some(false);
        body.store = Some(false);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(AgentError::Runtime)?;
        let response = runtime
            .block_on(self.client.create_response(&body))
            .map_err(AgentError::AiApi)?;
        if let Some(error) = response.error {
            return Err(AgentError::Provider(format!(
                "OpenAI response error: {error}"
            )));
        }
        Ok(AgentProviderResponse {
            output_items: response.output,
            output_text: response.output_text,
        })
    }
}

pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn definition(&self) -> Value;
    fn execute(&self, arguments: Value) -> ToolResult;
}

pub struct FunctionAgentTool {
    name: String,
    description: String,
    parameters: Value,
    executor: Arc<dyn Fn(Value) -> ToolResult + Send + Sync>,
}

impl FunctionAgentTool {
    pub fn new<F>(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
        executor: F,
    ) -> Self
    where
        F: Fn(Value) -> ToolResult + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            executor: Arc::new(executor),
        }
    }
}

impl AgentTool for FunctionAgentTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn definition(&self) -> Value {
        json!({
            "type": "function",
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters
        })
    }

    fn execute(&self, arguments: Value) -> ToolResult {
        (self.executor)(arguments)
    }
}

#[derive(Default)]
pub struct ToolRegistry {
    tools: BTreeMap<String, Arc<dyn AgentTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T>(&mut self, tool: T)
    where
        T: AgentTool + 'static,
    {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    pub fn definitions(&self) -> Vec<Value> {
        self.tools.values().map(|tool| tool.definition()).collect()
    }

    pub fn execute(&self, name: &str, arguments: Value) -> ToolResult {
        self.tools.get(name).map_or_else(
            || ToolResult::error(format!("unknown tool: {name}")),
            |tool| tool.execute(arguments),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub success: bool,
    pub output: Value,
    pub should_continue: bool,
}

impl ToolResult {
    pub fn success(output: impl Into<Value>) -> Self {
        Self {
            success: true,
            output: output.into(),
            should_continue: true,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            output: json!({ "error": message.into() }),
            should_continue: true,
        }
    }

    pub fn complete(output: impl Into<Value>) -> Self {
        Self {
            success: true,
            output: output.into(),
            should_continue: false,
        }
    }
}

pub fn complete_task_tool() -> FunctionAgentTool {
    FunctionAgentTool::new(
        "complete_task",
        "Signal that the task is complete and return the final structured result.",
        json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["success", "partial", "blocked"]
                },
                "summary": { "type": "string" },
                "result": { "type": "object" }
            },
            "required": ["status", "summary", "result"],
            "additionalProperties": false
        }),
        |arguments| {
            let status = arguments
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("success");
            let summary = arguments
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or("completed");
            let result = arguments.get("result").cloned().unwrap_or(Value::Null);
            ToolResult::complete(json!({
                "status": status,
                "summary": summary,
                "result": result
            }))
        },
    )
}

pub struct Agent<'a, P, C = NoopCheckpointStore>
where
    P: AgentProvider,
    C: AgentCheckpointStore,
{
    provider: &'a P,
    tools: ToolRegistry,
    checkpoint_store: C,
}

impl<'a, P> Agent<'a, P, NoopCheckpointStore>
where
    P: AgentProvider,
{
    pub fn new(provider: &'a P, tools: ToolRegistry) -> Self {
        Self {
            provider,
            tools,
            checkpoint_store: NoopCheckpointStore,
        }
    }
}

impl<'a, P, C> Agent<'a, P, C>
where
    P: AgentProvider,
    C: AgentCheckpointStore,
{
    pub fn with_checkpoint_store(
        provider: &'a P,
        tools: ToolRegistry,
        checkpoint_store: C,
    ) -> Self {
        Self {
            provider,
            tools,
            checkpoint_store,
        }
    }

    pub fn run(&self, config: AgentRunConfig) -> Result<AgentRunResult, AgentError> {
        let mut input_items = self
            .checkpoint_store
            .load(&config.session_id)?
            .map(|checkpoint| checkpoint.input_items)
            .unwrap_or_else(|| vec![user_message(&config.user_goal)]);
        let mut iteration_count = 0;
        for iteration in 0..config.max_iterations {
            iteration_count = iteration + 1;
            let response = self.provider.respond(AgentProviderRequest {
                model: config.model.clone(),
                instructions: config.instructions.clone(),
                input_items: input_items.clone(),
                tools: self.tools.definitions(),
                max_output_tokens: config.max_output_tokens,
                temperature: config.temperature,
            })?;

            let function_calls = function_calls_from_output(&response.output_items)?;
            input_items.extend(response.output_items);
            if function_calls.is_empty() {
                self.save_checkpoint(&config, &input_items, iteration_count)?;
                return Err(AgentError::NoToolCall(
                    response
                        .output_text
                        .unwrap_or_else(|| "model returned no tool calls".to_string()),
                ));
            }

            for call in function_calls {
                let result = self.tools.execute(&call.name, call.arguments);
                let output_item = function_call_output_item(&call.call_id, &result)?;
                input_items.push(output_item);
                if !result.should_continue {
                    self.checkpoint_store.clear(&config.session_id)?;
                    return Ok(agent_run_result(result.output, iteration_count));
                }
            }
            self.save_checkpoint(&config, &input_items, iteration_count)?;
        }

        Ok(AgentRunResult {
            status: AgentRunStatus::MaxIterations,
            summary: "agent reached max iterations".to_string(),
            result: None,
            iterations: iteration_count,
        })
    }

    fn save_checkpoint(
        &self,
        config: &AgentRunConfig,
        input_items: &[Value],
        iteration_count: u32,
    ) -> Result<(), AgentError> {
        self.checkpoint_store.save(&AgentCheckpoint {
            session_id: config.session_id.clone(),
            agent_type: config.agent_type.clone(),
            input_items: input_items.to_vec(),
            iteration_count,
            tasks: config.tasks.clone(),
            custom_state: BTreeMap::new(),
            saved_at_unix: now_unix(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FunctionCall {
    call_id: String,
    name: String,
    arguments: Value,
}

fn user_message(text: &str) -> Value {
    json!({
        "role": "user",
        "content": text
    })
}

fn function_calls_from_output(output_items: &[Value]) -> Result<Vec<FunctionCall>, AgentError> {
    let mut calls = Vec::new();
    for item in output_items {
        if item.get("type").and_then(Value::as_str) != Some("function_call") {
            continue;
        }
        let call_id = item
            .get("call_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AgentError::InvalidResponse("function_call missing call_id".to_string())
            })?
            .to_string();
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| AgentError::InvalidResponse("function_call missing name".to_string()))?
            .to_string();
        let arguments = match item.get("arguments") {
            Some(Value::String(arguments)) => serde_json::from_str(arguments).map_err(|error| {
                AgentError::InvalidToolArguments(format!(
                    "invalid JSON arguments for {name}: {error}"
                ))
            })?,
            Some(Value::Object(_)) => item["arguments"].clone(),
            Some(Value::Null) | None => Value::Object(Default::default()),
            Some(other) => {
                return Err(AgentError::InvalidToolArguments(format!(
                    "tool arguments for {name} must be object or JSON string, got {other}"
                )));
            }
        };
        calls.push(FunctionCall {
            call_id,
            name,
            arguments,
        });
    }
    Ok(calls)
}

fn function_call_output_item(call_id: &str, result: &ToolResult) -> Result<Value, AgentError> {
    let payload = serde_json::to_string(&json!({
        "success": result.success,
        "output": result.output
    }))
    .map_err(AgentError::CheckpointJson)?;
    Ok(json!({
        "type": "function_call_output",
        "call_id": call_id,
        "output": payload
    }))
}

fn agent_run_result(output: Value, iterations: u32) -> AgentRunResult {
    let status = match output
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("success")
    {
        "partial" => AgentRunStatus::Partial,
        "blocked" => AgentRunStatus::Blocked,
        _ => AgentRunStatus::Success,
    };
    let summary = output
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("completed")
        .to_string();
    let result = output.get("result").cloned();
    AgentRunResult {
        status,
        summary,
        result,
        iterations,
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[derive(Debug)]
pub enum AgentError {
    AiApi(AiApiError),
    Provider(String),
    Runtime(std::io::Error),
    InvalidResponse(String),
    InvalidToolArguments(String),
    NoToolCall(String),
    CheckpointIo(std::io::Error),
    CheckpointJson(serde_json::Error),
}

impl fmt::Display for AgentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AiApi(error) => write!(formatter, "{error}"),
            Self::Provider(message) => write!(formatter, "{message}"),
            Self::Runtime(error) => write!(formatter, "agent runtime failed: {error}"),
            Self::InvalidResponse(message) => {
                write!(formatter, "invalid agent response: {message}")
            }
            Self::InvalidToolArguments(message) => {
                write!(formatter, "invalid tool arguments: {message}")
            }
            Self::NoToolCall(message) => write!(formatter, "agent did not call a tool: {message}"),
            Self::CheckpointIo(error) => write!(formatter, "agent checkpoint IO failed: {error}"),
            Self::CheckpointJson(error) => {
                write!(formatter, "agent checkpoint JSON failed: {error}")
            }
        }
    }
}

impl std::error::Error for AgentError {}

pub fn default_checkpoint_dir(base_dir: impl AsRef<Path>) -> PathBuf {
    base_dir.as_ref().join("agent_checkpoints")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeProvider {
        responses: Mutex<Vec<AgentProviderResponse>>,
    }

    impl FakeProvider {
        fn new(responses: Vec<AgentProviderResponse>) -> Self {
            Self {
                responses: Mutex::new(responses.into_iter().rev().collect()),
            }
        }
    }

    impl AgentProvider for FakeProvider {
        fn respond(
            &self,
            _request: AgentProviderRequest,
        ) -> Result<AgentProviderResponse, AgentError> {
            self.responses
                .lock()
                .unwrap()
                .pop()
                .ok_or_else(|| AgentError::Provider("no fake response".to_string()))
        }
    }

    #[test]
    fn agent_executes_tools_until_complete_task() {
        let provider = FakeProvider::new(vec![
            AgentProviderResponse {
                output_items: vec![json!({
                    "type": "function_call",
                    "call_id": "call_1",
                    "name": "lookup",
                    "arguments": "{\"id\":\"liu_bei\"}"
                })],
                output_text: None,
            },
            AgentProviderResponse {
                output_items: vec![json!({
                    "type": "function_call",
                    "call_id": "call_2",
                    "name": "complete_task",
                    "arguments": {
                        "status": "success",
                        "summary": "named child",
                        "result": { "name": "刘安" }
                    }
                })],
                output_text: None,
            },
        ]);
        let mut tools = ToolRegistry::new();
        tools.register(FunctionAgentTool::new(
            "lookup",
            "lookup",
            json!({"type": "object"}),
            |_arguments| ToolResult::success(json!({"name": "刘备"})),
        ));
        tools.register(complete_task_tool());

        let agent = Agent::new(&provider, tools);
        let result = agent
            .run(AgentRunConfig::new(
                "session",
                "test",
                "model",
                "instructions",
                "goal",
            ))
            .unwrap();

        assert_eq!(result.status, AgentRunStatus::Success);
        assert_eq!(result.summary, "named child");
        assert_eq!(result.result.unwrap()["name"], "刘安");
        assert_eq!(result.iterations, 2);
    }

    #[test]
    fn checkpoint_store_round_trips_without_secrets() {
        let temp_dir = tempfile::tempdir().unwrap();
        let store = FileAgentCheckpointStore::new(temp_dir.path());
        let checkpoint = AgentCheckpoint {
            session_id: "child-name-1".to_string(),
            agent_type: "child_name".to_string(),
            input_items: vec![json!({"role": "user", "content": "name child"})],
            iteration_count: 1,
            tasks: vec![AgentTask {
                id: "lookup".to_string(),
                description: "lookup parents".to_string(),
                status: AgentTaskStatus::Completed,
                notes: None,
            }],
            custom_state: BTreeMap::new(),
            saved_at_unix: 1,
        };

        store.save(&checkpoint).unwrap();
        let loaded = store.load("child-name-1").unwrap().unwrap();

        assert_eq!(loaded, checkpoint);
        let body = fs::read_to_string(temp_dir.path().join("child-name-1.json")).unwrap();
        assert!(!body.contains("api_key"));
        assert!(!body.contains("token"));
        store.clear("child-name-1").unwrap();
        assert!(store.load("child-name-1").unwrap().is_none());
    }
}
