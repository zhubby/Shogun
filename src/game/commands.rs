use super::city::{
    CITY_MAX_LEVEL, City, CityEconomyEffects, FACILITY_MAX_LEVEL, FacilityKind, ResourceCost,
    city_core_upgrade_cost, facility_upgrade_cost, project_city_monthly_change_with_effects,
};
use super::history_db::{HistoricalCatalog, LifeEventKind};
use super::ids::{CityId, OfficerId};
use super::model::*;
use super::officer::{
    Officer, OfficerStats, OfficerStatus, OfficialPostEffect, official_post_spec,
    official_rank_salary_bonus,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct CommandReservations {
    city_ids: BTreeSet<CityId>,
    officer_ids: BTreeSet<OfficerId>,
}

pub fn queue_player_command(state: &mut GameState, command: Command) -> Result<(), CommandError> {
    if state.status != GameStatus::Running {
        return Err(CommandError::Invalid("游戏已经结束".to_string()));
    }
    if command.issuer_faction_id != state.player_faction_id {
        return Err(CommandError::Invalid("玩家只能提交己方命令".to_string()));
    }

    let mut reservations = CommandReservations::default();
    for pending in &state.pending_commands {
        reserve_command(state, pending, &mut reservations)?;
    }
    validate_command(state, &command, &mut reservations)?;
    state.pending_commands.push(command);
    Ok(())
}

pub fn resolve_command_batch(state: &mut GameState, commands: Vec<Command>) -> TurnReport {
    resolve_command_batch_inner(state, commands, None)
}

pub fn resolve_command_batch_with_history<C: HistoricalCatalog>(
    state: &mut GameState,
    commands: Vec<Command>,
    catalog: &C,
) -> TurnReport {
    resolve_command_batch_inner(state, commands, Some(catalog as &dyn HistoricalCatalog))
}

fn resolve_command_batch_inner(
    state: &mut GameState,
    commands: Vec<Command>,
    catalog: Option<&dyn HistoricalCatalog>,
) -> TurnReport {
    let mut report = TurnReport::new(state);
    let mut reservations = CommandReservations::default();
    let command_count = commands.len();
    let mut executed_commands = 0;
    let mut rejected_commands = 0;

    for command in commands {
        match validate_command(state, &command, &mut reservations) {
            Ok(()) => {
                executed_commands += 1;
                apply_command(state, &command, &mut report);
            }
            Err(error) => {
                rejected_commands += 1;
                report.warning(format!("命令被拒绝: {} ({error})", command.summary()));
            }
        }
    }
    report.info(format!(
        "本月处理军令 {command_count} 条，执行 {executed_commands} 条，拒绝 {rejected_commands} 条"
    ));

    resolve_due_army_movements(state, &mut report);
    apply_monthly_income(state, &mut report);
    state.pending_commands.clear();
    state.refresh_status();
    if state.status == GameStatus::Running {
        state.advance_month();
        if let Some(catalog) = catalog {
            apply_due_life_events(state, catalog, &mut report);
            state.refresh_status();
        }
    }
    append_turn_summary(state, &mut report);
    state.reports.push(report.clone());
    report
}

pub fn validate_command_for_state(
    state: &GameState,
    command: &Command,
) -> Result<(), CommandError> {
    let mut reservations = CommandReservations::default();
    validate_command(state, command, &mut reservations)
}

pub fn appoint_official_post(
    state: &mut GameState,
    faction_id: &str,
    officer_id: &str,
    office_id: &str,
) -> Result<(), CommandError> {
    if official_post_spec(office_id).is_none() {
        return Err(CommandError::Invalid(format!("官职 {office_id} 不存在")));
    }
    let officer = state
        .officers
        .get(officer_id)
        .ok_or_else(|| CommandError::Invalid(format!("武将 {officer_id} 不存在")))?;
    if officer.faction_id != faction_id {
        return Err(CommandError::Invalid(format!(
            "{} 不是己方武将",
            officer.name
        )));
    }
    if !officer.is_active() {
        return Err(CommandError::Invalid(format!(
            "{} 当前不可授官",
            officer.name
        )));
    }

    for other in state.officers.values_mut() {
        if other.faction_id == faction_id
            && other.id != officer_id
            && other.office_id.as_deref() == Some(office_id)
        {
            other.office_id = None;
        }
    }
    state
        .officers
        .get_mut(officer_id)
        .ok_or_else(|| CommandError::Invalid(format!("武将 {officer_id} 不存在")))?
        .office_id = Some(office_id.to_string());
    Ok(())
}

pub fn dismiss_official_post(
    state: &mut GameState,
    faction_id: &str,
    officer_id: &str,
) -> Result<(), CommandError> {
    let officer = state
        .officers
        .get_mut(officer_id)
        .ok_or_else(|| CommandError::Invalid(format!("武将 {officer_id} 不存在")))?;
    if officer.faction_id != faction_id {
        return Err(CommandError::Invalid(format!(
            "{} 不是己方武将",
            officer.name
        )));
    }
    officer.office_id = None;
    Ok(())
}

fn validate_command(
    state: &GameState,
    command: &Command,
    reservations: &mut CommandReservations,
) -> Result<(), CommandError> {
    let city = state
        .cities
        .get(&command.city_id)
        .ok_or_else(|| CommandError::Invalid(format!("城池 {} 不存在", command.city_id)))?;
    if city.faction_id != command.issuer_faction_id {
        return Err(CommandError::Invalid(format!("{} 不是己方城池", city.name)));
    }
    if reservations.city_ids.contains(&command.city_id) {
        return Err(CommandError::Invalid(format!(
            "{} 本月已经有命令",
            city.name
        )));
    }

    let mut used_officers = BTreeSet::new();
    let officer_id = command
        .officer_id
        .as_ref()
        .ok_or_else(|| CommandError::Invalid("命令必须选择执行武将".to_string()))?;
    validate_officer_in_city(
        state,
        officer_id,
        &command.city_id,
        &command.issuer_faction_id,
    )?;
    used_officers.insert(officer_id.clone());

    match &command.kind {
        CommandKind::Develop { .. } => {
            if city.gold < 80 {
                return Err(CommandError::Invalid("开发需要至少 80 金".to_string()));
            }
        }
        CommandKind::UpgradeCityCore => {
            if city.level >= CITY_MAX_LEVEL {
                return Err(CommandError::Invalid("城镇核心已达最高等级".to_string()));
            }
            if city.order < 45 {
                return Err(CommandError::Invalid(
                    "治安低于 45，不能升级城镇核心".to_string(),
                ));
            }
            let cost = city_core_upgrade_cost(city.level + 1);
            ensure_city_can_pay(city, cost, "升级城镇核心")?;
        }
        CommandKind::BuildFacility { kind } => {
            let target_level = next_facility_level(city, *kind)?;
            let cost = facility_upgrade_cost(*kind, target_level);
            ensure_city_can_pay(city, cost, "建设设施")?;
        }
        CommandKind::Recruit { amount } => {
            if *amount == 0 || *amount > 5000 {
                return Err(CommandError::Invalid(
                    "征兵数量必须在 1-5000 之间".to_string(),
                ));
            }
            let cost = recruit_cost(*amount);
            if city.gold < cost.gold || city.food < cost.food {
                return Err(CommandError::Invalid(format!(
                    "征兵需要 {} 金和 {} 粮",
                    cost.gold, cost.food
                )));
            }
            if city.population < amount.saturating_mul(2) {
                return Err(CommandError::Invalid("人口不足以征兵".to_string()));
            }
        }
        CommandKind::Train => {
            if city.troops == 0 {
                return Err(CommandError::Invalid("没有士兵可训练".to_string()));
            }
            if city.gold < 40 {
                return Err(CommandError::Invalid("训练需要至少 40 金".to_string()));
            }
        }
        CommandKind::AppointGovernor { target_officer_id } => {
            validate_officer_in_city(
                state,
                target_officer_id,
                &command.city_id,
                &command.issuer_faction_id,
            )?;
            used_officers.insert(target_officer_id.clone());
        }
        CommandKind::Transfer {
            target_city_id,
            troops,
            officer_ids,
        } => {
            let target = state.cities.get(target_city_id).ok_or_else(|| {
                CommandError::Invalid(format!("目标城池 {target_city_id} 不存在"))
            })?;
            if target_city_id == &command.city_id {
                return Err(CommandError::Invalid("不能调动到同一座城".to_string()));
            }
            if target.faction_id != command.issuer_faction_id {
                return Err(CommandError::Invalid("只能向己方城池调动".to_string()));
            }
            if !state.are_adjacent(&command.city_id, target_city_id) {
                return Err(CommandError::Invalid("调动目标必须邻接".to_string()));
            }
            if *troops == 0 && officer_ids.is_empty() {
                return Err(CommandError::Invalid("调动必须包含兵力或武将".to_string()));
            }
            if *troops > city.troops {
                return Err(CommandError::Invalid("调动兵力超过城池驻军".to_string()));
            }
            for transfer_officer_id in officer_ids {
                validate_officer_in_city(
                    state,
                    transfer_officer_id,
                    &command.city_id,
                    &command.issuer_faction_id,
                )?;
                used_officers.insert(transfer_officer_id.clone());
            }
        }
        CommandKind::Expedition {
            target_city_id,
            troops,
        } => {
            if *troops == 0 {
                return Err(CommandError::Invalid("出征兵力必须大于 0".to_string()));
            }
            if *troops > city.troops {
                return Err(CommandError::Invalid("出征兵力超过城池驻军".to_string()));
            }
            let target = state.cities.get(target_city_id).ok_or_else(|| {
                CommandError::Invalid(format!("目标城池 {target_city_id} 不存在"))
            })?;
            if target.faction_id == command.issuer_faction_id {
                return Err(CommandError::Invalid("不能攻击己方城池".to_string()));
            }
            if !state.are_adjacent(&command.city_id, target_city_id) {
                return Err(CommandError::Invalid("出征目标必须邻接".to_string()));
            }
            if state
                .relation(&command.issuer_faction_id, &target.faction_id)
                .is_some_and(|relation| relation.has_active_truce(state.turn))
            {
                return Err(CommandError::Invalid("停战期内不能出征".to_string()));
            }
        }
        CommandKind::Diplomacy {
            target_faction_id, ..
        } => {
            if target_faction_id == &command.issuer_faction_id {
                return Err(CommandError::Invalid("不能对自己外交".to_string()));
            }
            if !state.factions.contains_key(target_faction_id) {
                return Err(CommandError::Invalid("目标势力不存在".to_string()));
            }
        }
    }

    for used_officer in used_officers {
        if reservations.officer_ids.contains(&used_officer) {
            return Err(CommandError::Invalid(format!(
                "武将 {used_officer} 本月已经行动"
            )));
        }
        reservations.officer_ids.insert(used_officer);
    }
    reservations.city_ids.insert(command.city_id.clone());
    Ok(())
}

fn reserve_command(
    state: &GameState,
    command: &Command,
    reservations: &mut CommandReservations,
) -> Result<(), CommandError> {
    validate_command(state, command, reservations)
}

fn validate_officer_in_city(
    state: &GameState,
    officer_id: &str,
    city_id: &str,
    faction_id: &str,
) -> Result<(), CommandError> {
    let officer = state
        .officers
        .get(officer_id)
        .ok_or_else(|| CommandError::Invalid(format!("武将 {officer_id} 不存在")))?;
    if officer.faction_id != faction_id {
        return Err(CommandError::Invalid(format!(
            "{} 不是己方武将",
            officer.name
        )));
    }
    if officer.city_id.as_deref() != Some(city_id) {
        return Err(CommandError::Invalid(format!(
            "{} 不在命令城池",
            officer.name
        )));
    }
    if !officer.is_active() {
        return Err(CommandError::Invalid(format!(
            "{} 当前不可行动",
            officer.name
        )));
    }
    Ok(())
}

fn ensure_city_can_pay(city: &City, cost: ResourceCost, action: &str) -> Result<(), CommandError> {
    if city.gold < cost.gold || city.food < cost.food || city.materials < cost.materials {
        return Err(CommandError::Invalid(format!(
            "{action}需要 {} 金、{} 粮、{} 建材",
            cost.gold, cost.food, cost.materials
        )));
    }
    Ok(())
}

fn next_facility_level(city: &City, kind: FacilityKind) -> Result<u8, CommandError> {
    if let Some(facility) = city.facility(kind) {
        if facility.level >= FACILITY_MAX_LEVEL {
            return Err(CommandError::Invalid("设施已达最高等级".to_string()));
        }
        let target_level = facility.level + 1;
        if target_level > city.level {
            return Err(CommandError::Invalid(
                "设施等级不能超过城镇核心等级".to_string(),
            ));
        }
        return Ok(target_level);
    }

    if city.facilities.len() >= city.facility_slots() {
        return Err(CommandError::Invalid("城镇设施槽位已满".to_string()));
    }
    Ok(1)
}

fn apply_command(state: &mut GameState, command: &Command, report: &mut TurnReport) {
    match &command.kind {
        CommandKind::Develop { focus } => apply_develop(state, command, focus, report),
        CommandKind::UpgradeCityCore => apply_upgrade_city_core(state, command, report),
        CommandKind::BuildFacility { kind } => apply_build_facility(state, command, *kind, report),
        CommandKind::Recruit { amount } => apply_recruit(state, command, *amount, report),
        CommandKind::Train => apply_train(state, command, report),
        CommandKind::AppointGovernor { target_officer_id } => {
            apply_appoint_governor(state, command, target_officer_id, report)
        }
        CommandKind::Transfer {
            target_city_id,
            troops,
            officer_ids,
        } => apply_transfer(state, command, target_city_id, *troops, officer_ids, report),
        CommandKind::Expedition {
            target_city_id,
            troops,
        } => apply_expedition(state, command, target_city_id, *troops, report),
        CommandKind::Diplomacy {
            target_faction_id,
            proposal,
        } => apply_diplomacy(state, command, target_faction_id, proposal, report),
    }
}

fn apply_develop(
    state: &mut GameState,
    command: &Command,
    focus: &DevelopmentFocus,
    report: &mut TurnReport,
) {
    let officer = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let gain = 8 + u16::from(officer.stats.politics) / 5;
    let city = state.cities.get_mut(&command.city_id).unwrap();
    city.gold -= 80;
    match focus {
        DevelopmentFocus::Agriculture => city.agriculture += gain,
        DevelopmentFocus::Commerce => city.commerce += gain,
        DevelopmentFocus::Defense => city.defense += gain,
        DevelopmentFocus::Order => {
            city.order = city
                .order
                .saturating_add((4 + officer.stats.charm / 15).min(12));
        }
    }
    city.clamp_fields();
    report.info(format!("{} 执行开发，{} 有所提升", officer.name, city.name));
}

fn apply_upgrade_city_core(state: &mut GameState, command: &Command, report: &mut TurnReport) {
    let officer = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let city = state.cities.get_mut(&command.city_id).unwrap();
    let next_level = city.level + 1;
    let cost = city_core_upgrade_cost(next_level);
    city.gold -= cost.gold;
    city.food -= cost.food;
    city.materials -= cost.materials;
    city.level = next_level.min(CITY_MAX_LEVEL);
    city.order = city.order.saturating_sub(2);
    city.clamp_fields();
    report.info(format!(
        "{} 主持扩建，{} 城镇核心升至 {} 级",
        officer.name, city.name, city.level
    ));
}

fn apply_build_facility(
    state: &mut GameState,
    command: &Command,
    kind: FacilityKind,
    report: &mut TurnReport,
) {
    let officer = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let city = state.cities.get_mut(&command.city_id).unwrap();
    let target_level = next_facility_level(city, kind).unwrap();
    let cost = facility_upgrade_cost(kind, target_level);
    city.gold -= cost.gold;
    city.food -= cost.food;
    city.materials -= cost.materials;
    if let Some(facility) = city
        .facilities
        .iter_mut()
        .find(|facility| facility.kind == kind)
    {
        facility.level = target_level;
    } else {
        city.facilities.push(super::city::CityFacility {
            kind,
            level: target_level,
        });
    }
    city.order = city.order.saturating_sub(1);
    city.clamp_fields();
    report.info(format!(
        "{} 主持建设，{} 的 {:?} 达到 {} 级",
        officer.name, city.name, kind, target_level
    ));
}

fn apply_recruit(state: &mut GameState, command: &Command, amount: u32, report: &mut TurnReport) {
    let officer = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let city = state.cities.get_mut(&command.city_id).unwrap();
    let cost = recruit_cost(amount);
    city.gold -= cost.gold;
    city.food -= cost.food;
    city.population = city.population.saturating_sub(amount * 2);
    city.troops += amount;
    city.order = city.order.saturating_sub(2);
    city.clamp_fields();
    report.info(format!("{} 在 {} 征兵 {}", officer.name, city.name, amount));
}

fn apply_train(state: &mut GameState, command: &Command, report: &mut TurnReport) {
    let officer = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let city = state.cities.get_mut(&command.city_id).unwrap();
    city.gold -= 40;
    city.training = city
        .training
        .saturating_add((6 + officer.stats.leadership / 12).min(15));
    city.clamp_fields();
    report.info(format!("{} 训练 {} 驻军", officer.name, city.name));
}

fn apply_appoint_governor(
    state: &mut GameState,
    command: &Command,
    target_officer_id: &str,
    report: &mut TurnReport,
) {
    let officer_name = state
        .officers
        .get(target_officer_id)
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| target_officer_id.to_string());
    let city = state.cities.get_mut(&command.city_id).unwrap();
    city.governor_id = Some(target_officer_id.to_string());
    report.info(format!("{} 被任命为 {} 太守", officer_name, city.name));
}

