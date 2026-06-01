use super::ids::{CityId, FactionId, OfficerId};
use super::model::GameState;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameEvent {
    pub id: String,
    pub sequence: u64,
    pub year: i32,
    pub month: u8,
    pub turn: u32,
    pub kind: GameEventKind,
    pub severity: GameEventSeverity,
    pub scope: GameEventScope,
    pub title: String,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub city_id: Option<CityId>,
    #[serde(default)]
    pub faction_id: Option<FactionId>,
    #[serde(default)]
    pub officer_id: Option<OfficerId>,
    #[serde(default)]
    pub viewed: bool,
    #[serde(default = "default_popup_pending")]
    pub popup_pending: bool,
    pub resolution: EventResolution,
}

fn default_popup_pending() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameEventKind {
    CityDevelopment,
    Battle,
    CityCaptured,
    FactionDestroyed,
    TechnologyCompleted,
    HistoricalLife,
    Famine,
    GameStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameEventSeverity {
    Info,
    Important,
    Warning,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameEventScope {
    Player,
    World,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EventResolution {
    NoneRequired,
    PendingDecision {
        deadline_turn: u32,
        default_choice_id: String,
        choices: Vec<EventChoice>,
    },
    Resolved {
        choice_id: Option<String>,
        label: String,
        turn: u32,
    },
    Expired {
        choice_id: String,
        label: String,
        turn: u32,
    },
    Cancelled {
        reason: String,
        turn: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EventChoice {
    pub id: String,
    pub label: String,
    pub description: String,
    #[serde(default)]
    pub requirements: EventChoiceRequirements,
    pub effects: EventChoiceEffects,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventChoiceRequirements {
    #[serde(default)]
    pub city_gold: i32,
    #[serde(default)]
    pub city_food: i32,
    #[serde(default)]
    pub city_materials: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventChoiceEffects {
    #[serde(default)]
    pub city_gold: i32,
    #[serde(default)]
    pub city_food: i32,
    #[serde(default)]
    pub city_materials: i32,
    #[serde(default)]
    pub city_order: i32,
    #[serde(default)]
    pub city_population: i32,
}

#[derive(Clone, Debug)]
pub struct GameEventDraft {
    pub kind: GameEventKind,
    pub severity: GameEventSeverity,
    pub scope: GameEventScope,
    pub title: String,
    pub summary: String,
    pub detail: String,
    pub city_id: Option<CityId>,
    pub faction_id: Option<FactionId>,
    pub officer_id: Option<OfficerId>,
    pub resolution: EventResolution,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventError {
    NotFound(String),
    NoDecisionRequired(String),
    InvalidChoice { event_id: String, choice_id: String },
    RequirementsNotMet(String),
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::NotFound(event_id) => write!(f, "事件 {event_id} 不存在"),
            EventError::NoDecisionRequired(event_id) => {
                write!(f, "事件 {event_id} 不需要决策")
            }
            EventError::InvalidChoice {
                event_id,
                choice_id,
            } => {
                write!(f, "事件 {event_id} 没有选项 {choice_id}")
            }
            EventError::RequirementsNotMet(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for EventError {}

pub fn record_game_event(state: &mut GameState, draft: GameEventDraft) -> String {
    state.next_event_sequence += 1;
    let sequence = state.next_event_sequence;
    let id = format!("event-{sequence}");
    state.events.push(GameEvent {
        id: id.clone(),
        sequence,
        year: state.year,
        month: state.month,
        turn: state.turn,
        kind: draft.kind,
        severity: draft.severity,
        scope: draft.scope,
        title: draft.title,
        summary: draft.summary,
        detail: draft.detail,
        city_id: draft.city_id,
        faction_id: draft.faction_id,
        officer_id: draft.officer_id,
        viewed: false,
        popup_pending: true,
        resolution: draft.resolution,
    });
    id
}

pub fn dismiss_event_popup(state: &mut GameState, event_id: &str) -> Result<(), EventError> {
    let event = event_mut(state, event_id)?;
    event.popup_pending = false;
    event.viewed = true;
    Ok(())
}

pub fn mark_event_viewed(state: &mut GameState, event_id: &str) -> Result<(), EventError> {
    event_mut(state, event_id)?.viewed = true;
    Ok(())
}

pub fn resolve_event_decision(
    state: &mut GameState,
    event_id: &str,
    choice_id: &str,
) -> Result<(), EventError> {
    let event_index = event_index(state, event_id)?;
    let choice = pending_choice(&state.events[event_index], choice_id)?;
    apply_choice_effects(state, event_index, &choice, DecisionOutcome::Resolved)
}

pub fn expire_due_event_decisions(state: &mut GameState) {
    let now_turn = state.turn;
    let due: Vec<(usize, String)> = state
        .events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| match &event.resolution {
            EventResolution::PendingDecision {
                deadline_turn,
                default_choice_id,
                ..
            } if *deadline_turn <= now_turn => Some((index, default_choice_id.clone())),
            _ => None,
        })
        .collect();

    for (event_index, default_choice_id) in due {
        let choice = match pending_choice(&state.events[event_index], &default_choice_id) {
            Ok(choice) => choice,
            Err(_) => {
                state.events[event_index].resolution = EventResolution::Cancelled {
                    reason: "默认选项不可用".to_string(),
                    turn: now_turn,
                };
                continue;
            }
        };
        let _ = apply_choice_effects(state, event_index, &choice, DecisionOutcome::Expired);
    }
}

pub fn pending_event_count(state: &GameState) -> usize {
    state
        .events
        .iter()
        .filter(|event| matches!(event.resolution, EventResolution::PendingDecision { .. }))
        .count()
}

pub fn unread_event_count(state: &GameState) -> usize {
    state.events.iter().filter(|event| !event.viewed).count()
}

pub fn popup_event_id(state: &GameState) -> Option<&str> {
    state
        .events
        .iter()
        .filter(|event| event.popup_pending)
        .min_by_key(|event| event.sequence)
        .map(|event| event.id.as_str())
}

pub fn has_pending_famine_for_city(state: &GameState, city_id: &str) -> bool {
    state.events.iter().any(|event| {
        event.kind == GameEventKind::Famine
            && event.city_id.as_deref() == Some(city_id)
            && matches!(event.resolution, EventResolution::PendingDecision { .. })
    })
}

fn event_mut<'a>(
    state: &'a mut GameState,
    event_id: &str,
) -> Result<&'a mut GameEvent, EventError> {
    state
        .events
        .iter_mut()
        .find(|event| event.id == event_id)
        .ok_or_else(|| EventError::NotFound(event_id.to_string()))
}

fn event_index(state: &GameState, event_id: &str) -> Result<usize, EventError> {
    state
        .events
        .iter()
        .position(|event| event.id == event_id)
        .ok_or_else(|| EventError::NotFound(event_id.to_string()))
}

fn pending_choice(event: &GameEvent, choice_id: &str) -> Result<EventChoice, EventError> {
    let EventResolution::PendingDecision { choices, .. } = &event.resolution else {
        return Err(EventError::NoDecisionRequired(event.id.clone()));
    };
    choices
        .iter()
        .find(|choice| choice.id == choice_id)
        .cloned()
        .ok_or_else(|| EventError::InvalidChoice {
            event_id: event.id.clone(),
            choice_id: choice_id.to_string(),
        })
}

#[derive(Clone, Copy)]
enum DecisionOutcome {
    Resolved,
    Expired,
}

fn apply_choice_effects(
    state: &mut GameState,
    event_index: usize,
    choice: &EventChoice,
    outcome: DecisionOutcome,
) -> Result<(), EventError> {
    let event_id = state.events[event_index].id.clone();
    let Some(city_id) = state.events[event_index].city_id.clone() else {
        set_cancelled(state, event_index, "事件缺少目标城池");
        return Ok(());
    };

    {
        let Some(city) = state.cities.get(&city_id) else {
            set_cancelled(state, event_index, "目标城池已不存在");
            return Ok(());
        };
        if city.faction_id != state.player_faction_id {
            set_cancelled(state, event_index, "目标城池已不属玩家");
            return Ok(());
        }
        if city.gold < choice.requirements.city_gold
            || city.food < choice.requirements.city_food
            || city.materials < choice.requirements.city_materials
        {
            return Err(EventError::RequirementsNotMet(format!(
                "{} 资源不足，无法执行 {}",
                city.name, choice.label
            )));
        }
    }

    let city = state.cities.get_mut(&city_id).ok_or_else(|| {
        EventError::RequirementsNotMet(format!("事件 {event_id} 的目标城池已不存在"))
    })?;
    city.gold += choice.effects.city_gold;
    city.food += choice.effects.city_food;
    city.materials += choice.effects.city_materials;
    city.order = apply_i32_to_u8(city.order, choice.effects.city_order, 0, 100);
    city.population = apply_i32_to_u32(city.population, choice.effects.city_population);
    city.clamp_fields();

    let turn = state.turn;
    let label = choice.label.clone();
    state.events[event_index].viewed = true;
    state.events[event_index].popup_pending = false;
    state.events[event_index].resolution = match outcome {
        DecisionOutcome::Resolved => EventResolution::Resolved {
            choice_id: Some(choice.id.clone()),
            label,
            turn,
        },
        DecisionOutcome::Expired => EventResolution::Expired {
            choice_id: choice.id.clone(),
            label,
            turn,
        },
    };
    Ok(())
}

fn set_cancelled(state: &mut GameState, event_index: usize, reason: &str) {
    state.events[event_index].viewed = true;
    state.events[event_index].popup_pending = false;
    state.events[event_index].resolution = EventResolution::Cancelled {
        reason: reason.to_string(),
        turn: state.turn,
    };
}

fn apply_i32_to_u8(value: u8, delta: i32, min: u8, max: u8) -> u8 {
    (i32::from(value) + delta).clamp(i32::from(min), i32::from(max)) as u8
}

fn apply_i32_to_u32(value: u32, delta: i32) -> u32 {
    if delta >= 0 {
        value.saturating_add(delta as u32)
    } else {
        value.saturating_sub(delta.unsigned_abs())
    }
}
