use shogun::game::*;
use std::collections::BTreeMap;
use std::fs;

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
fn city_core_upgrade_consumes_resources_and_unlocks_growth() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.gold = 2_000;
        city.food = 2_000;
        city.materials = 2_000;
        city.order = 80;
    }
    let before = game.cities["pingyuan"].clone();
    queue_player_command(
        &mut game,
        command("pingyuan", "liu_bei", CommandKind::UpgradeCityCore),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    let city = &game.cities["pingyuan"];
    assert_eq!(city.level, before.level + 1);
    assert!(city.facility_slots() >= before.facility_slots());
    assert!(city.gold < before.gold);
    assert!(city.materials < before.materials);
}

#[test]
fn build_facility_adds_or_upgrades_facility() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.level = 4;
        city.gold = 2_000;
        city.food = 2_000;
        city.materials = 2_000;
        city.facilities = vec![CityFacility {
            kind: FacilityKind::Farmland,
            level: 1,
        }];
    }

    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "liu_bei",
            CommandKind::BuildFacility {
                kind: FacilityKind::Market,
            },
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert_eq!(
        game.cities["pingyuan"]
            .facility(FacilityKind::Market)
            .map(|facility| facility.level),
        Some(1)
    );

    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "liu_bei",
            CommandKind::BuildFacility {
                kind: FacilityKind::Farmland,
            },
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert_eq!(
        game.cities["pingyuan"]
            .facility(FacilityKind::Farmland)
            .map(|facility| facility.level),
        Some(2)
    );
}

#[test]
fn facility_construction_respects_slots_and_level_caps() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.level = 1;
        city.gold = 2_000;
        city.food = 2_000;
        city.materials = 2_000;
        city.facilities = vec![
            CityFacility {
                kind: FacilityKind::Farmland,
                level: 1,
            },
            CityFacility {
                kind: FacilityKind::Granary,
                level: 1,
            },
        ];
    }

    let no_slot = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "liu_bei",
            CommandKind::BuildFacility {
                kind: FacilityKind::Market,
            },
        ),
    );
    assert!(no_slot.is_err());

    game.cities.get_mut("pingyuan").unwrap().facilities.pop();
    let over_city_level = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "liu_bei",
            CommandKind::BuildFacility {
                kind: FacilityKind::Farmland,
            },
        ),
    );
    assert!(over_city_level.is_err());
}

#[test]
fn monthly_economy_applies_facilities_upkeep_growth_and_salaries() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.gold = 1_000;
        city.food = 1_000;
        city.materials = 500;
        city.facilities = vec![
            CityFacility {
                kind: FacilityKind::Market,
                level: 2,
            },
            CityFacility {
                kind: FacilityKind::Workshop,
                level: 2,
            },
            CityFacility {
                kind: FacilityKind::Barracks,
                level: 1,
            },
        ];
    }
    let before = game.cities["pingyuan"].clone();
    let salary = game
        .officers_in_city("pingyuan")
        .into_iter()
        .map(officer_monthly_salary)
        .sum();
    let projection = project_city_monthly_change(&before, salary);

    let report = resolve_command_batch(&mut game, Vec::new());

    let city = &game.cities["pingyuan"];
    assert_eq!(city.gold, before.gold + projection.net_gold);
    assert_eq!(city.food, before.food + projection.net_food);
    assert_eq!(city.materials, before.materials + projection.net_materials);
    assert_eq!(
        city.population,
        before.population + projection.population_delta as u32
    );
    assert!(city.troops > before.troops);
    assert!(projection.officer_salary > 0);
    assert!(
        report
            .entries
            .iter()
            .any(|entry| entry.message.contains("月度经济结算"))
    );
}

#[test]
fn technology_research_pays_full_cost_up_front() {
    let mut game = sample_game();
    let before_total = faction_total_gold(&game, "liu_bei");

    let outcome = start_research(&mut game, "liu_bei", TechnologyId::MilitiaDrill).unwrap();

    assert_eq!(outcome.cost_paid, 90);
    assert_eq!(faction_total_gold(&game, "liu_bei"), before_total - 90);
    let state = faction_technology_state(&game, "liu_bei").unwrap();
    assert_eq!(state.active, Some(TechnologyId::MilitiaDrill));
    assert!(state.funded.contains(&TechnologyId::MilitiaDrill));
}

