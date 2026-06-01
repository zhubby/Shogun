use super::city::{City, CityFacility};
use super::ids::{CityId, FactionId, OfficerId, OfficialPostId};
use super::model::*;
use super::officer::{Officer, OfficerGender, OfficerStats, OfficerStatus, official_post_spec};
use super::technology::FactionTechnologyState;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

const DEFAULT_SCENARIO_JSON: &str =
    include_str!("../../assets/scenarios/early_three_kingdoms.json");

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ScenarioData {
    pub id: String,
    pub name: String,
    pub start_year: i32,
    pub start_month: u8,
    pub player_selectable_factions: Vec<FactionId>,
    pub factions: Vec<FactionSeed>,
    pub cities: Vec<CitySeed>,
    pub officers: Vec<OfficerSeed>,
    pub roads: Vec<Road>,
    pub diplomacy: Vec<DiplomaticRelation>,
}

impl ScenarioData {
    pub fn from_json_str(input: &str) -> Result<Self, ScenarioError> {
        serde_json::from_str(input).map_err(ScenarioError::Parse)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ScenarioError> {
        let input = fs::read_to_string(path).map_err(ScenarioError::Io)?;
        Self::from_json_str(&input)
    }

    pub fn default_scenario() -> Result<Self, ScenarioError> {
        Self::from_json_str(DEFAULT_SCENARIO_JSON)
    }

    pub fn build_game(&self, player_faction_id: &str) -> Result<GameState, ScenarioError> {
        if !self
            .player_selectable_factions
            .iter()
            .any(|id| id == player_faction_id)
        {
            return Err(ScenarioError::Invalid(format!(
                "势力 {player_faction_id} 不可选"
            )));
        }

        let mut factions = BTreeMap::new();
        for seed in &self.factions {
            factions.insert(
                seed.id.clone(),
                Faction {
                    id: seed.id.clone(),
                    name: seed.name.clone(),
                    ruler_id: seed.ruler_id.clone(),
                    color: seed.color,
                    selectable: self.player_selectable_factions.contains(&seed.id),
                    controlled_by: if seed.id == player_faction_id {
                        Controller::Player
                    } else {
                        Controller::RuleAi
                    },
                },
            );
        }

        let mut cities = BTreeMap::new();
        for seed in &self.cities {
            let mut city = City {
                id: seed.id.clone(),
                name: seed.name.clone(),
                faction_id: seed.faction_id.clone(),
                position: seed.position,
                level: seed.level,
                population: seed.population,
                gold: seed.gold,
                food: seed.food,
                materials: seed.materials,
                troops: seed
                    .troop_pool
                    .unwrap_or_else(|| TroopPool::from_total(seed.troops)),
                wounded_troops: TroopPool::default(),
                training: seed.training,
                agriculture: seed.agriculture,
                commerce: seed.commerce,
                defense: seed.defense,
                order: seed.order,
                facilities: seed.facilities.clone(),
                governor_id: seed.governor_id.clone(),
                profile: None,
            };
            city.clamp_fields();
            cities.insert(seed.id.clone(), city);
        }

        let mut officers = BTreeMap::new();
        for seed in &self.officers {
            officers.insert(
                seed.id.clone(),
                Officer {
                    id: seed.id.clone(),
                    name: seed.name.clone(),
                    faction_id: seed.faction_id.clone(),
                    city_id: seed.city_id.clone(),
                    office_id: seed.office_id.clone(),
                    stats: seed.stats,
                    loyalty: seed.loyalty,
                    gender: OfficerGender::Male,
                    status: OfficerStatus::Active,
                    profile: None,
                },
            );
        }

        for city in cities.values() {
            if !factions.contains_key(&city.faction_id) {
                return Err(ScenarioError::Invalid(format!(
                    "城池 {} 引用了不存在的势力 {}",
                    city.id, city.faction_id
                )));
            }
        }

        let mut assigned_offices = BTreeSet::new();
        for officer in officers.values() {
            if !factions.contains_key(&officer.faction_id) {
                return Err(ScenarioError::Invalid(format!(
                    "武将 {} 引用了不存在的势力 {}",
                    officer.id, officer.faction_id
                )));
            }
            if let Some(city_id) = &officer.city_id
                && !cities.contains_key(city_id)
            {
                return Err(ScenarioError::Invalid(format!(
                    "武将 {} 引用了不存在的城池 {}",
                    officer.id, city_id
                )));
            }
            if let Some(office_id) = &officer.office_id {
                if official_post_spec(office_id).is_none() {
                    return Err(ScenarioError::Invalid(format!(
                        "武将 {} 引用了不存在的官职 {}",
                        officer.id, office_id
                    )));
                }
                if !assigned_offices.insert((officer.faction_id.clone(), office_id.clone())) {
                    return Err(ScenarioError::Invalid(format!(
                        "势力 {} 重复任命官职 {}",
                        officer.faction_id, office_id
                    )));
                }
            }
        }

        let diplomacy = self
            .diplomacy
            .iter()
            .map(|relation| {
                (
                    diplomacy_key(&relation.faction_a, &relation.faction_b),
                    relation.clone(),
                )
            })
            .collect();
        let technologies = factions
            .keys()
            .map(|faction_id| (faction_id.clone(), FactionTechnologyState::default()))
            .collect();

        Ok(GameState {
            version: SAVE_VERSION,
            scenario_id: self.id.clone(),
            scenario_name: self.name.clone(),
            year: self.start_year,
            month: self.start_month,
            turn: 1,
            player_faction_id: player_faction_id.to_string(),
            factions,
            cities,
            officers,
            roads: self.roads.clone(),
            diplomacy,
            pending_commands: Vec::new(),
            army_movements: Vec::new(),
            technologies,
            events: Vec::new(),
            next_event_sequence: 0,
            applied_event_ids: BTreeSet::new(),
            reports: Vec::new(),
            status: GameStatus::Running,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FactionSeed {
    pub id: FactionId,
    pub name: String,
    pub ruler_id: OfficerId,
    pub color: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CitySeed {
    pub id: CityId,
    pub name: String,
    pub faction_id: FactionId,
    pub position: MapPosition,
    pub level: u8,
    pub population: u32,
    pub gold: i32,
    pub food: i32,
    pub materials: i32,
    pub troops: u32,
    #[serde(default)]
    pub troop_pool: Option<TroopPool>,
    pub training: u8,
    pub agriculture: u16,
    pub commerce: u16,
    pub defense: u16,
    pub order: u8,
    pub facilities: Vec<CityFacility>,
    pub governor_id: Option<OfficerId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OfficerSeed {
    pub id: OfficerId,
    pub name: String,
    pub faction_id: FactionId,
    pub city_id: Option<CityId>,
    #[serde(default)]
    pub office_id: Option<OfficialPostId>,
    pub stats: OfficerStats,
    pub loyalty: u8,
}

#[derive(Debug)]
pub enum ScenarioError {
    Io(std::io::Error),
    Parse(serde_json::Error),
    Invalid(String),
}

impl std::fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioError::Io(error) => write!(f, "读取剧本失败: {error}"),
            ScenarioError::Parse(error) => write!(f, "解析剧本失败: {error}"),
            ScenarioError::Invalid(message) => write!(f, "剧本数据无效: {message}"),
        }
    }
}

impl std::error::Error for ScenarioError {}
