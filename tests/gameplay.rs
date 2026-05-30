use shogun::game::*;
use std::collections::BTreeMap;

fn sample_game() -> GameState {
    ScenarioData::default_scenario()
        .unwrap()
        .build_game("liu_bei")
        .unwrap()
}

fn command(city_id: &str, officer_id: &str, kind: CommandKind) -> Command {
    Command {
        issuer_faction_id: "liu_bei".to_string(),
        city_id: city_id.to_string(),
        officer_id: Some(officer_id.to_string()),
        kind,
    }
}

#[test]
fn one_city_gets_only_one_command_per_month() {
    let mut game = sample_game();
    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "liu_bei",
            CommandKind::Develop {
                focus: DevelopmentFocus::Agriculture,
            },
        ),
    )
    .unwrap();

    let result = queue_player_command(
        &mut game,
        command("pingyuan", "guan_yu", CommandKind::Train),
    );

    assert!(result.is_err());
}

#[test]
fn one_officer_gets_only_one_command_per_month() {
    let mut game = sample_game();
    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "liu_bei",
            CommandKind::Develop {
                focus: DevelopmentFocus::Commerce,
            },
        ),
    )
    .unwrap();

    let result = queue_player_command(&mut game, command("xiapi", "liu_bei", CommandKind::Train));

    assert!(result.is_err());
}

#[test]
fn cannot_attack_non_adjacent_city() {
    let game = sample_game();
    let result = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "guan_yu",
            CommandKind::Expedition {
                target_city_id: "jianye".to_string(),
                troops: 1000,
            },
        ),
    );

    assert!(result.is_err());
}

#[test]
fn cannot_command_enemy_city() {
    let game = sample_game();
    let result = validate_command_for_state(
        &game,
        &Command {
            issuer_faction_id: "liu_bei".to_string(),
            city_id: "xuchang".to_string(),
            officer_id: Some("guan_yu".to_string()),
            kind: CommandKind::Train,
        },
    );

    assert!(result.is_err());
}

#[test]
fn development_changes_city_values() {
    let mut game = sample_game();
    let before = game.cities["pingyuan"].agriculture;
    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "liu_bei",
            CommandKind::Develop {
                focus: DevelopmentFocus::Agriculture,
            },
        ),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert!(game.cities["pingyuan"].agriculture > before);
}

#[test]
fn recruit_consumes_resources_and_adds_troops() {
    let mut game = sample_game();
    let before_troops = game.cities["xiapi"].troops;
    let before_gold = game.cities["xiapi"].gold;
    queue_player_command(
        &mut game,
        command("xiapi", "zhang_fei", CommandKind::Recruit { amount: 500 }),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert!(game.cities["xiapi"].troops > before_troops);
    assert!(game.cities["xiapi"].gold < before_gold);
}

#[test]
fn train_increases_training() {
    let mut game = sample_game();
    let before = game.cities["xiapi"].training;
    queue_player_command(&mut game, command("xiapi", "zhang_fei", CommandKind::Train)).unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert!(game.cities["xiapi"].training > before);
}

#[test]
fn appoint_governor_updates_city() {
    let mut game = sample_game();
    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "liu_bei",
            CommandKind::AppointGovernor {
                target_officer_id: "zhao_yun".to_string(),
            },
        ),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert_eq!(
        game.cities["pingyuan"].governor_id.as_deref(),
        Some("zhao_yun")
    );
}

#[test]
fn transfer_moves_troops_between_adjacent_owned_cities() {
    let mut game = sample_game();
    let before_source = game.cities["pingyuan"].troops;
    let before_target = game.cities["xiapi"].troops;
    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "liu_bei",
            CommandKind::Transfer {
                target_city_id: "xiapi".to_string(),
                troops: 700,
                officer_ids: Vec::new(),
            },
        ),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert_eq!(game.cities["pingyuan"].troops, before_source - 700);
    assert_eq!(game.cities["xiapi"].troops, before_target + 700);
}

#[test]
fn strong_expedition_can_capture_city() {
    let mut game = sample_game();
    {
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = 30_000;
        source.training = 100;
    }
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                troops: 25_000,
            },
        ),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert_eq!(game.cities["xuchang"].faction_id, "liu_bei");
}

#[test]
fn ai_json_provider_rejects_invalid_json_without_commands() {
    let provider = MockAiProvider {
        scripted: BTreeMap::from([("cao_cao".to_string(), "{not json".to_string())]),
    };
    let request = AiDecisionRequest::from_state(&sample_game(), "cao_cao");

    let response = provider.decide(request);

    assert!(response.commands.is_empty());
    assert_eq!(response.diagnostics.len(), 1);
}

#[test]
fn rule_ai_outputs_legal_commands_for_current_state() {
    let game = sample_game();
    let provider = RuleBasedAiProvider;
    let response = provider.decide(AiDecisionRequest::from_state(&game, "cao_cao"));

    assert!(!response.commands.is_empty());
    assert_eq!(
        legal_ai_commands(&game, &response).len(),
        response.commands.len()
    );
}

#[test]
fn save_manager_round_trips_multiple_slots() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    manager.save_slot("slot1", "第一档", &game).unwrap();
    game.year = 201;
    manager.save_slot("slot2", "第二档", &game).unwrap();

    let slots = manager.list_slots().unwrap();
    assert_eq!(slots.len(), 2);

    let loaded = manager.load_slot("slot2").unwrap();
    assert_eq!(loaded.year, 201);

    manager.delete_slot("slot1").unwrap();
    assert_eq!(manager.list_slots().unwrap().len(), 1);
}

#[test]
fn player_can_play_several_months_with_ai() {
    let mut game = sample_game();
    let provider = RuleBasedAiProvider;
    for _ in 0..3 {
        if let Some(city_id) = first_available_player_city(&game) {
            let officer_id = game.officers_in_city(&city_id).first().unwrap().id.clone();
            queue_player_command(
                &mut game,
                command(
                    &city_id,
                    &officer_id,
                    CommandKind::Develop {
                        focus: DevelopmentFocus::Commerce,
                    },
                ),
            )
            .unwrap();
        }
        finish_turn_with_ai(&mut game, &provider);
    }

    assert!(game.turn >= 4);
    assert!(!game.reports.is_empty());
}

#[test]
fn turn_report_includes_monthly_and_state_summaries() {
    let mut game = sample_game();

    let report = resolve_command_batch(&mut game, Vec::new());

    assert!(report
        .entries
        .iter()
        .any(|entry| entry.message.contains("本月处理军令 0 条")));
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.message.contains("月度税粮结算")));
    assert!(report
        .entries
        .iter()
        .any(|entry| entry.message.contains("玩家控制")));
}

#[test]
fn owning_all_cities_triggers_victory() {
    let mut game = sample_game();
    for city in game.cities.values_mut() {
        city.faction_id = "liu_bei".to_string();
    }

    resolve_command_batch(&mut game, Vec::new());

    assert!(matches!(game.status, GameStatus::Victory { .. }));
}

fn first_available_player_city(game: &GameState) -> Option<String> {
    game.cities
        .values()
        .find(|city| city.faction_id == game.player_faction_id)
        .map(|city| city.id.clone())
}