#[test]
fn technology_research_rejects_insufficient_gold_without_progress() {
    let mut game = sample_game();
    for city in game
        .cities
        .values_mut()
        .filter(|city| city.faction_id == "liu_bei")
    {
        city.gold = 0;
    }

    let result = start_research(&mut game, "liu_bei", TechnologyId::MilitiaDrill);

    assert!(matches!(
        result,
        Err(TechnologyError::InsufficientGold {
            required: 90,
            available: 0
        })
    ));
    let state = faction_technology_state(&game, "liu_bei").unwrap();
    assert!(state.active.is_none());
    assert!(state.funded.is_empty());
    assert!(state.progress.is_empty());
}

#[test]
fn funded_technology_can_resume_without_repaying() {
    let mut game = sample_game();
    start_research(&mut game, "liu_bei", TechnologyId::MilitiaDrill).unwrap();
    start_research(&mut game, "liu_bei", TechnologyId::ArsenalLogistics).unwrap();
    let after_two_projects = faction_total_gold(&game, "liu_bei");

    let outcome = start_research(&mut game, "liu_bei", TechnologyId::MilitiaDrill).unwrap();

    assert!(outcome.resumed);
    assert_eq!(outcome.cost_paid, 0);
    assert_eq!(faction_total_gold(&game, "liu_bei"), after_two_projects);
    assert_eq!(
        faction_technology_state(&game, "liu_bei").unwrap().active,
        Some(TechnologyId::MilitiaDrill)
    );
}

#[test]
fn technology_completes_after_required_turns() {
    let mut game = sample_game();
    start_research(&mut game, "liu_bei", TechnologyId::MilitiaDrill).unwrap();

    resolve_command_batch(&mut game, Vec::new());
    assert!(
        !faction_technology_state(&game, "liu_bei")
            .unwrap()
            .completed
            .contains(&TechnologyId::MilitiaDrill)
    );
    let report = resolve_command_batch(&mut game, Vec::new());

    let state = faction_technology_state(&game, "liu_bei").unwrap();
    assert!(state.completed.contains(&TechnologyId::MilitiaDrill));
    assert!(state.active.is_none());
    assert!(
        report
            .entries
            .iter()
            .any(|entry| entry.message.contains("完成科技：乡勇操练"))
    );
}

#[test]
fn technology_prerequisites_are_enforced() {
    let mut game = sample_game();

    let result = start_research(&mut game, "liu_bei", TechnologyId::IronWeapons);

    assert!(matches!(
        result,
        Err(TechnologyError::MissingPrerequisite(name)) if name == "军械整备"
    ));
}

#[test]
fn completed_technology_affects_training_recruitment_and_income() {
    let mut game = sample_game();
    {
        let state = faction_technology_state_mut(&mut game, "liu_bei");
        state.completed.insert(TechnologyId::MilitiaDrill);
        state.completed.insert(TechnologyId::ArsenalLogistics);
        state.completed.insert(TechnologyId::HouseholdRegisters);
    }

    let discounted = recruit_cost_for_faction(&game, "liu_bei", 500);
    assert!(discounted.gold < recruit_cost(500).gold);

    let before_training = game.cities["xiapi"].training;
    queue_player_command(&mut game, command("xiapi", "zhang_fei", CommandKind::Train)).unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);
    assert!(game.cities["xiapi"].training >= before_training + 10);

    let before_population = game.cities["pingyuan"].population;
    resolve_command_batch(&mut game, Vec::new());
    assert!(game.cities["pingyuan"].population > before_population);
}

#[test]
fn completed_scout_roads_reduces_travel_months() {
    let mut game = sample_game();
    let baseline = game.travel_months_between("jiangxia", "jianye").unwrap();
    faction_technology_state_mut(&mut game, "sun_quan")
        .completed
        .insert(TechnologyId::ScoutRoads);
    let distance = game.road_distance_li("jiangxia", "jianye").unwrap();

    assert_eq!(
        travel_months_for_faction(&game, "sun_quan", distance),
        baseline.saturating_sub(1).max(1)
    );
}

#[test]
fn officer_without_office_uses_ability_salary() {
    let game = sample_game();
    let officer = &game.officers["liu_bei"];
    let expected = 10
        + (i32::from(officer.stats.leadership)
            + i32::from(officer.stats.strength)
            + i32::from(officer.stats.intelligence)
            + i32::from(officer.stats.politics)
            + i32::from(officer.stats.charm))
            / 50;

    assert_eq!(officer_base_monthly_salary(officer), expected);
    assert_eq!(officer_monthly_salary(officer), expected);
}

