use super::city::{City, FacilityKind};
use super::ids::{CityId, FactionId, OfficerId};
use super::officer::Officer;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SAVE_VERSION: u32 = 3;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    pub version: u32,
    pub scenario_id: String,
    pub scenario_name: String,
    pub year: i32,
    pub month: u8,
    pub turn: u32,
    pub player_faction_id: FactionId,
    pub factions: BTreeMap<FactionId, Faction>,
    pub cities: BTreeMap<CityId, City>,
    pub officers: BTreeMap<OfficerId, Officer>,
    pub roads: Vec<Road>,
    pub diplomacy: BTreeMap<String, DiplomaticRelation>,
    pub pending_commands: Vec<Command>,
    pub applied_event_ids: BTreeSet<String>,
    pub reports: Vec<TurnReport>,
    pub status: GameStatus,
}

impl GameState {
    pub fn cities_for_faction(&self, faction_id: &str) -> Vec<&City> {
        self.cities
            .values()
            .filter(|city| city.faction_id == faction_id)
            .collect()
    }

    pub fn officers_in_city(&self, city_id: &str) -> Vec<&Officer> {
        self.officers
            .values()
            .filter(|officer| officer.is_active() && officer.city_id.as_deref() == Some(city_id))
            .collect()
    }

    pub fn faction_alive(&self, faction_id: &str) -> bool {
        self.cities
            .values()
            .any(|city| city.faction_id == faction_id)
    }

    pub fn are_adjacent(&self, a: &str, b: &str) -> bool {
        self.roads
            .iter()
            .any(|road| (road.from == a && road.to == b) || (road.from == b && road.to == a))
    }

    pub fn relation(&self, a: &str, b: &str) -> Option<&DiplomaticRelation> {
        self.diplomacy.get(&diplomacy_key(a, b))
    }

    pub fn relation_mut(&mut self, a: &str, b: &str) -> &mut DiplomaticRelation {
        let key = diplomacy_key(a, b);
        self.diplomacy
            .entry(key)
            .or_insert_with(|| DiplomaticRelation::new(a.to_string(), b.to_string()))
    }

    pub fn pending_city_ids(&self) -> BTreeSet<&str> {
        self.pending_commands
            .iter()
            .map(|command| command.city_id.as_str())
            .collect()
    }

    pub fn pending_officer_ids(&self) -> BTreeSet<&str> {
        self.pending_commands
            .iter()
            .filter_map(|command| command.officer_id.as_deref())
            .collect()
    }

    pub fn advance_month(&mut self) {
        self.turn += 1;
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
    }

    pub fn refresh_status(&mut self) {
        if !self.faction_alive(&self.player_faction_id) {
            self.status = GameStatus::Defeat {
                reason: "玩家势力失去全部城池".to_string(),
            };
            return;
        }

        let player_owns_all = self
            .cities
            .values()
            .all(|city| city.faction_id == self.player_faction_id);
        if player_owns_all {
            self.status = GameStatus::Victory {
                reason: "玩家势力统一全部城池".to_string(),
            };
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameStatus {
    Running,
    Victory { reason: String },
    Defeat { reason: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub ruler_id: OfficerId,
    pub color: [f32; 3],
    pub selectable: bool,
    pub controlled_by: Controller,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Controller {
    Player,
    RuleAi,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Road {
    pub from: CityId,
    pub to: CityId,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiplomaticRelation {
    pub faction_a: FactionId,
    pub faction_b: FactionId,
    pub score: i16,
    pub truce_until_turn: Option<u32>,
}

impl DiplomaticRelation {
    pub fn new(faction_a: FactionId, faction_b: FactionId) -> Self {
        Self {
            faction_a,
            faction_b,
            score: 0,
            truce_until_turn: None,
        }
    }

    pub fn has_active_truce(&self, turn: u32) -> bool {
        self.truce_until_turn.is_some_and(|until| until >= turn)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Command {
    pub issuer_faction_id: FactionId,
    pub city_id: CityId,
    pub officer_id: Option<OfficerId>,
    pub kind: CommandKind,
}

impl Command {
    pub fn summary(&self) -> String {
        match &self.kind {
            CommandKind::Develop { focus } => format!("开发 {focus:?}"),
            CommandKind::UpgradeCityCore => "升级城镇核心".to_string(),
            CommandKind::BuildFacility { kind } => format!("建设设施 {kind:?}"),
            CommandKind::Recruit { amount } => format!("征兵 {amount}"),
            CommandKind::Train => "训练".to_string(),
            CommandKind::AppointGovernor { target_officer_id } => {
                format!("任命太守 {target_officer_id}")
            }
            CommandKind::Transfer {
                target_city_id,
                troops,
                officer_ids,
            } => format!(
                "调动到 {target_city_id}: 兵力 {troops}, 武将 {}",
                officer_ids.len()
            ),
            CommandKind::Expedition {
                target_city_id,
                troops,
            } => {
                format!("出征 {target_city_id}: {troops}")
            }
            CommandKind::Diplomacy {
                target_faction_id,
                proposal,
            } => format!("外交 {target_faction_id} {proposal:?}"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CommandKind {
    Develop {
        focus: DevelopmentFocus,
    },
    UpgradeCityCore,
    BuildFacility {
        kind: FacilityKind,
    },
    Recruit {
        amount: u32,
    },
    Train,
    AppointGovernor {
        target_officer_id: OfficerId,
    },
    Transfer {
        target_city_id: CityId,
        troops: u32,
        officer_ids: Vec<OfficerId>,
    },
    Expedition {
        target_city_id: CityId,
        troops: u32,
    },
    Diplomacy {
        target_faction_id: FactionId,
        proposal: DiplomacyProposal,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DevelopmentFocus {
    Agriculture,
    Commerce,
    Defense,
    Order,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiplomacyProposal {
    ImproveRelations,
    Truce,
    DeclareWar,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnReport {
    pub turn: u32,
    pub year: i32,
    pub month: u8,
    pub entries: Vec<ReportEntry>,
}

impl TurnReport {
    pub fn new(state: &GameState) -> Self {
        Self {
            turn: state.turn,
            year: state.year,
            month: state.month,
            entries: Vec::new(),
        }
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.entries.push(ReportEntry {
            severity: ReportSeverity::Info,
            message: message.into(),
        });
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.entries.push(ReportEntry {
            severity: ReportSeverity::Warning,
            message: message.into(),
        });
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportEntry {
    pub severity: ReportSeverity,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportSeverity {
    Info,
    Warning,
}

pub fn diplomacy_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}
