use super::city::CityEconomyEffects;
use super::ids::FactionId;
use super::model::GameState;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TechnologyId {
    MilitiaDrill,
    ArsenalLogistics,
    ScoutRoads,
    IronWeapons,
    StrictDiscipline,
    FortifiedGarrisons,
    SupplyEscort,
    CombinedArms,
    GateFireTactics,
    RotatingDefense,
    MilitaryGranaries,
    SiegeEngines,
    OfficerMerit,
    GrandCommandery,
    HouseholdRegisters,
    IrrigationSurvey,
    MarketRegisters,
    GranarySystem,
    PriceStabilization,
    ArtisanRegisters,
    CanalRestoration,
    TradePasses,
    BureaucraticRecords,
    EverNormalGranary,
    WorkshopGuilds,
    CommanderyReviews,
    CanalTaxation,
    MinistryOfFinance,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TechnologyBranch {
    Military,
    Domestic,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactionTechnologyState {
    #[serde(default)]
    pub active: Option<TechnologyId>,
    #[serde(default)]
    pub progress: BTreeMap<TechnologyId, u8>,
    #[serde(default)]
    pub funded: BTreeSet<TechnologyId>,
    #[serde(default)]
    pub completed: BTreeSet<TechnologyId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TechnologySpec {
    pub id: TechnologyId,
    pub branch: TechnologyBranch,
    pub name: &'static str,
    pub turns: u8,
    pub gold_cost: i32,
    pub prerequisites: &'static [TechnologyId],
    pub effect: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TechnologyBonuses {
    pub city_effects: CityEconomyEffects,
    pub training_command_bonus: u8,
    pub development_bonus: u16,
    pub agriculture_development_bonus: u16,
    pub commerce_development_bonus: u16,
    pub order_development_bonus: u8,
    pub recruit_gold_discount_percent: i32,
    pub travel_month_reduction: u32,
    pub attack_percent: i32,
    pub defense_percent: i32,
    pub siege_attack_percent: i32,
    pub battle_loss_reduction_percent: i32,
    pub retreat_survival_percent: i32,
    pub research_cost_reduction_percent: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletedTechnology {
    pub faction_id: FactionId,
    pub technology_id: TechnologyId,
}

const NONE: &[TechnologyId] = &[];
const IRON_WEAPONS_REQS: &[TechnologyId] = &[TechnologyId::ArsenalLogistics];
const STRICT_DISCIPLINE_REQS: &[TechnologyId] = &[TechnologyId::MilitiaDrill];
const FORTIFIED_GARRISONS_REQS: &[TechnologyId] = &[TechnologyId::MilitiaDrill];
const SUPPLY_ESCORT_REQS: &[TechnologyId] = &[TechnologyId::ScoutRoads];
const COMBINED_ARMS_REQS: &[TechnologyId] =
    &[TechnologyId::IronWeapons, TechnologyId::StrictDiscipline];
const GATE_FIRE_TACTICS_REQS: &[TechnologyId] = &[TechnologyId::IronWeapons];
const ROTATING_DEFENSE_REQS: &[TechnologyId] = &[TechnologyId::FortifiedGarrisons];
const MILITARY_GRANARIES_REQS: &[TechnologyId] = &[TechnologyId::SupplyEscort];
const SIEGE_ENGINES_REQS: &[TechnologyId] =
    &[TechnologyId::GateFireTactics, TechnologyId::CombinedArms];
const OFFICER_MERIT_REQS: &[TechnologyId] =
    &[TechnologyId::CombinedArms, TechnologyId::RotatingDefense];
const GRAND_COMMANDERY_REQS: &[TechnologyId] = &[
    TechnologyId::SiegeEngines,
    TechnologyId::MilitaryGranaries,
    TechnologyId::OfficerMerit,
];
const GRANARY_SYSTEM_REQS: &[TechnologyId] = &[TechnologyId::IrrigationSurvey];
const PRICE_STABILIZATION_REQS: &[TechnologyId] = &[TechnologyId::MarketRegisters];
const ARTISAN_REGISTERS_REQS: &[TechnologyId] = &[TechnologyId::HouseholdRegisters];
const CANAL_RESTORATION_REQS: &[TechnologyId] =
    &[TechnologyId::IrrigationSurvey, TechnologyId::GranarySystem];
const TRADE_PASSES_REQS: &[TechnologyId] = &[TechnologyId::PriceStabilization];
const BUREAUCRATIC_RECORDS_REQS: &[TechnologyId] = &[
    TechnologyId::HouseholdRegisters,
    TechnologyId::ArtisanRegisters,
];
const EVER_NORMAL_GRANARY_REQS: &[TechnologyId] = &[TechnologyId::CanalRestoration];
const WORKSHOP_GUILDS_REQS: &[TechnologyId] =
    &[TechnologyId::ArtisanRegisters, TechnologyId::TradePasses];
const COMMANDERY_REVIEWS_REQS: &[TechnologyId] = &[
    TechnologyId::BureaucraticRecords,
    TechnologyId::PriceStabilization,
];
const CANAL_TAXATION_REQS: &[TechnologyId] =
    &[TechnologyId::EverNormalGranary, TechnologyId::TradePasses];
const MINISTRY_OF_FINANCE_REQS: &[TechnologyId] = &[
    TechnologyId::CommanderyReviews,
    TechnologyId::WorkshopGuilds,
    TechnologyId::CanalTaxation,
];

pub const TECHNOLOGY_SPECS: [TechnologySpec; 28] = [
    TechnologySpec {
        id: TechnologyId::MilitiaDrill,
        branch: TechnologyBranch::Military,
        name: "乡勇操练",
        turns: 2,
        gold_cost: 90,
        prerequisites: NONE,
        effect: "训练命令额外 +2；每月兵力恢复 +25",
    },
    TechnologySpec {
        id: TechnologyId::ArsenalLogistics,
        branch: TechnologyBranch::Military,
        name: "军械整备",
        turns: 2,
        gold_cost: 110,
        prerequisites: NONE,
        effect: "征兵金钱成本 -5%",
    },
    TechnologySpec {
        id: TechnologyId::ScoutRoads,
        branch: TechnologyBranch::Military,
        name: "斥候路网",
        turns: 3,
        gold_cost: 140,
        prerequisites: NONE,
        effect: "调动/出征行军时间 -1 月，最低 1 月",
    },
    TechnologySpec {
        id: TechnologyId::IronWeapons,
        branch: TechnologyBranch::Military,
        name: "铁制兵器",
        turns: 3,
        gold_cost: 190,
        prerequisites: IRON_WEAPONS_REQS,
        effect: "进攻战斗评分 +4%",
    },
    TechnologySpec {
        id: TechnologyId::StrictDiscipline,
        branch: TechnologyBranch::Military,
        name: "严整军纪",
        turns: 3,
        gold_cost: 200,
        prerequisites: STRICT_DISCIPLINE_REQS,
        effect: "训练命令额外 +2；败退返还兵力 +5%",
    },
    TechnologySpec {
        id: TechnologyId::FortifiedGarrisons,
        branch: TechnologyBranch::Military,
        name: "坚壁戍防",
        turns: 4,
        gold_cost: 280,
        prerequisites: FORTIFIED_GARRISONS_REQS,
        effect: "防守战斗评分 +5%；每月城防 +2",
    },
    TechnologySpec {
        id: TechnologyId::SupplyEscort,
        branch: TechnologyBranch::Military,
        name: "粮道护送",
        turns: 4,
        gold_cost: 320,
        prerequisites: SUPPLY_ESCORT_REQS,
        effect: "行军到达兵力损耗 -5%；粮耗减免 +4%",
    },
    TechnologySpec {
        id: TechnologyId::CombinedArms,
        branch: TechnologyBranch::Military,
        name: "骑步协同",
        turns: 4,
        gold_cost: 380,
        prerequisites: COMBINED_ARMS_REQS,
        effect: "进攻战斗评分 +5%；训练命令额外 +1",
    },
    TechnologySpec {
        id: TechnologyId::GateFireTactics,
        branch: TechnologyBranch::Military,
        name: "城门火攻",
        turns: 5,
        gold_cost: 520,
        prerequisites: GATE_FIRE_TACTICS_REQS,
        effect: "攻城时进攻评分额外 +6%",
    },
    TechnologySpec {
        id: TechnologyId::RotatingDefense,
        branch: TechnologyBranch::Military,
        name: "守备轮戍",
        turns: 5,
        gold_cost: 540,
        prerequisites: ROTATING_DEFENSE_REQS,
        effect: "防守评分额外 +6%；城市训练每月 +1",
    },
    TechnologySpec {
        id: TechnologyId::MilitaryGranaries,
        branch: TechnologyBranch::Military,
        name: "军府仓储",
        turns: 5,
        gold_cost: 620,
        prerequisites: MILITARY_GRANARIES_REQS,
        effect: "兵力恢复 +60；粮耗减免额外 +6%",
    },
    TechnologySpec {
        id: TechnologyId::SiegeEngines,
        branch: TechnologyBranch::Military,
        name: "攻城器械",
        turns: 6,
        gold_cost: 860,
        prerequisites: SIEGE_ENGINES_REQS,
        effect: "攻城进攻评分额外 +10%",
    },
    TechnologySpec {
        id: TechnologyId::OfficerMerit,
        branch: TechnologyBranch::Military,
        name: "将校考课",
        turns: 6,
        gold_cost: 920,
        prerequisites: OFFICER_MERIT_REQS,
        effect: "攻防评分各 +4%；训练命令额外 +2",
    },
    TechnologySpec {
        id: TechnologyId::GrandCommandery,
        branch: TechnologyBranch::Military,
        name: "都督府制",
        turns: 8,
        gold_cost: 1450,
        prerequisites: GRAND_COMMANDERY_REQS,
        effect: "攻防评分各 +6%；强化后期军队质量",
    },
    TechnologySpec {
        id: TechnologyId::HouseholdRegisters,
        branch: TechnologyBranch::Domestic,
        name: "户籍清丈",
        turns: 2,
        gold_cost: 90,
        prerequisites: NONE,
        effect: "每月人口增长 +10；治安 +1",
    },
    TechnologySpec {
        id: TechnologyId::IrrigationSurvey,
        branch: TechnologyBranch::Domestic,
        name: "水利勘测",
        turns: 2,
        gold_cost: 100,
        prerequisites: NONE,
        effect: "粮收入 +10；农业开发额外 +2",
    },
    TechnologySpec {
        id: TechnologyId::MarketRegisters,
        branch: TechnologyBranch::Domestic,
        name: "市籍整理",
        turns: 2,
        gold_cost: 110,
        prerequisites: NONE,
        effect: "金收入 +8；商业开发额外 +2",
    },
    TechnologySpec {
        id: TechnologyId::GranarySystem,
        branch: TechnologyBranch::Domestic,
        name: "仓廪制度",
        turns: 3,
        gold_cost: 180,
        prerequisites: GRANARY_SYSTEM_REQS,
        effect: "粮收入 +8；粮耗减免 +5%",
    },
    TechnologySpec {
        id: TechnologyId::PriceStabilization,
        branch: TechnologyBranch::Domestic,
        name: "平准市易",
        turns: 3,
        gold_cost: 200,
        prerequisites: PRICE_STABILIZATION_REQS,
        effect: "金收入额外 +6；金收入 +3%",
    },
    TechnologySpec {
        id: TechnologyId::ArtisanRegisters,
        branch: TechnologyBranch::Domestic,
        name: "工匠名籍",
        turns: 3,
        gold_cost: 210,
        prerequisites: ARTISAN_REGISTERS_REQS,
        effect: "建材收入 +4；建材收入 +2%",
    },
    TechnologySpec {
        id: TechnologyId::CanalRestoration,
        branch: TechnologyBranch::Domestic,
        name: "灌渠修复",
        turns: 4,
        gold_cost: 320,
        prerequisites: CANAL_RESTORATION_REQS,
        effect: "粮收入 +4%；人口增长额外 +8",
    },
    TechnologySpec {
        id: TechnologyId::TradePasses,
        branch: TechnologyBranch::Domestic,
        name: "商旅关津",
        turns: 4,
        gold_cost: 350,
        prerequisites: TRADE_PASSES_REQS,
        effect: "金收入额外 +4%；治安 -1",
    },
    TechnologySpec {
        id: TechnologyId::BureaucraticRecords,
        branch: TechnologyBranch::Domestic,
        name: "官署文书",
        turns: 4,
        gold_cost: 380,
        prerequisites: BUREAUCRATIC_RECORDS_REQS,
        effect: "维护减免 +5%；治安 +1",
    },
    TechnologySpec {
        id: TechnologyId::EverNormalGranary,
        branch: TechnologyBranch::Domestic,
        name: "常平仓法",
        turns: 5,
        gold_cost: 560,
        prerequisites: EVER_NORMAL_GRANARY_REQS,
        effect: "粮收入 +12；粮耗减免额外 +8%；治安 +1",
    },
    TechnologySpec {
        id: TechnologyId::WorkshopGuilds,
        branch: TechnologyBranch::Domestic,
        name: "工坊行会",
        turns: 5,
        gold_cost: 600,
        prerequisites: WORKSHOP_GUILDS_REQS,
        effect: "建材收入额外 +6；建材收入 +4%",
    },
    TechnologySpec {
        id: TechnologyId::CommanderyReviews,
        branch: TechnologyBranch::Domestic,
        name: "郡县考课",
        turns: 6,
        gold_cost: 820,
        prerequisites: COMMANDERY_REVIEWS_REQS,
        effect: "所有开发命令额外 +3；治安开发额外 +1",
    },
    TechnologySpec {
        id: TechnologyId::CanalTaxation,
        branch: TechnologyBranch::Domestic,
        name: "漕运税制",
        turns: 6,
        gold_cost: 960,
        prerequisites: CANAL_TAXATION_REQS,
        effect: "金 +5%、粮 +5%；维护减免额外 +5%",
    },
    TechnologySpec {
        id: TechnologyId::MinistryOfFinance,
        branch: TechnologyBranch::Domestic,
        name: "度支尚书",
        turns: 8,
        gold_cost: 1500,
        prerequisites: MINISTRY_OF_FINANCE_REQS,
        effect: "金/粮/建材各 +4%；后续太学研习立项成本 -10%",
    },
];

const AI_RESEARCH_PRIORITY: [TechnologyId; 28] = [
    TechnologyId::HouseholdRegisters,
    TechnologyId::IrrigationSurvey,
    TechnologyId::MarketRegisters,
    TechnologyId::MilitiaDrill,
    TechnologyId::ArsenalLogistics,
    TechnologyId::ScoutRoads,
    TechnologyId::GranarySystem,
    TechnologyId::PriceStabilization,
    TechnologyId::ArtisanRegisters,
    TechnologyId::IronWeapons,
    TechnologyId::StrictDiscipline,
    TechnologyId::FortifiedGarrisons,
    TechnologyId::CanalRestoration,
    TechnologyId::TradePasses,
    TechnologyId::BureaucraticRecords,
    TechnologyId::SupplyEscort,
    TechnologyId::CombinedArms,
    TechnologyId::GateFireTactics,
    TechnologyId::EverNormalGranary,
    TechnologyId::WorkshopGuilds,
    TechnologyId::RotatingDefense,
    TechnologyId::MilitaryGranaries,
    TechnologyId::CommanderyReviews,
    TechnologyId::CanalTaxation,
    TechnologyId::SiegeEngines,
    TechnologyId::OfficerMerit,
    TechnologyId::MinistryOfFinance,
    TechnologyId::GrandCommandery,
];

pub fn technology_specs() -> &'static [TechnologySpec] {
    &TECHNOLOGY_SPECS
}

pub fn technology_specs_for_branch(
    branch: TechnologyBranch,
) -> impl Iterator<Item = &'static TechnologySpec> {
    TECHNOLOGY_SPECS
        .iter()
        .filter(move |spec| spec.branch == branch)
}

pub fn technology_spec(id: TechnologyId) -> &'static TechnologySpec {
    TECHNOLOGY_SPECS
        .iter()
        .find(|spec| spec.id == id)
        .expect("all technology ids must have specs")
}

pub fn ensure_faction_technology_states(state: &mut GameState) {
    for faction_id in state.factions.keys() {
        state.technologies.entry(faction_id.clone()).or_default();
    }
}

pub fn faction_technology_state<'a>(
    state: &'a GameState,
    faction_id: &str,
) -> Option<&'a FactionTechnologyState> {
    state.technologies.get(faction_id)
}