#[test]
fn official_post_increases_salary_and_affects_monthly_city_economy() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.gold = 1_000;
        city.food = 1_000;
        city.materials = 500;
        city.facilities.clear();
    }
    let base_salary = officer_monthly_salary(&game.officers["liu_bei"]);
    appoint_official_post(&mut game, "liu_bei", "liu_bei", "taifu").unwrap();
    let appointed_salary = officer_monthly_salary(&game.officers["liu_bei"]);
    assert_eq!(
        appointed_salary,
        base_salary + official_rank_salary_bonus(OfficialRank::WanShi)
    );

    let before = game.cities["pingyuan"].clone();
    let salary = game
        .officers_in_city("pingyuan")
        .into_iter()
        .map(officer_monthly_salary)
        .sum();
    let projection = project_city_monthly_change_with_effects(
        &before,
        salary,
        city_official_effects(&game, "pingyuan"),
    );

    resolve_command_batch(&mut game, Vec::new());

    assert_eq!(
        game.cities["pingyuan"].gold,
        before.gold + projection.net_gold
    );
    assert_eq!(
        game.cities["pingyuan"].order,
        (i32::from(before.order) + projection.order_delta).clamp(0, 100) as u8
    );
}

#[test]
fn official_posts_are_unique_within_faction() {
    let mut game = sample_game();

    appoint_official_post(&mut game, "liu_bei", "liu_bei", "taifu").unwrap();
    let liu_bei_loyalty = game.officers["liu_bei"].loyalty;
    appoint_official_post(&mut game, "liu_bei", "guan_yu", "taifu").unwrap();

    assert_eq!(game.officers["liu_bei"].office_id, None);
    assert_eq!(game.officers["guan_yu"].office_id.as_deref(), Some("taifu"));
    assert!(game.officers["liu_bei"].loyalty < liu_bei_loyalty);
}

#[test]
fn official_posts_reject_enemy_dead_and_unavailable_officers() {
    let mut game = sample_game();
    assert!(appoint_official_post(&mut game, "liu_bei", "cao_cao", "taifu").is_err());

    game.officers.get_mut("zhao_yun").unwrap().status = OfficerStatus::Dead;
    assert!(appoint_official_post(&mut game, "liu_bei", "zhao_yun", "taifu").is_err());

    game.officers.get_mut("jian_yong").unwrap().status = OfficerStatus::Unavailable;
    assert!(appoint_official_post(&mut game, "liu_bei", "jian_yong", "taifu").is_err());
}

#[test]
fn dismiss_official_post_restores_base_salary() {
    let mut game = sample_game();
    appoint_official_post(&mut game, "liu_bei", "liu_bei", "taifu").unwrap();
    assert!(game.officers["liu_bei"].office_id.is_some());

    dismiss_official_post(&mut game, "liu_bei", "liu_bei").unwrap();

    let officer = &game.officers["liu_bei"];
    assert_eq!(officer.office_id, None);
    assert_eq!(
        officer_monthly_salary(officer),
        officer_base_monthly_salary(officer)
    );
}

#[test]
fn appointing_official_post_increases_loyalty_by_rank() {
    let mut game = sample_game();
    game.officers.get_mut("jian_yong").unwrap().loyalty = 70;

    appoint_official_post(&mut game, "liu_bei", "jian_yong", "taifu").unwrap();

    assert_eq!(game.officers["jian_yong"].loyalty, 80);
}

#[test]
fn promotion_and_demotion_adjust_loyalty() {
    let mut game = sample_game();
    game.officers.get_mut("jian_yong").unwrap().loyalty = 70;

    appoint_official_post(&mut game, "liu_bei", "jian_yong", "zhubu").unwrap();
    let after_first_post = game.officers["jian_yong"].loyalty;
    appoint_official_post(&mut game, "liu_bei", "jian_yong", "taifu").unwrap();
    let after_promotion = game.officers["jian_yong"].loyalty;
    appoint_official_post(&mut game, "liu_bei", "jian_yong", "zhubu").unwrap();

    assert_eq!(after_first_post, 72);
    assert!(after_promotion > after_first_post);
    assert!(game.officers["jian_yong"].loyalty < after_promotion);
}