fn apply_transfer(
    state: &mut GameState,
    command: &Command,
    target_city_id: &str,
    troops: u32,
    officer_ids: &[OfficerId],
    report: &mut TurnReport,
) {
    let source_name = state.cities[&command.city_id].name.clone();
    let target_name = state.cities[target_city_id].name.clone();
    let distance_li = state
        .road_distance_li(&command.city_id, target_city_id)
        .unwrap_or_default();
    let travel_months = travel_months_for_distance(distance_li);
    let source_training = state.cities[&command.city_id].training;
    let moving_officers = movement_officer_ids(command.officer_id.as_ref().unwrap(), officer_ids);
    let moving_officer_count = moving_officers.len();
    {
        let source = state.cities.get_mut(&command.city_id).unwrap();
        source.troops -= troops;
    }
    for officer_id in &moving_officers {
        if let Some(officer) = state.officers.get_mut(officer_id) {
            officer.city_id = None;
        }
    }
    state.army_movements.push(ArmyMovement {
        kind: ArmyMovementKind::Transfer,
        issuer_faction_id: command.issuer_faction_id.clone(),
        source_city_id: command.city_id.clone(),
        target_city_id: target_city_id.to_string(),
        commander_id: command.officer_id.clone().unwrap(),
        officer_ids: moving_officers,
        troops,
        training: source_training,
        distance_li,
        departure_turn: state.turn,
        arrival_turn: state.turn + travel_months,
    });
    report.info(format!(
        "{} 向 {} 调动出发：{} 兵、{} 名武将，距离 {} 里，预计行军 {} 月",
        source_name, target_name, troops, moving_officer_count, distance_li, travel_months
    ));
}