pub fn faction_technology_state_mut<'a>(
    state: &'a mut GameState,
    faction_id: &str,
) -> &'a mut FactionTechnologyState {
    state
        .technologies
        .entry(faction_id.to_string())
        .or_default()
}

pub fn faction_total_gold(state: &GameState, faction_id: &str) -> i32 {
    state
        .cities
        .values()
        .filter(|city| city.faction_id == faction_id)
        .map(|city| city.gold.max(0))
        .sum()
}

pub fn effective_technology_cost(state: &GameState, faction_id: &str, id: TechnologyId) -> i32 {
    let spec = technology_spec(id);
    let reduction = faction_technology_bonuses(state, faction_id)
        .research_cost_reduction_percent
        .clamp(0, 80);
    spec.gold_cost * (100 - reduction) / 100
}

pub fn can_research(
    state: &GameState,
    faction_id: &str,
    id: TechnologyId,
) -> Result<(), TechnologyError> {
    let spec = technology_spec(id);
    let faction_state = state.technologies.get(faction_id);
    if faction_state.is_some_and(|state| state.completed.contains(&id)) {
        return Err(TechnologyError::Completed(spec.name.to_string()));
    }
    if let Some(missing) = spec.prerequisites.iter().find(|prerequisite| {
        !faction_state.is_some_and(|state| state.completed.contains(prerequisite))
    }) {
        return Err(TechnologyError::MissingPrerequisite(
            technology_spec(*missing).name.to_string(),
        ));
    }
    Ok(())
}