#[test]
fn dismissal_decreases_loyalty_by_rank_bonus() {
    let mut game = sample_game();
    game.officers.get_mut("jian_yong").unwrap().loyalty = 70;
    appoint_official_post(&mut game, "liu_bei", "jian_yong", "taifu").unwrap();
    let after_appointment = game.officers["jian_yong"].loyalty;

    dismiss_official_post(&mut game, "liu_bei", "jian_yong").unwrap();

    assert_eq!(after_appointment, 80);
    assert_eq!(game.officers["jian_yong"].loyalty, 70);
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
    game.cities.get_mut("pingyuan").unwrap().facilities.clear();
    game.cities.get_mut("xiapi").unwrap().facilities.clear();
    let before_source = game.cities["pingyuan"].troops;
    let before_target = game.cities["xiapi"].troops;
    let travel_months = game.travel_months_between("pingyuan", "xiapi").unwrap();
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
    assert_eq!(game.cities["xiapi"].troops, before_target);
    assert_eq!(game.officers["liu_bei"].city_id, None);
    assert_eq!(game.army_movements.len(), 1);

    for _ in 0..travel_months {
        resolve_command_batch(&mut game, Vec::new());
    }

    assert_eq!(game.cities["xiapi"].troops, before_target + 700);
    assert_eq!(game.officers["liu_bei"].city_id.as_deref(), Some("xiapi"));
    assert!(game.army_movements.is_empty());
}

#[test]
fn strong_expedition_can_capture_city() {
    let mut game = sample_game();
    {
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = 30_000;
        source.training = 100;
    }
    let travel_months = game.travel_months_between("xiapi", "xuchang").unwrap();
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

    assert_eq!(game.cities["xuchang"].faction_id, "cao_cao");
    assert_eq!(game.officers["zhang_fei"].city_id, None);
    assert_eq!(game.army_movements.len(), 1);

    for _ in 0..travel_months {
        resolve_command_batch(&mut game, Vec::new());
    }

    assert_eq!(game.cities["xuchang"].faction_id, "liu_bei");
    assert_eq!(
        game.officers["zhang_fei"].city_id.as_deref(),
        Some("xuchang")
    );
    assert!(game.army_movements.is_empty());
}

#[test]
fn adjacent_city_distance_controls_travel_months() {
    let game = sample_game();
    let short_distance = game.road_distance_li("xiapi", "xuchang").unwrap();
    let long_distance = game.road_distance_li("jiangxia", "jianye").unwrap();
    let short_months = game.travel_months_between("xiapi", "xuchang").unwrap();
    let long_months = game.travel_months_between("jiangxia", "jianye").unwrap();

    assert!(short_distance < long_distance);
    assert!(short_months < long_months);
    assert!(game.road_distance_li("pingyuan", "jianye").is_none());
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
fn save_load_preserves_official_post_assignment() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    appoint_official_post(&mut game, "liu_bei", "guan_yu", "taifu").unwrap();

    manager.save_slot("official_post", "官职", &game).unwrap();
    let loaded = manager.load_slot("official_post").unwrap();

    assert_eq!(
        loaded.officers["guan_yu"].office_id.as_deref(),
        Some("taifu")
    );
}

#[test]
fn invalid_save_can_be_discarded() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let game = sample_game();
    manager.save_slot("slot1", "旧档", &game).unwrap();

    let slot_path = manager.base_dir().join("slots").join("slot1.json");
    let mut save_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&slot_path).unwrap()).unwrap();
    save_json["version"] = serde_json::json!(SAVE_VERSION - 1);
    fs::write(
        &slot_path,
        serde_json::to_string_pretty(&save_json).unwrap(),
    )
    .unwrap();

    assert!(manager.load_slot("slot1").is_err());
    manager.delete_slot("slot1").unwrap();
    assert!(manager.list_slots().unwrap().is_empty());
}