fn apply_expedition(
    state: &mut GameState,
    command: &Command,
    target_city_id: &str,
    troops: u32,
    report: &mut TurnReport,
) {
    let attacker = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let attacker_name = attacker.name.clone();
    let source_city = state.cities[&command.city_id].clone();
    let target_city = state.cities[target_city_id].clone();
    let distance_li = state
        .road_distance_li(&command.city_id, target_city_id)
        .unwrap_or_default();
    let travel_months = travel_months_for_distance(distance_li);
    {
        let source = state.cities.get_mut(&command.city_id).unwrap();
        source.troops = source.troops.saturating_sub(troops);
    }
    if let Some(officer) = state.officers.get_mut(command.officer_id.as_ref().unwrap()) {
        officer.city_id = None;
    }
    state.army_movements.push(ArmyMovement {
        kind: ArmyMovementKind::Expedition,
        issuer_faction_id: command.issuer_faction_id.clone(),
        source_city_id: command.city_id.clone(),
        target_city_id: target_city_id.to_string(),
        commander_id: command.officer_id.clone().unwrap(),
        officer_ids: vec![command.officer_id.clone().unwrap()],
        troops,
        training: source_city.training,
        distance_li,
        departure_turn: state.turn,
        arrival_turn: state.turn + travel_months,
    });
    report.info(format!(
        "{} 率兵从 {} 出征 {}：{} 兵，距离 {} 里，预计行军 {} 月",
        attacker_name, source_city.name, target_city.name, troops, distance_li, travel_months
    ));
}