pub fn start_research(
    state: &mut GameState,
    faction_id: &str,
    id: TechnologyId,
) -> Result<ResearchStartOutcome, TechnologyError> {
    can_research(state, faction_id, id)?;
    let already_funded = state
        .technologies
        .get(faction_id)
        .is_some_and(|faction_state| faction_state.funded.contains(&id));

    if !already_funded {
        let cost = effective_technology_cost(state, faction_id, id);
        let available = faction_total_gold(state, faction_id);
        if available < cost {
            return Err(TechnologyError::InsufficientGold {
                required: cost,
                available,
            });
        }
        deduct_faction_gold(state, faction_id, cost);
        let faction_state = faction_technology_state_mut(state, faction_id);
        faction_state.funded.insert(id);
        faction_state.progress.entry(id).or_insert(0);
        faction_state.active = Some(id);
        return Ok(ResearchStartOutcome {
            cost_paid: cost,
            resumed: false,
        });
    }

    faction_technology_state_mut(state, faction_id).active = Some(id);
    Ok(ResearchStartOutcome {
        cost_paid: 0,
        resumed: true,
    })
}

pub fn advance_active_research(state: &mut GameState) -> Vec<CompletedTechnology> {
    let faction_ids: Vec<FactionId> = state
        .factions
        .keys()
        .filter(|faction_id| state.faction_alive(faction_id))
        .cloned()
        .collect();
    let mut completed = Vec::new();

    for faction_id in faction_ids {
        let Some(active_id) = state
            .technologies
            .get(&faction_id)
            .and_then(|faction_state| faction_state.active)
        else {
            continue;
        };
        let spec = technology_spec(active_id);
        let faction_state = faction_technology_state_mut(state, &faction_id);
        if !faction_state.funded.contains(&active_id)
            || faction_state.completed.contains(&active_id)
        {
            continue;
        }
        let progress = faction_state.progress.entry(active_id).or_insert(0);
        *progress = progress.saturating_add(1).min(spec.turns);
        if *progress >= spec.turns {
            faction_state.completed.insert(active_id);
            faction_state.active = None;
            completed.push(CompletedTechnology {
                faction_id: faction_id.clone(),
                technology_id: active_id,
            });
        }
    }

    completed
}

