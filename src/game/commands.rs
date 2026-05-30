use super::history_db::{HistoricalCatalog, LifeEventKind};
use super::ids::{CityId, OfficerId};
use super::model::*;
use super::officer::{Officer, OfficerStats, OfficerStatus};
use std::collections::BTreeSet;

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
        CommandKind::Recruit { amount } => {
            if *amount == 0 || *amount > 5000 {
                return Err(CommandError::Invalid(
                    "征兵数量必须在 1-5000 之间".to_string(),
                ));
            }
            let gold_cost = recruit_gold_cost(*amount);
            let food_cost = recruit_food_cost(*amount);
            if city.gold < gold_cost || city.food < food_cost {
                return Err(CommandError::Invalid(format!(
                    "征兵需要 {gold_cost} 金和 {food_cost} 粮"
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

fn apply_command(state: &mut GameState, command: &Command, report: &mut TurnReport) {
    match &command.kind {
        CommandKind::Develop { focus } => apply_develop(state, command, focus, report),
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

fn apply_recruit(state: &mut GameState, command: &Command, amount: u32, report: &mut TurnReport) {
    let officer = state
        .officers
        .get(command.officer_id.as_ref().unwrap())
        .unwrap();
    let city = state.cities.get_mut(&command.city_id).unwrap();
    let gold_cost = recruit_gold_cost(amount);
    let food_cost = recruit_food_cost(amount);
    city.gold -= gold_cost;
    city.food -= food_cost;
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
    {
        let source = state.cities.get_mut(&command.city_id).unwrap();
        source.troops -= troops;
    }
    {
        let target = state.cities.get_mut(target_city_id).unwrap();
        target.troops += troops;
    }
    for officer_id in officer_ids {
        if let Some(officer) = state.officers.get_mut(officer_id) {
            officer.city_id = Some(target_city_id.to_string());
        }
    }
    report.info(format!(
        "{} 向 {} 调动 {} 兵、{} 名武将",
        source_name,
        target_name,
        troops,
        officer_ids.len()
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
    let attacker_leadership = u32::from(attacker.stats.leadership);
    let attacker_strength = u32::from(attacker.stats.strength);
    let source_city = state.cities[&command.city_id].clone();
    let target_city = state.cities[target_city_id].clone();
    let defender_governor = target_city
        .governor_id
        .as_ref()
        .and_then(|id| state.officers.get(id));
    let defender_leadership = defender_governor
        .map(|officer| u32::from(officer.stats.leadership))
        .unwrap_or(45);

    let attack_score = troops * (60 + u32::from(source_city.training)) / 100
        + attacker_leadership * 18
        + attacker_strength * 10;
    let defense_score = target_city.troops * (55 + u32::from(target_city.training)) / 100
        + u32::from(target_city.defense) * 12
        + defender_leadership * 15
        + u32::from(target_city.order) * 5;
    let noise = battle_noise(state.turn, &command.city_id, target_city_id);
    let attacker_wins = attack_score as i32 + noise > defense_score as i32;

    {
        let source = state.cities.get_mut(&command.city_id).unwrap();
        source.troops = source.troops.saturating_sub(troops);
    }

    if attacker_wins {
        let attacker_loss = troops.saturating_mul(35) / 100;
        let surviving_troops = troops.saturating_sub(attacker_loss).max(100);
        let old_faction = target_city.faction_id.clone();
        {
            let target = state.cities.get_mut(target_city_id).unwrap();
            target.faction_id = command.issuer_faction_id.clone();
            target.troops = surviving_troops;
            target.training = source_city.training.saturating_div(2).max(20);
            target.governor_id = Some(command.officer_id.clone().unwrap());
            target.order = target.order.saturating_sub(12);
            target.clamp_fields();
        }
        if let Some(officer) = state.officers.get_mut(command.officer_id.as_ref().unwrap()) {
            officer.city_id = Some(target_city_id.to_string());
        }
        retreat_defenders(state, target_city_id, &old_faction);
        report.info(format!(
            "{} 攻下 {}，剩余兵力 {}",
            attacker_name, target_city.name, surviving_troops
        ));
    } else {
        let attacker_loss = troops.saturating_mul(60) / 100;
        let defender_loss = troops.saturating_mul(30) / 100;
        {
            let source = state.cities.get_mut(&command.city_id).unwrap();
            source.troops += troops.saturating_sub(attacker_loss);
        }
        {
            let target = state.cities.get_mut(target_city_id).unwrap();
            target.troops = target.troops.saturating_sub(defender_loss);
            target.training = target.training.saturating_sub(4);
        }
        report.info(format!(
            "{} 进攻 {} 失败，损失 {}",
            attacker_name, target_city.name, attacker_loss
        ));
    }
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

fn apply_monthly_income(state: &mut GameState, report: &mut TurnReport) {
    let player_faction_id = state.player_faction_id.clone();
    let mut total_gold_income = 0;
    let mut total_food_income = 0;
    let mut player_gold_income = 0;
    let mut player_food_income = 0;
    let mut player_city_count = 0;

    for city in state.cities.values_mut() {
        let gold_income = i32::from(city.commerce) / 4 + (city.population / 20_000) as i32;
        let food_income = i32::from(city.agriculture) / 3 + (city.population / 18_000) as i32;
        city.gold += gold_income;
        city.food += food_income;
        city.order = city.order.saturating_add(1).min(100);
        city.clamp_fields();

        total_gold_income += gold_income;
        total_food_income += food_income;
        if city.faction_id == player_faction_id {
            player_city_count += 1;
            player_gold_income += gold_income;
            player_food_income += food_income;
        }
    }
    report.info(format!(
        "各城完成月度税粮结算：全图金 +{total_gold_income}、粮 +{total_food_income}；玩家 {player_city_count} 城收入金 +{player_gold_income}、粮 +{player_food_income}"
    ));
}

fn append_turn_summary(state: &GameState, report: &mut TurnReport) {
    let mut player_city_count = 0;
    let mut player_troops = 0;
    let mut player_gold = 0;
    let mut player_food = 0;

    for city in state
        .cities
        .values()
        .filter(|city| city.faction_id == state.player_faction_id)
    {
        player_city_count += 1;
        player_troops += city.troops;
        player_gold += city.gold;
        player_food += city.food;
    }

    let alive_factions = state
        .factions
        .keys()
        .filter(|faction_id| state.faction_alive(faction_id))
        .count();

    report.info(format!(
        "进入 {}年{}月，存续势力 {} 个；玩家控制 {} 城，兵力 {}，金 {}，粮 {}",
        state.year,
        state.month,
        alive_factions,
        player_city_count,
        player_troops,
        player_gold,
        player_food
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

fn recruit_gold_cost(amount: u32) -> i32 {
    (amount / 10) as i32 + 30
}

fn recruit_food_cost(amount: u32) -> i32 {
    (amount / 4) as i32 + 80
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