fn movement_officer_ids(commander_id: &str, officer_ids: &[OfficerId]) -> Vec<OfficerId> {
    let mut moving = vec![commander_id.to_string()];
    for officer_id in officer_ids {
        if !moving.iter().any(|moving_id| moving_id == officer_id) {
            moving.push(officer_id.clone());
        }
    }
    moving
}

fn resolve_due_army_movements(state: &mut GameState, report: &mut TurnReport) {
    let movements = std::mem::take(&mut state.army_movements);
    let mut pending = Vec::new();
    for movement in movements {
        if movement.arrival_turn <= state.turn {
            resolve_army_movement_arrival(state, movement, report);
        } else {
            pending.push(movement);
        }
    }
    state.army_movements = pending;
}

fn resolve_army_movement_arrival(
    state: &mut GameState,
    movement: ArmyMovement,
    report: &mut TurnReport,
) {
    match movement.kind {
        ArmyMovementKind::Transfer => resolve_transfer_arrival(state, movement, report),
        ArmyMovementKind::Expedition => resolve_expedition_arrival(state, movement, report),
    }
}

fn resolve_transfer_arrival(
    state: &mut GameState,
    movement: ArmyMovement,
    report: &mut TurnReport,
) {
    let target_name = city_name(state, &movement.target_city_id);
    let target_owned = state
        .cities
        .get(&movement.target_city_id)
        .is_some_and(|city| city.faction_id == movement.issuer_faction_id);
    if target_owned {
        place_movement_at_city(state, &movement, &movement.target_city_id, movement.troops);
        report.info(format!(
            "调动队抵达 {}：{} 兵、{} 名武将入城",
            target_name,
            movement.troops,
            movement.officer_ids.len()
        ));
        return;
    }

    return_movement_to_friendly_city(
        state,
        &movement,
        movement.troops,
        format!("调动目标 {target_name} 已不属己方"),
        report,
    );
}