#[test]
fn save_load_preserves_officer_profile_extensions_and_dynamic_loyalty() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    let officer = game.officers.get_mut("liu_bei").unwrap();
    officer.loyalty = 41;
    officer.gender = OfficerGender::Male;
    officer.profile = Some(OfficerProfile {
        id: "liu_bei".to_string(),
        name: "刘备".to_string(),
        courtesy_name: Some("玄德".to_string()),
        native_place: Some("涿郡涿县".to_string()),
        birth_year: Some(161),
        death_year: Some(223),
        gender: OfficerGender::Male,
        stats: officer.stats,
        tags: vec!["ruler".to_string()],
        confidence: SourceConfidence::High,
        biography: String::new(),
        relationships: Vec::new(),
        notes: String::new(),
    });
    let profile = officer.profile.as_mut().unwrap();
    profile.gender = OfficerGender::Male;
    profile.biography = "刘备详细生平摘要".to_string();
    profile.relationships = vec![OfficerRelationship {
        target_id: "guan_yu".to_string(),
        target_name: "关羽".to_string(),
        kind: OfficerRelationshipKind::SwornSibling,
        confidence: SourceConfidence::Medium,
        notes: "桃园结义".to_string(),
        source: "test".to_string(),
    }];

    manager
        .save_slot("profile_fields", "资料字段", &game)
        .unwrap();
    let loaded = manager.load_slot("profile_fields").unwrap();
    let loaded_officer = &loaded.officers["liu_bei"];
    let loaded_profile = loaded_officer.profile.as_ref().unwrap();

    assert_eq!(loaded_officer.loyalty, 41);
    assert_eq!(loaded_officer.gender, OfficerGender::Male);
    assert_eq!(loaded_profile.gender, OfficerGender::Male);
    assert_eq!(loaded_profile.biography, "刘备详细生平摘要");
    assert_eq!(loaded_profile.relationships.len(), 1);
    assert_eq!(
        loaded_profile.relationships[0].kind,
        OfficerRelationshipKind::SwornSibling
    );
}

#[test]
fn old_save_json_missing_profile_extensions_still_deserializes() {
    let mut game_json = serde_json::to_value(sample_game()).unwrap();
    let officers = game_json
        .get_mut("officers")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    let liu_bei = officers
        .get_mut("liu_bei")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    liu_bei.insert(
        "profile".to_string(),
        serde_json::json!({
            "id": "liu_bei",
            "name": "刘备",
            "courtesy_name": "玄德",
            "native_place": "涿郡涿县",
            "birth_year": 161,
            "death_year": 223,
            "stats": {
                "leadership": 76,
                "strength": 72,
                "intelligence": 78,
                "politics": 80,
                "charm": 99
            },
            "tags": ["ruler"],
            "confidence": "High",
            "notes": "旧存档资料"
        }),
    );
    liu_bei.remove("gender");
    let profile = liu_bei
        .get_mut("profile")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    profile.remove("gender");
    profile.remove("biography");
    profile.remove("relationships");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();
    let officer = &loaded.officers["liu_bei"];
    let profile = officer.profile.as_ref().unwrap();

    assert_eq!(officer.gender, OfficerGender::Male);
    assert_eq!(profile.gender, OfficerGender::Male);
    assert!(profile.biography.is_empty());
    assert!(profile.relationships.is_empty());
}

#[test]
fn old_save_json_missing_army_movements_still_deserializes() {
    let mut game_json = serde_json::to_value(sample_game()).unwrap();
    game_json.as_object_mut().unwrap().remove("army_movements");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();

    assert!(loaded.army_movements.is_empty());
}

#[test]
fn old_save_json_missing_technologies_still_deserializes() {
    let mut game_json = serde_json::to_value(sample_game()).unwrap();
    game_json.as_object_mut().unwrap().remove("technologies");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();

    assert!(loaded.technologies.is_empty());
}

#[test]
fn save_load_preserves_technology_state() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    start_research(&mut game, "liu_bei", TechnologyId::MilitiaDrill).unwrap();
    resolve_command_batch(&mut game, Vec::new());

    manager.save_slot("technology", "科技", &game).unwrap();
    let loaded = manager.load_slot("technology").unwrap();
    let state = faction_technology_state(&loaded, "liu_bei").unwrap();

    assert_eq!(state.active, Some(TechnologyId::MilitiaDrill));
    assert!(state.funded.contains(&TechnologyId::MilitiaDrill));
    assert_eq!(technology_progress(state, TechnologyId::MilitiaDrill), 1);
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

    assert!(
        report
            .entries
            .iter()
            .any(|entry| entry.message.contains("本月处理军令 0 条"))
    );
    assert!(
        report
            .entries
            .iter()
            .any(|entry| entry.message.contains("月度经济结算"))
    );
    assert!(
        report
            .entries
            .iter()
            .any(|entry| entry.message.contains("玩家控制"))
    );
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