pub fn choose_ai_research(state: &GameState, faction_id: &str) -> Option<TechnologyId> {
    let faction_state = state.technologies.get(faction_id);
    if faction_state.is_some_and(|state| state.active.is_some()) {
        return None;
    }
    AI_RESEARCH_PRIORITY.into_iter().find(|technology_id| {
        can_research(state, faction_id, *technology_id).is_ok()
            && (faction_state.is_some_and(|state| state.funded.contains(technology_id))
                || faction_total_gold(state, faction_id)
                    >= effective_technology_cost(state, faction_id, *technology_id))
    })
}

pub fn faction_technology_bonuses(state: &GameState, faction_id: &str) -> TechnologyBonuses {
    let Some(faction_state) = state.technologies.get(faction_id) else {
        return TechnologyBonuses::default();
    };
    let mut bonuses = TechnologyBonuses::default();
    for technology_id in &faction_state.completed {
        apply_completed_technology_bonus(&mut bonuses, *technology_id);
    }
    bonuses
}

pub fn faction_technology_city_effects(state: &GameState, faction_id: &str) -> CityEconomyEffects {
    faction_technology_bonuses(state, faction_id).city_effects
}

pub fn technology_progress(state: &FactionTechnologyState, id: TechnologyId) -> u8 {
    state.progress.get(&id).copied().unwrap_or_default()
}