fn resolve_expedition_arrival(
    state: &mut GameState,
    movement: ArmyMovement,
    report: &mut TurnReport,
) {
    let Some(target_city) = state.cities.get(&movement.target_city_id).cloned() else {
        return_movement_to_friendly_city(
            state,
            &movement,
            movement.troops,
            format!("出征目标 {} 不存在", movement.target_city_id),
            report,
        );
        return;
    };

    if target_city.faction_id == movement.issuer_faction_id {
        place_movement_at_city(state, &movement, &movement.target_city_id, movement.troops);
        report.info(format!(
            "出征队抵达 {}，目标已属己方，{} 兵入城",
            target_city.name, movement.troops
        ));
        return;
    }

    if state
        .relation(&movement.issuer_faction_id, &target_city.faction_id)
        .is_some_and(|relation| relation.has_active_truce(state.turn))
    {
        return_movement_to_friendly_city(
            state,
            &movement,
            movement.troops,
            format!("{} 已处于停战期，出征队撤回", target_city.name),
            report,
        );
        return;
    }

    let Some(attacker) = state.officers.get(&movement.commander_id) else {
        return_movement_to_friendly_city(
            state,
            &movement,
            movement.troops,
            format!("主将 {} 不存在，出征队撤回", movement.commander_id),
            report,
        );
        return;
    };
    if !attacker.is_active() {
        return_movement_to_friendly_city(
            state,
            &movement,
            movement.troops,
            format!("主将 {} 当前不可行动，出征队撤回", attacker.name),
            report,
        );
        return;
    }

    resolve_expedition_battle(state, &movement, &target_city, report);
}

fn resolve_expedition_battle(
    state: &mut GameState,
    movement: &ArmyMovement,
    target_city: &City,
    report: &mut TurnReport,
) {
    let attacker = state.officers[&movement.commander_id].clone();
    let attacker_leadership = u32::from(attacker.stats.leadership);
    let attacker_strength = u32::from(attacker.stats.strength);
    let defender_governor = target_city
        .governor_id
        .as_ref()
        .and_then(|id| state.officers.get(id));
    let defender_leadership = defender_governor
        .map(|officer| u32::from(officer.stats.leadership))
        .unwrap_or(45);

    let attack_score = movement.troops * (60 + u32::from(movement.training)) / 100
        + attacker_leadership * 18
        + attacker_strength * 10;
    let defense_score = target_city.troops * (55 + u32::from(target_city.training)) / 100
        + u32::from(target_city.defense) * 12
        + defender_leadership * 15
        + u32::from(target_city.order) * 5;
    let noise = battle_noise(
        state.turn,
        &movement.source_city_id,
        &movement.target_city_id,
    );
    let attacker_wins = attack_score as i32 + noise > defense_score as i32;

    if attacker_wins {
        let attacker_loss = movement.troops.saturating_mul(35) / 100;
        let surviving_troops = movement.troops.saturating_sub(attacker_loss).max(100);
        let old_faction = target_city.faction_id.clone();
        {
            let target = state.cities.get_mut(&movement.target_city_id).unwrap();
            target.faction_id = movement.issuer_faction_id.clone();
            target.troops = surviving_troops;
            target.training = movement.training.saturating_div(2).max(20);
            target.governor_id = Some(movement.commander_id.clone());
            target.order = target.order.saturating_sub(12);
            target.clamp_fields();
        }
        place_movement_officers_at_city(state, movement, &movement.target_city_id);
        retreat_defenders(state, &movement.target_city_id, &old_faction);
        report.info(format!(
            "{} 攻下 {}，剩余兵力 {}",
            attacker.name, target_city.name, surviving_troops
        ));
    } else {
        let attacker_loss = movement.troops.saturating_mul(60) / 100;
        let defender_loss = movement.troops.saturating_mul(30) / 100;
        {
            let target = state.cities.get_mut(&movement.target_city_id).unwrap();
            target.troops = target.troops.saturating_sub(defender_loss);
            target.training = target.training.saturating_sub(4);
        }
        report.info(format!(
            "{} 进攻 {} 失败，损失 {}",
            attacker.name, target_city.name, attacker_loss
        ));
        return_movement_to_friendly_city(
            state,
            movement,
            movement.troops.saturating_sub(attacker_loss),
            "败军撤回".to_string(),
            report,
        );
    }
}

fn place_movement_at_city(
    state: &mut GameState,
    movement: &ArmyMovement,
    city_id: &str,
    troops: u32,
) {
    if let Some(city) = state.cities.get_mut(city_id) {
        city.troops = city.troops.saturating_add(troops);
    }
    place_movement_officers_at_city(state, movement, city_id);
}

fn place_movement_officers_at_city(state: &mut GameState, movement: &ArmyMovement, city_id: &str) {
    for officer_id in &movement.officer_ids {
        if let Some(officer) = state.officers.get_mut(officer_id)
            && officer.is_active()
        {
            officer.city_id = Some(city_id.to_string());
        }
    }
}

fn return_movement_to_friendly_city(
    state: &mut GameState,
    movement: &ArmyMovement,
    troops: u32,
    reason: String,
    report: &mut TurnReport,
) {
    let fallback_city_id = state
        .cities
        .get(&movement.source_city_id)
        .filter(|city| city.faction_id == movement.issuer_faction_id)
        .map(|city| city.id.clone())
        .or_else(|| {
            state
                .cities
                .values()
                .find(|city| city.faction_id == movement.issuer_faction_id)
                .map(|city| city.id.clone())
        });

    if let Some(city_id) = fallback_city_id {
        let city_name = city_name(state, &city_id);
        place_movement_at_city(state, movement, &city_id, troops);
        report.warning(format!("{reason}，队伍退回 {city_name}，保全 {troops} 兵"));
    } else {
        for officer_id in &movement.officer_ids {
            if let Some(officer) = state.officers.get_mut(officer_id)
                && officer.is_active()
            {
                officer.faction_id = "wild".to_string();
                officer.city_id = None;
                officer.office_id = None;
                officer.status = OfficerStatus::Wild;
            }
        }
        report.warning(format!("{reason}，无己方城池可退，{} 兵溃散", troops));
    }
}

fn city_name(state: &GameState, city_id: &str) -> String {
    state
        .cities
        .get(city_id)
        .map(|city| city.name.clone())
        .unwrap_or_else(|| city_id.to_string())
}

fn apply_diplomacy(
    state: &mut GameState,
    command: &Command,
    target_faction_id: &str,
    proposal: &DiplomacyProposal,
    report: &mut TurnReport,
) {
    let officer = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let officer_name = officer.name.clone();
    let score_delta =
        8 + (i16::from(officer.stats.politics) + i16::from(officer.stats.intelligence)) / 18;
    let turn = state.turn;
    let relation = state.relation_mut(&command.issuer_faction_id, target_faction_id);
    match proposal {
        DiplomacyProposal::ImproveRelations => {
            relation.score = (relation.score + score_delta).clamp(-100, 100);
            report.info(format!("{officer_name} 改善外交，关系提升 {score_delta}"));
        }
        DiplomacyProposal::Truce => {
            if relation.score >= -40 {
                relation.truce_until_turn = Some(turn + 6);
                relation.score = (relation.score + 5).clamp(-100, 100);
                report.info(format!("{officer_name} 达成 6 个月停战"));
            } else {
                relation.score = (relation.score + 3).clamp(-100, 100);
                report.warning(format!("{officer_name} 提议停战失败"));
            }
        }
        DiplomacyProposal::DeclareWar => {
            relation.truce_until_turn = None;
            relation.score = -80;
            report.info(format!("{officer_name} 宣战"));
        }
    }
}

fn retreat_defenders(state: &mut GameState, captured_city_id: &str, old_faction_id: &str) {
    let fallback_city = state
        .cities
        .values()
        .find(|city| city.faction_id == old_faction_id && city.id != captured_city_id)
        .map(|city| city.id.clone());
    for officer in state.officers.values_mut() {
        if officer.faction_id == old_faction_id
            && officer.city_id.as_deref() == Some(captured_city_id)
        {
            officer.city_id = fallback_city.clone();
        }
    }
}

pub fn officer_monthly_salary(officer: &Officer) -> i32 {
    let mut salary = officer_base_monthly_salary(officer);
    if let Some(spec) = officer.office_id.as_deref().and_then(official_post_spec) {
        salary += official_rank_salary_bonus(spec.rank);
    }
    salary
}

pub fn officer_base_monthly_salary(officer: &Officer) -> i32 {
    let stats = officer.stats;
    10 + (i32::from(stats.leadership)
        + i32::from(stats.strength)
        + i32::from(stats.intelligence)
        + i32::from(stats.politics)
        + i32::from(stats.charm))
        / 50
}

pub fn city_official_effects(state: &GameState, city_id: &str) -> CityEconomyEffects {
    let Some(city) = state.cities.get(city_id) else {
        return CityEconomyEffects::default();
    };
    let mut effects = CityEconomyEffects::default();
    for officer in state.officers.values().filter(|officer| {
        officer.is_active()
            && officer.city_id.as_deref() == Some(city_id)
            && officer.faction_id == city.faction_id
    }) {
        let Some(spec) = officer.office_id.as_deref().and_then(official_post_spec) else {
            continue;
        };
        effects.add(official_effect_to_city_effect(spec.effect));
    }
    effects
}