pub fn missing_prerequisite_names(
    state: Option<&FactionTechnologyState>,
    id: TechnologyId,
) -> Vec<&'static str> {
    technology_spec(id)
        .prerequisites
        .iter()
        .filter(|prerequisite| !state.is_some_and(|state| state.completed.contains(prerequisite)))
        .map(|prerequisite| technology_spec(*prerequisite).name)
        .collect()
}

fn deduct_faction_gold(state: &mut GameState, faction_id: &str, mut amount: i32) {
    let mut city_ids: Vec<_> = state
        .cities
        .values()
        .filter(|city| city.faction_id == faction_id && city.gold > 0)
        .map(|city| (city.id.clone(), city.gold))
        .collect();
    city_ids.sort_by(|(left_id, left_gold), (right_id, right_gold)| {
        right_gold
            .cmp(left_gold)
            .then_with(|| left_id.cmp(right_id))
    });

    for (city_id, _) in city_ids {
        if amount <= 0 {
            break;
        }
        let Some(city) = state.cities.get_mut(&city_id) else {
            continue;
        };
        let paid = city.gold.min(amount);
        city.gold -= paid;
        amount -= paid;
    }
}

fn apply_completed_technology_bonus(bonuses: &mut TechnologyBonuses, id: TechnologyId) {
    match id {
        TechnologyId::MilitiaDrill => {
            bonuses.training_command_bonus += 2;
            bonuses.city_effects.troop_recovery += 25;
        }
        TechnologyId::ArsenalLogistics => bonuses.recruit_gold_discount_percent += 5,
        TechnologyId::ScoutRoads => bonuses.travel_month_reduction += 1,
        TechnologyId::IronWeapons => bonuses.attack_percent += 4,
        TechnologyId::StrictDiscipline => {
            bonuses.training_command_bonus += 2;
            bonuses.retreat_survival_percent += 5;
        }
        TechnologyId::FortifiedGarrisons => {
            bonuses.defense_percent += 5;
            bonuses.city_effects.defense += 2;
        }
        TechnologyId::SupplyEscort => {
            bonuses.battle_loss_reduction_percent += 5;
            bonuses.city_effects.food_upkeep_reduction_percent += 4;
        }
        TechnologyId::CombinedArms => {
            bonuses.attack_percent += 5;
            bonuses.training_command_bonus += 1;
        }
        TechnologyId::GateFireTactics => bonuses.siege_attack_percent += 6,
        TechnologyId::RotatingDefense => {
            bonuses.defense_percent += 6;
            bonuses.city_effects.training += 1;
        }
        TechnologyId::MilitaryGranaries => {
            bonuses.city_effects.troop_recovery += 60;
            bonuses.city_effects.food_upkeep_reduction_percent += 6;
        }
        TechnologyId::SiegeEngines => bonuses.siege_attack_percent += 10,
        TechnologyId::OfficerMerit => {
            bonuses.attack_percent += 4;
            bonuses.defense_percent += 4;
            bonuses.training_command_bonus += 2;
        }
        TechnologyId::GrandCommandery => {
            bonuses.attack_percent += 6;
            bonuses.defense_percent += 6;
        }
        TechnologyId::HouseholdRegisters => {
            bonuses.city_effects.population_growth += 10;
            bonuses.city_effects.order += 1;
        }
        TechnologyId::IrrigationSurvey => {
            bonuses.city_effects.food_income += 10;
            bonuses.agriculture_development_bonus += 2;
        }
        TechnologyId::MarketRegisters => {
            bonuses.city_effects.gold_income += 8;
            bonuses.commerce_development_bonus += 2;
        }
        TechnologyId::GranarySystem => {
            bonuses.city_effects.food_income += 8;
            bonuses.city_effects.food_upkeep_reduction_percent += 5;
        }
        TechnologyId::PriceStabilization => {
            bonuses.city_effects.gold_income += 6;
            bonuses.city_effects.gold_percent += 3;
        }
        TechnologyId::ArtisanRegisters => {
            bonuses.city_effects.materials_income += 4;
            bonuses.city_effects.materials_percent += 2;
        }
        TechnologyId::CanalRestoration => {
            bonuses.city_effects.food_percent += 4;
            bonuses.city_effects.population_growth += 8;
        }
        TechnologyId::TradePasses => {
            bonuses.city_effects.gold_percent += 4;
            bonuses.city_effects.order -= 1;
        }
        TechnologyId::BureaucraticRecords => {
            bonuses.city_effects.upkeep_reduction_percent += 5;
            bonuses.city_effects.order += 1;
        }
        TechnologyId::EverNormalGranary => {
            bonuses.city_effects.food_income += 12;
            bonuses.city_effects.food_upkeep_reduction_percent += 8;
            bonuses.city_effects.order += 1;
        }
        TechnologyId::WorkshopGuilds => {
            bonuses.city_effects.materials_income += 6;
            bonuses.city_effects.materials_percent += 4;
        }
        TechnologyId::CommanderyReviews => {
            bonuses.development_bonus += 3;
            bonuses.order_development_bonus += 1;
        }
        TechnologyId::CanalTaxation => {
            bonuses.city_effects.gold_percent += 5;
            bonuses.city_effects.food_percent += 5;
            bonuses.city_effects.upkeep_reduction_percent += 5;
        }
        TechnologyId::MinistryOfFinance => {
            bonuses.city_effects.gold_percent += 4;
            bonuses.city_effects.food_percent += 4;
            bonuses.city_effects.materials_percent += 4;
            bonuses.research_cost_reduction_percent += 10;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResearchStartOutcome {
    pub cost_paid: i32,
    pub resumed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TechnologyError {
    #[error("{0} 已完成")]
    Completed(String),
    #[error("需要先完成 {0}")]
    MissingPrerequisite(String),
    #[error("金钱不足，太学研习立项需要 {required} 金，当前 {available} 金")]
    InsufficientGold { required: i32, available: i32 },
}