fn official_effect_to_city_effect(effect: OfficialPostEffect) -> CityEconomyEffects {
    CityEconomyEffects {
        gold_income: effect.gold_income,
        food_income: effect.food_income,
        materials_income: effect.materials_income,
        gold_percent: effect.gold_percent,
        food_percent: effect.food_percent,
        materials_percent: effect.materials_percent,
        population_growth: effect.population_growth,
        troop_recovery: effect.troop_recovery,
        order: effect.order,
        training: effect.training,
        defense: effect.defense,
        ..CityEconomyEffects::default()
    }
}

fn apply_monthly_income(state: &mut GameState, report: &mut TurnReport) {
    let player_faction_id = state.player_faction_id.clone();
    let salaries = city_salary_totals(state);
    let official_effects = city_official_effect_totals(state);
    let mut total_gold_delta = 0;
    let mut total_food_delta = 0;
    let mut total_materials_delta = 0;
    let mut total_population_delta = 0;
    let mut total_troop_delta = 0;
    let mut total_salary = 0;
    let mut player_gold_delta = 0;
    let mut player_food_delta = 0;
    let mut player_materials_delta = 0;
    let mut player_population_delta = 0;
    let mut player_troop_delta = 0;
    let mut player_salary = 0;
    let mut player_city_count = 0;

    for city in state.cities.values_mut() {
        let salary = salaries.get(&city.id).copied().unwrap_or_default();
        let extra_effects = official_effects.get(&city.id).copied().unwrap_or_default();
        let projection = project_city_monthly_change_with_effects(city, salary, extra_effects);
        city.gold += projection.net_gold;
        city.food += projection.net_food;
        city.materials += projection.net_materials;
        city.population = apply_i32_to_u32(city.population, projection.population_delta);
        city.troops = apply_i32_to_u32(city.troops, projection.troop_delta);
        city.order = apply_i32_to_u8(city.order, projection.order_delta, 0, 100);
        city.training = apply_i32_to_u8(city.training, projection.training_delta, 0, 100);
        city.defense = apply_i32_to_u16(city.defense, projection.defense_delta, 0, 999);
        city.clamp_fields();

        total_gold_delta += projection.net_gold;
        total_food_delta += projection.net_food;
        total_materials_delta += projection.net_materials;
        total_population_delta += projection.population_delta;
        total_troop_delta += projection.troop_delta;
        total_salary += salary;
        if city.faction_id == player_faction_id {
            player_city_count += 1;
            player_gold_delta += projection.net_gold;
            player_food_delta += projection.net_food;
            player_materials_delta += projection.net_materials;
            player_population_delta += projection.population_delta;
            player_troop_delta += projection.troop_delta;
            player_salary += salary;
        }
    }
    report.info(format!(
        "各城完成月度经济结算：全图金 {total_gold_delta:+}、粮 {total_food_delta:+}、建材 {total_materials_delta:+}、人口 {total_population_delta:+}、兵 {total_troop_delta:+}、俸禄 -{total_salary}；玩家 {player_city_count} 城金 {player_gold_delta:+}、粮 {player_food_delta:+}、建材 {player_materials_delta:+}、人口 {player_population_delta:+}、兵 {player_troop_delta:+}、俸禄 -{player_salary}"
    ));
}

fn city_salary_totals(state: &GameState) -> BTreeMap<CityId, i32> {
    let mut salaries = BTreeMap::new();
    for officer in state
        .officers
        .values()
        .filter(|officer| officer.is_active())
    {
        let Some(city_id) = &officer.city_id else {
            continue;
        };
        let Some(city) = state.cities.get(city_id) else {
            continue;
        };
        if city.faction_id != officer.faction_id {
            continue;
        }
        *salaries.entry(city_id.clone()).or_insert(0) += officer_monthly_salary(officer);
    }
    salaries
}

fn city_official_effect_totals(state: &GameState) -> BTreeMap<CityId, CityEconomyEffects> {
    let mut effects_by_city = BTreeMap::new();
    for officer in state
        .officers
        .values()
        .filter(|officer| officer.is_active())
    {
        let Some(city_id) = &officer.city_id else {
            continue;
        };
        let Some(city) = state.cities.get(city_id) else {
            continue;
        };
        if city.faction_id != officer.faction_id {
            continue;
        }
        let Some(spec) = officer.office_id.as_deref().and_then(official_post_spec) else {
            continue;
        };
        effects_by_city
            .entry(city_id.clone())
            .or_insert_with(CityEconomyEffects::default)
            .add(official_effect_to_city_effect(spec.effect));
    }
    effects_by_city
}

fn apply_i32_to_u32(value: u32, delta: i32) -> u32 {
    if delta >= 0 {
        value.saturating_add(delta as u32)
    } else {
        value.saturating_sub(delta.unsigned_abs())
    }
}

fn apply_i32_to_u8(value: u8, delta: i32, min: u8, max: u8) -> u8 {
    (i32::from(value) + delta).clamp(i32::from(min), i32::from(max)) as u8
}

fn apply_i32_to_u16(value: u16, delta: i32, min: u16, max: u16) -> u16 {
    (i32::from(value) + delta).clamp(i32::from(min), i32::from(max)) as u16
}

fn append_turn_summary(state: &GameState, report: &mut TurnReport) {
    let mut player_city_count = 0;
    let mut player_troops = 0;
    let mut player_gold = 0;
    let mut player_food = 0;
    let mut player_materials = 0;

    for city in state
        .cities
        .values()
        .filter(|city| city.faction_id == state.player_faction_id)
    {
        player_city_count += 1;
        player_troops += city.troops;
        player_gold += city.gold;
        player_food += city.food;
        player_materials += city.materials;
    }

    let alive_factions = state
        .factions
        .keys()
        .filter(|faction_id| state.faction_alive(faction_id))
        .count();

    report.info(format!(
        "进入 {}年{}月，存续势力 {} 个；玩家控制 {} 城，兵力 {}，金 {}，粮 {}，建材 {}",
        state.year,
        state.month,
        alive_factions,
        player_city_count,
        player_troops,
        player_gold,
        player_food,
        player_materials
    ));

    match &state.status {
        GameStatus::Running => {}
        GameStatus::Victory { reason } => report.info(format!("胜利：{reason}")),
        GameStatus::Defeat { reason } => report.warning(format!("失败：{reason}")),
    }
}

fn apply_due_life_events(
    state: &mut GameState,
    catalog: &dyn HistoricalCatalog,
    report: &mut TurnReport,
) {
    let Ok(events) = catalog.life_events_until(state.year, state.month) else {
        report.warning("读取武将履历事件失败");
        return;
    };

    for event in events {
        if state.applied_event_ids.contains(&event.id) {
            continue;
        }
        let officer_profile = match catalog.officer_profile(&event.officer_id) {
            Ok(profile) => profile,
            Err(_) => {
                report.warning(format!("读取武将 {} 资料失败", event.officer_id));
                continue;
            }
        };
        let officer_name = officer_profile
            .as_ref()
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| event.officer_id.clone());

        match event.kind {
            LifeEventKind::Appear | LifeEventKind::ServeFaction | LifeEventKind::MoveToCity => {
                let (target_faction_id, city_id, status) = resolve_life_event_assignment(
                    state,
                    event.faction_id.as_deref(),
                    event.city_id.as_deref(),
                );
                let stats = officer_profile
                    .as_ref()
                    .map(|profile| profile.stats)
                    .unwrap_or(OfficerStats {
                        leadership: 50,
                        strength: 50,
                        intelligence: 50,
                        politics: 50,
                        charm: 50,
                    });

                state
                    .officers
                    .entry(event.officer_id.clone())
                    .and_modify(|officer| {
                        officer.faction_id = target_faction_id.clone();
                        officer.city_id = city_id.clone();
                        officer.status = status.clone();
                        if let Some(loyalty) = event.loyalty {
                            officer.loyalty = loyalty;
                        }
                        officer.gender = officer_profile
                            .as_ref()
                            .map(|profile| profile.gender.clone())
                            .unwrap_or_default();
                        officer.profile = officer_profile.clone();
                    })
                    .or_insert(Officer {
                        id: event.officer_id.clone(),
                        name: officer_name.clone(),
                        faction_id: target_faction_id.clone(),
                        city_id: city_id.clone(),
                        office_id: None,
                        stats,
                        loyalty: event.loyalty.unwrap_or(75),
                        gender: officer_profile
                            .as_ref()
                            .map(|profile| profile.gender.clone())
                            .unwrap_or_default(),
                        status: status.clone(),
                        profile: officer_profile.clone(),
                    });

                report.info(format!(
                    "{} 于 {}年{}月进入局势",
                    officer_name, event.year, event.month
                ));
            }
            LifeEventKind::BecomeUnavailable | LifeEventKind::Die => {
                if let Some(officer) = state.officers.get_mut(&event.officer_id) {
                    officer.city_id = None;
                    officer.office_id = None;
                    officer.status = if event.kind == LifeEventKind::Die {
                        OfficerStatus::Dead
                    } else {
                        OfficerStatus::Unavailable
                    };
                    for city in state.cities.values_mut() {
                        if city.governor_id.as_deref() == Some(&event.officer_id) {
                            city.governor_id = None;
                        }
                    }
                    report.info(format!("{} 离开当前局势", officer.name));
                }
            }
        }

        state.applied_event_ids.insert(event.id);
    }
}

fn resolve_life_event_assignment(
    state: &GameState,
    requested_faction_id: Option<&str>,
    requested_city_id: Option<&str>,
) -> (String, Option<String>, OfficerStatus) {
    let target_faction_id = requested_faction_id
        .map(str::to_string)
        .or_else(|| {
            requested_city_id
                .and_then(|city_id| state.cities.get(city_id))
                .map(|city| city.faction_id.clone())
        })
        .unwrap_or_else(|| "wild".to_string());

    if target_faction_id == "wild" || !state.faction_alive(&target_faction_id) {
        return ("wild".to_string(), None, OfficerStatus::Wild);
    }

    let city_id = requested_city_id
        .and_then(|city_id| state.cities.get(city_id))
        .filter(|city| city.faction_id == target_faction_id)
        .map(|city| city.id.clone())
        .or_else(|| {
            state
                .cities
                .values()
                .find(|city| city.faction_id == target_faction_id)
                .map(|city| city.id.clone())
        });

    match city_id {
        Some(city_id) => (target_faction_id, Some(city_id), OfficerStatus::Active),
        None => ("wild".to_string(), None, OfficerStatus::Wild),
    }
}

pub fn recruit_cost(amount: u32) -> ResourceCost {
    ResourceCost {
        gold: (amount / 10) as i32 + 30,
        food: (amount / 4) as i32 + 80,
        materials: 0,
    }
}

fn battle_noise(turn: u32, source: &str, target: &str) -> i32 {
    let source_sum: u32 = source.bytes().map(u32::from).sum();
    let target_sum: u32 = target.bytes().map(u32::from).sum();
    ((turn + source_sum * 3 + target_sum * 7) % 21) as i32 - 10
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    Invalid(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CommandError {}
