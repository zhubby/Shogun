use shogun::game::*;
use std::collections::BTreeMap;
use std::fs;

fn sample_game() -> GameState {
    let mut game = SqliteHistoricalCatalog::in_memory_from_seed()
        .unwrap()
        .build_game("ad200", "liu_bei")
        .unwrap();

    // Gameplay tests exercise command rules against a compact, deterministic
    // Liu Bei fixture. The fixture is now derived from the SQLite catalog, then
    // pinned to the city/officer layout expected by these rule-level tests.
    for city_id in ["pingyuan", "xiapi"] {
        let city = game.cities.get_mut(city_id).unwrap();
        city.faction_id = "liu_bei".to_string();
        city.gold = city.gold.max(1_200);
        city.food = city.food.max(1_800);
        city.materials = city.materials.max(1_200);
    }
    game.cities.get_mut("xuchang").unwrap().faction_id = "cao_cao".to_string();
    game.cities.get_mut("jianye").unwrap().faction_id = "sun_quan".to_string();
    game.cities.get_mut("jiangxia").unwrap().faction_id = "liu_biao".to_string();
    game.cities.get_mut("jianye").unwrap().position.x += 420.0;
    let xiapi_position = game.cities["xiapi"].position;
    game.cities.get_mut("xuchang").unwrap().position = MapPosition {
        x: xiapi_position.x + 40.0,
        y: xiapi_position.y,
    };

    let fixture_officers = [
        "liu_bei",
        "guan_yu",
        "zhao_yun",
        "jian_yong",
        "sun_qian",
        "zhang_fei",
        "chen_dao",
    ];
    for officer in game.officers.values_mut() {
        if officer.faction_id == "liu_bei" && !fixture_officers.contains(&officer.id.as_str()) {
            officer.faction_id = WILD_FACTION_ID.to_string();
            officer.city_id = Some("pingyuan".to_string());
            officer.status = OfficerStatus::Wild;
            officer.office_id = None;
        }
    }

    place_test_officer(&mut game, "liu_bei", "pingyuan", 90);
    place_test_officer(&mut game, "guan_yu", "pingyuan", 88);
    place_test_officer(&mut game, "zhao_yun", "pingyuan", 84);
    place_test_officer(&mut game, "jian_yong", "pingyuan", 76);
    place_test_officer(&mut game, "sun_qian", "pingyuan", 76);
    place_test_officer(&mut game, "zhang_fei", "xiapi", 86);
    place_test_officer(&mut game, "chen_dao", "xiapi", 80);
    game.cities.get_mut("pingyuan").unwrap().governor_id = Some("guan_yu".to_string());
    game.cities.get_mut("xiapi").unwrap().governor_id = Some("zhang_fei".to_string());

    add_test_road(&mut game, "pingyuan", "xiapi");
    add_test_road(&mut game, "xiapi", "xuchang");
    add_test_road(&mut game, "jiangxia", "jianye");
    game
}

fn place_test_officer(game: &mut GameState, officer_id: &str, city_id: &str, loyalty: u8) {
    let officer = game.officers.get_mut(officer_id).unwrap();
    officer.faction_id = "liu_bei".to_string();
    officer.city_id = Some(city_id.to_string());
    officer.status = OfficerStatus::Active;
    officer.loyalty = loyalty;
    officer.office_id = None;
}

fn add_test_road(game: &mut GameState, from: &str, to: &str) {
    if !game.are_adjacent(from, to) {
        game.roads.push(Road {
            from: from.to_string(),
            to: to.to_string(),
        });
    }
}

fn event_by_kind(game: &GameState, kind: GameEventKind) -> &GameEvent {
    game.events
        .iter()
        .find(|event| event.kind == kind)
        .expect("expected event kind")
}

fn command(city_id: &str, officer_id: &str, kind: CommandKind) -> Command {
    Command {
        issuer_faction_id: "liu_bei".to_string(),
        city_id: city_id.to_string(),
        officer_id: Some(officer_id.to_string()),
        kind,
    }
}

fn infantry(amount: u32) -> TroopPool {
    TroopPool::new(amount, 0, 0)
}

fn expedition(target_city_id: &str, officer_id: &str, kind: TroopKind, troops: u32) -> CommandKind {
    CommandKind::Expedition {
        target_city_id: target_city_id.to_string(),
        assignments: vec![ExpeditionAssignment::commander(
            officer_id.to_string(),
            kind,
            troops,
        )],
        food_supply: 1_000,
    }
}

fn resolve_until_arrival(game: &mut GameState, from: &str, to: &str) {
    let travel_months = game.travel_months_between(from, to).unwrap();
    for _ in 0..travel_months {
        resolve_command_batch(game, Vec::new());
    }
}

fn resolve_until_expedition_done(game: &mut GameState, target: &str, max_months: usize) {
    for _ in 0..max_months {
        if !game.army_movements.iter().any(|movement| {
            movement.kind == ArmyMovementKind::Expedition && movement.target_city_id == target
        }) {
            return;
        }
        resolve_command_batch(game, Vec::new());
    }
    let city = game.cities.get(target).unwrap();
    let movement = game
        .army_movements
        .iter()
        .find(|movement| {
            movement.kind == ArmyMovementKind::Expedition && movement.target_city_id == target
        })
        .unwrap();
    panic!(
        "expedition to {target} did not finish within {max_months} months; target troops {}, attacker troops {}, wounded {}, supply {}",
        city.troops.total(),
        movement.troops.total(),
        movement.wounded_troops.total(),
        movement.food_supply
    );
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
            expedition("jianye", "guan_yu", TroopKind::Infantry, 1000),
        ),
    );

    assert!(result.is_err());
}

#[test]
fn expedition_requires_one_commander_and_at_most_two_deputies() {
    let game = sample_game();
    let no_commander = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "guan_yu",
            CommandKind::Expedition {
                target_city_id: "ye".to_string(),
                assignments: vec![ExpeditionAssignment::deputy(
                    "zhao_yun".to_string(),
                    TroopKind::Infantry,
                    500,
                )],
                food_supply: 200,
            },
        ),
    );
    let too_many = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "guan_yu",
            CommandKind::Expedition {
                target_city_id: "ye".to_string(),
                assignments: vec![
                    ExpeditionAssignment::commander(
                        "guan_yu".to_string(),
                        TroopKind::Infantry,
                        500,
                    ),
                    ExpeditionAssignment::deputy("zhao_yun".to_string(), TroopKind::Infantry, 500),
                    ExpeditionAssignment::deputy("liu_bei".to_string(), TroopKind::Archers, 500),
                    ExpeditionAssignment::deputy("sun_qian".to_string(), TroopKind::Archers, 500),
                ],
                food_supply: 200,
            },
        ),
    );
    let duplicate = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "guan_yu",
            CommandKind::Expedition {
                target_city_id: "ye".to_string(),
                assignments: vec![
                    ExpeditionAssignment::commander(
                        "guan_yu".to_string(),
                        TroopKind::Infantry,
                        500,
                    ),
                    ExpeditionAssignment::deputy("guan_yu".to_string(), TroopKind::Archers, 500),
                ],
                food_supply: 200,
            },
        ),
    );

    assert!(no_commander.is_err());
    assert!(too_many.is_err());
    assert!(duplicate.is_err());
}

#[test]
fn expedition_rejects_officer_capacity_and_troop_kind_stock_overages() {
    let game = sample_game();
    let over_capacity = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "guan_yu",
            expedition("ye", "guan_yu", TroopKind::Infantry, 5_000),
        ),
    );
    let over_stock = validate_command_for_state(
        &game,
        &command(
            "pingyuan",
            "guan_yu",
            expedition("ye", "guan_yu", TroopKind::Cavalry, 3_300),
        ),
    );

    assert!(over_capacity.is_err());
    assert!(over_stock.is_err());
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
    let event = event_by_kind(&game, GameEventKind::CityDevelopment);
    assert_eq!(event.city_id.as_deref(), Some("pingyuan"));
    assert!(event.summary.contains("城镇核心"));
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
    assert!(game.events.iter().any(
        |event| event.kind == GameEventKind::CityDevelopment && event.summary.contains("市场")
    ));

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
fn medical_facility_increases_wounded_recovery_and_food_shortage_blocks_it() {
    let mut baseline = sample_game();
    let mut medical = sample_game();
    for game in [&mut baseline, &mut medical] {
        let city = game.cities.get_mut("xiapi").unwrap();
        city.troops = TroopPool::default();
        city.wounded_troops = TroopPool::new(1_000, 0, 0);
        city.food = 2_000;
        city.facilities.clear();
    }
    medical
        .cities
        .get_mut("xiapi")
        .unwrap()
        .facilities
        .push(CityFacility {
            kind: FacilityKind::Medical,
            level: 2,
        });

    resolve_command_batch(&mut baseline, Vec::new());
    resolve_command_batch(&mut medical, Vec::new());

    assert!(medical.cities["xiapi"].troops.total() > baseline.cities["xiapi"].troops.total());
    assert!(
        medical.cities["xiapi"].wounded_troops.total()
            < baseline.cities["xiapi"].wounded_troops.total()
    );

    let mut starving = sample_game();
    {
        let city = starving.cities.get_mut("xiapi").unwrap();
        city.troops = TroopPool::new(10_000, 0, 0);
        city.wounded_troops = TroopPool::new(1_000, 0, 0);
        city.food = 0;
        city.population = 0;
        city.agriculture = 0;
        city.level = 1;
        city.facilities.clear();
        city.facilities.push(CityFacility {
            kind: FacilityKind::Medical,
            level: 2,
        });
    }
    resolve_command_batch(&mut starving, Vec::new());

    assert_eq!(starving.cities["xiapi"].wounded_troops.total(), 1_000);
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
    assert!(city.troops.total() > before.troops.total());
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
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Recruit {
                kind: TroopKind::Infantry,
                amount: 500,
            },
        ),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert!(game.cities["xiapi"].troops.total() > before_troops.total());
    assert!(game.cities["xiapi"].gold < before_gold);
}

fn add_wild_officer(game: &mut GameState, officer_id: &str, city_id: &str, charm: u8) {
    let mut officer = game.officers["jian_yong"].clone();
    officer.id = officer_id.to_string();
    officer.name = "在野士人".to_string();
    officer.faction_id = WILD_FACTION_ID.to_string();
    officer.city_id = Some(city_id.to_string());
    officer.status = OfficerStatus::Wild;
    officer.birth_year = game.year - 30;
    officer.stats.charm = charm;
    officer.loyalty = 75;
    officer.office_id = None;
    game.officers.insert(officer.id.clone(), officer);
}

#[test]
fn wild_officer_cannot_execute_city_command_but_can_be_recruited() {
    let mut game = sample_game();
    add_wild_officer(&mut game, "wild_xun", "pingyuan", 60);

    let command_error =
        validate_command_for_state(&game, &command("pingyuan", "wild_xun", CommandKind::Train))
            .unwrap_err();
    assert_eq!(command_error.to_string(), "在野士人 不是己方武将");

    start_officer_recruitment(&mut game, "liu_bei", "pingyuan", "liu_bei", "wild_xun").unwrap();

    assert_eq!(game.officer_recruitments.len(), 1);
    assert_eq!(game.officer_recruitments[0].target_officer_id, "wild_xun");
}

#[test]
fn officer_recruitment_locks_recruiter_but_not_city_command_slot() {
    let mut game = sample_game();
    add_wild_officer(&mut game, "wild_xun", "pingyuan", 60);
    start_officer_recruitment(&mut game, "liu_bei", "pingyuan", "liu_bei", "wild_xun").unwrap();

    let same_officer_error = queue_player_command(
        &mut game,
        command("pingyuan", "liu_bei", CommandKind::Train),
    )
    .unwrap_err();
    assert_eq!(same_officer_error.to_string(), "武将 liu_bei 本月已经行动");

    queue_player_command(
        &mut game,
        command(
            "pingyuan",
            "guan_yu",
            CommandKind::Develop {
                focus: DevelopmentFocus::Agriculture,
            },
        ),
    )
    .unwrap();

    assert_eq!(game.pending_commands.len(), 1);
}

#[test]
fn officer_recruitment_progresses_across_months_and_can_succeed() {
    let mut game = sample_game();
    add_wild_officer(&mut game, "wild_xun", "pingyuan", 20);
    game.officers.get_mut("liu_bei").unwrap().stats.charm = 100;
    game.officers.get_mut("liu_bei").unwrap().stats.politics = 100;
    start_officer_recruitment(&mut game, "liu_bei", "pingyuan", "liu_bei", "wild_xun").unwrap();

    for _ in 0..6 {
        if game.officer_recruitments.is_empty() {
            break;
        }
        resolve_command_batch(&mut game, Vec::new());
    }

    assert!(game.officer_recruitments.is_empty());
    let officer = &game.officers["wild_xun"];
    assert_eq!(officer.status, OfficerStatus::Active);
    assert_eq!(officer.faction_id, "liu_bei");
    assert_eq!(officer.city_id.as_deref(), Some("pingyuan"));
    assert!((70..=95).contains(&officer.loyalty));
}

#[test]
fn officer_recruitment_can_end_with_refusal() {
    let mut game = sample_game();
    game.scenario_id = "refusal-seed".to_string();
    add_wild_officer(&mut game, "wild_xun", "pingyuan", 100);
    game.officers.get_mut("liu_bei").unwrap().stats.charm = 1;
    game.officers.get_mut("liu_bei").unwrap().stats.politics = 1;
    start_officer_recruitment(&mut game, "liu_bei", "pingyuan", "liu_bei", "wild_xun").unwrap();

    for _ in 0..12 {
        if game.officer_recruitments.is_empty() {
            break;
        }
        resolve_command_batch(&mut game, Vec::new());
    }

    assert!(game.officer_recruitments.is_empty());
    let officer = &game.officers["wild_xun"];
    assert_eq!(officer.status, OfficerStatus::Wild);
    assert_eq!(officer.faction_id, WILD_FACTION_ID);
}

#[test]
fn wild_officers_move_between_adjacent_cities_but_recruitment_target_stays_put() {
    let mut game = sample_game();
    game.scenario_id = "wild-move-seed".to_string();
    add_wild_officer(&mut game, "wild_mover", "pingyuan", 60);
    add_wild_officer(&mut game, "wild_target", "pingyuan", 60);
    start_officer_recruitment(&mut game, "liu_bei", "pingyuan", "liu_bei", "wild_target").unwrap();
    let target_before = game.officers["wild_target"].city_id.clone();

    for _ in 0..8 {
        resolve_command_batch(&mut game, Vec::new());
        if game.officers["wild_mover"].city_id.as_deref() != Some("pingyuan") {
            break;
        }
    }

    let moved_to = game.officers["wild_mover"].city_id.as_deref().unwrap();
    assert!(game.are_adjacent("pingyuan", moved_to));
    assert_eq!(game.officers["wild_target"].city_id, target_before);
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
                troops: infantry(700),
                officer_ids: Vec::new(),
            },
        ),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    let mut expected_source = before_source;
    expected_source.saturating_sub_pool(infantry(700));
    assert_eq!(game.cities["pingyuan"].troops, expected_source);
    assert_eq!(game.cities["xiapi"].troops, before_target);
    assert_eq!(game.officers["liu_bei"].city_id, None);
    assert_eq!(game.army_movements.len(), 1);

    for _ in 0..travel_months {
        resolve_command_batch(&mut game, Vec::new());
    }

    let mut expected_target = before_target;
    expected_target.add_pool(infantry(700));
    assert_eq!(game.cities["xiapi"].troops, expected_target);
    assert_eq!(game.officers["liu_bei"].city_id.as_deref(), Some("xiapi"));
    assert!(game.army_movements.is_empty());
}

#[test]
fn pending_expedition_reserves_deputy_officers() {
    let mut game = sample_game();
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![
                    ExpeditionAssignment::commander(
                        "zhang_fei".to_string(),
                        TroopKind::Infantry,
                        1_000,
                    ),
                    ExpeditionAssignment::deputy("chen_dao".to_string(), TroopKind::Archers, 800),
                ],
                food_supply: 200,
            },
        ),
    )
    .unwrap();

    let pending_officers = game.pending_officer_ids();
    assert!(pending_officers.contains("zhang_fei"));
    assert!(pending_officers.contains("chen_dao"));
}

#[test]
fn expedition_food_supply_is_required_and_spent_on_departure() {
    let mut game = sample_game();
    let before_food = game.cities["xiapi"].food;
    let without_supply = validate_command_for_state(
        &game,
        &command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![ExpeditionAssignment::commander(
                    "zhang_fei".to_string(),
                    TroopKind::Infantry,
                    500,
                )],
                food_supply: 0,
            },
        ),
    );
    assert!(without_supply.is_err());

    let too_much_supply = validate_command_for_state(
        &game,
        &command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![ExpeditionAssignment::commander(
                    "zhang_fei".to_string(),
                    TroopKind::Infantry,
                    500,
                )],
                food_supply: before_food as u32 + 1,
            },
        ),
    );
    assert!(too_much_supply.is_err());

    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![ExpeditionAssignment::commander(
                    "zhang_fei".to_string(),
                    TroopKind::Infantry,
                    500,
                )],
                food_supply: 500,
            },
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert!(game.cities["xiapi"].food < before_food);
    assert_eq!(game.army_movements[0].food_supply, 500);
}

#[test]
fn expedition_arrival_starts_siege_without_instant_capture() {
    let mut game = sample_game();
    {
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = TroopPool::new(12_000, 0, 0);
        source.training = 100;
        source.food = 4_000;
        let target = game.cities.get_mut("xuchang").unwrap();
        target.troops = TroopPool::new(4_000, 0, 0);
        target.training = 40;
        target.defense = 40;
        target.governor_id = None;
        target.facilities.clear();
    }
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            expedition("xuchang", "zhang_fei", TroopKind::Infantry, 3_000),
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);
    resolve_until_arrival(&mut game, "xiapi", "xuchang");

    assert_eq!(game.cities["xuchang"].faction_id, "cao_cao");
    assert!(game.army_movements.iter().any(|movement| {
        movement.kind == ArmyMovementKind::Expedition && movement.target_city_id == "xuchang"
    }));
    assert!(game.cities["xuchang"].troops.total() < 4_000);
    assert!(game.cities["xuchang"].wounded_troops.total() > 0);
}

#[test]
fn strong_expedition_can_capture_city() {
    let mut game = sample_game();
    {
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = TroopPool::new(20_000, 5_000, 5_000);
        source.training = 100;
        source.food = 10_000;
        let target = game.cities.get_mut("xuchang").unwrap();
        target.troops = TroopPool::new(4_000, 1_000, 1_000);
        target.training = 35;
        target.defense = 30;
        target.governor_id = None;
        target.facilities.clear();
    }
    appoint_official_post(&mut game, "liu_bei", "zhang_fei", "da_jiangjun").unwrap();
    appoint_official_post(&mut game, "liu_bei", "chen_dao", "taiwei").unwrap();
    let travel_months = game.travel_months_between("xiapi", "xuchang").unwrap();
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![
                    ExpeditionAssignment::commander(
                        "zhang_fei".to_string(),
                        TroopKind::Infantry,
                        15_000,
                    ),
                    ExpeditionAssignment::deputy("chen_dao".to_string(), TroopKind::Cavalry, 5_000),
                ],
                food_supply: 5_000,
            },
        ),
    )
    .unwrap();

    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    assert_eq!(game.cities["xuchang"].faction_id, "cao_cao");
    assert_eq!(game.officers["zhang_fei"].city_id, None);
    assert_eq!(game.army_movements.len(), 1);

    resolve_until_expedition_done(&mut game, "xuchang", travel_months as usize + 120);

    assert_eq!(game.cities["xuchang"].faction_id, "liu_bei");
    assert!(game.cities["xuchang"].wounded_troops.total() > 0);
    assert_eq!(
        game.officers["zhang_fei"].city_id.as_deref(),
        Some("xuchang")
    );
    assert_eq!(
        game.officers["chen_dao"].city_id.as_deref(),
        Some("xuchang")
    );
    assert!(game.army_movements.is_empty());
    assert!(
        game.events
            .iter()
            .any(|event| event.kind == GameEventKind::Battle
                && event.summary.contains("攻下")
                && event.city_id.as_deref() == Some("xuchang"))
    );
    assert!(
        game.events
            .iter()
            .any(|event| event.kind == GameEventKind::CityCaptured
                && event.city_id.as_deref() == Some("xuchang"))
    );
}

#[test]
fn troop_kind_matchups_change_battle_outcome() {
    let mut cavalry_game = sample_game();
    let mut infantry_game = sample_game();
    for game in [&mut cavalry_game, &mut infantry_game] {
        appoint_official_post(game, "liu_bei", "zhang_fei", "da_jiangjun").unwrap();
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = TroopPool::new(8_000, 8_000, 0);
        source.training = 100;
        source.facilities.clear();
        let target = game.cities.get_mut("xuchang").unwrap();
        target.troops = TroopPool::new(0, 0, 8_500);
        target.training = 40;
        target.defense = 20;
        target.order = 60;
        target.governor_id = None;
        target.facilities.clear();
    }

    queue_player_command(
        &mut cavalry_game,
        command(
            "xiapi",
            "zhang_fei",
            expedition("xuchang", "zhang_fei", TroopKind::Cavalry, 6_000),
        ),
    )
    .unwrap();
    let commands = cavalry_game.pending_commands.clone();
    resolve_command_batch(&mut cavalry_game, commands);
    resolve_until_expedition_done(&mut cavalry_game, "xuchang", 120);

    queue_player_command(
        &mut infantry_game,
        command(
            "xiapi",
            "zhang_fei",
            expedition("xuchang", "zhang_fei", TroopKind::Infantry, 6_000),
        ),
    )
    .unwrap();
    let commands = infantry_game.pending_commands.clone();
    resolve_command_batch(&mut infantry_game, commands);
    resolve_until_arrival(&mut infantry_game, "xiapi", "xuchang");
    for _ in 0..8 {
        resolve_command_batch(&mut infantry_game, Vec::new());
    }

    assert_eq!(cavalry_game.cities["xuchang"].faction_id, "liu_bei");
    assert_eq!(infantry_game.cities["xuchang"].faction_id, "cao_cao");
}

#[test]
fn defending_governor_can_prevent_borderline_capture() {
    let mut no_governor_game = sample_game();
    let mut governor_game = sample_game();
    for game in [&mut no_governor_game, &mut governor_game] {
        appoint_official_post(game, "liu_bei", "zhang_fei", "da_jiangjun").unwrap();
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = TroopPool::new(5_000, 0, 0);
        source.training = 100;
        source.facilities.clear();
        let target = game.cities.get_mut("xuchang").unwrap();
        target.troops = TroopPool::new(0, 7_500, 0);
        target.training = 50;
        target.defense = 50;
        target.order = 70;
        target.facilities.clear();
    }
    no_governor_game
        .cities
        .get_mut("xuchang")
        .unwrap()
        .governor_id = None;
    governor_game.cities.get_mut("xuchang").unwrap().governor_id = Some("cao_cao".to_string());
    {
        let governor = governor_game.officers.get_mut("cao_cao").unwrap();
        governor.stats.leadership = 100;
        governor.stats.strength = 100;
    }

    for game in [&mut no_governor_game, &mut governor_game] {
        queue_player_command(
            game,
            command(
                "xiapi",
                "zhang_fei",
                expedition("xuchang", "zhang_fei", TroopKind::Infantry, 3_200),
            ),
        )
        .unwrap();
        let commands = game.pending_commands.clone();
        resolve_command_batch(game, commands);
    }
    resolve_until_expedition_done(&mut no_governor_game, "xuchang", 120);
    resolve_until_arrival(&mut governor_game, "xiapi", "xuchang");
    for _ in 0..8 {
        resolve_command_batch(&mut governor_game, Vec::new());
    }

    assert_eq!(no_governor_game.cities["xuchang"].faction_id, "liu_bei");
    assert_eq!(governor_game.cities["xuchang"].faction_id, "cao_cao");
}

#[test]
fn faction_destroyed_event_is_recorded_once() {
    let mut game = sample_game();
    for city in game
        .cities
        .values_mut()
        .filter(|city| city.faction_id == "cao_cao" && city.id != "xuchang")
    {
        city.faction_id = "liu_bei".to_string();
    }
    {
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = TroopPool::new(20_000, 5_000, 5_000);
        source.training = 100;
        source.facilities.clear();
        let target = game.cities.get_mut("xuchang").unwrap();
        target.troops = TroopPool::new(100, 0, 0);
        target.training = 1;
        target.defense = 1;
        target.facilities.clear();
    }
    appoint_official_post(&mut game, "liu_bei", "zhang_fei", "da_jiangjun").unwrap();
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            expedition("xuchang", "zhang_fei", TroopKind::Infantry, 8_000),
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);
    resolve_until_expedition_done(&mut game, "xuchang", 120);
    resolve_command_batch(&mut game, Vec::new());

    let destroyed_events = game
        .events
        .iter()
        .filter(|event| {
            event.kind == GameEventKind::FactionDestroyed
                && event.faction_id.as_deref() == Some("cao_cao")
        })
        .count();
    assert_eq!(destroyed_events, 1);
}

#[test]
fn failed_expedition_returns_surviving_troop_pool_and_officers() {
    let mut game = sample_game();
    {
        let source = game.cities.get_mut("xiapi").unwrap();
        source.troops = TroopPool::new(2_000, 0, 0);
        source.training = 40;
        source.facilities.clear();
        let target = game.cities.get_mut("xuchang").unwrap();
        target.troops = TroopPool::new(12_000, 0, 0);
        target.training = 80;
        target.defense = 200;
        target.facilities.clear();
    }
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            expedition("xuchang", "zhang_fei", TroopKind::Infantry, 1_000),
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);
    assert_eq!(game.officers["zhang_fei"].city_id, None);

    resolve_until_expedition_done(&mut game, "xuchang", 60);

    assert_eq!(game.cities["xuchang"].faction_id, "cao_cao");
    assert_eq!(game.officers["zhang_fei"].city_id.as_deref(), Some("xiapi"));
    assert!(game.cities["xiapi"].troops.total() > 0);
    assert!(game.cities["xiapi"].troops.infantry > 0);
    assert!(game.cities["xiapi"].wounded_troops.total() > 0);
    assert!(
        game.events
            .iter()
            .any(|event| event.kind == GameEventKind::Battle
                && event.summary.contains("围攻")
                && event.city_id.as_deref() == Some("xuchang"))
    );
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
fn save_load_preserves_wounded_and_expedition_supply_state() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![ExpeditionAssignment::commander(
                    "zhang_fei".to_string(),
                    TroopKind::Infantry,
                    500,
                )],
                food_supply: 150,
            },
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);
    game.cities.get_mut("xiapi").unwrap().wounded_troops = TroopPool::new(300, 20, 10);
    game.army_movements[0].wounded_troops = TroopPool::new(40, 0, 0);
    game.army_movements[0].siege_started_turn = Some(game.turn);

    manager.save_slot("wounded", "伤兵", &game).unwrap();
    let loaded = manager.load_slot("wounded").unwrap();

    assert_eq!(
        loaded.cities["xiapi"].wounded_troops,
        TroopPool::new(300, 20, 10)
    );
    assert_eq!(loaded.army_movements[0].food_supply, 150);
    assert_eq!(
        loaded.army_movements[0].wounded_troops,
        TroopPool::new(40, 0, 0)
    );
    assert_eq!(loaded.army_movements[0].siege_started_turn, Some(game.turn));
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
fn old_save_json_missing_wounded_and_supply_fields_still_deserializes() {
    let mut game = sample_game();
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![ExpeditionAssignment::commander(
                    "zhang_fei".to_string(),
                    TroopKind::Infantry,
                    500,
                )],
                food_supply: 100,
            },
        ),
    )
    .unwrap();
    let commands = game.pending_commands.clone();
    resolve_command_batch(&mut game, commands);

    let mut game_json = serde_json::to_value(game).unwrap();
    let cities = game_json
        .get_mut("cities")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    cities
        .get_mut("xiapi")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .remove("wounded_troops");
    let movement = game_json
        .get_mut("army_movements")
        .and_then(serde_json::Value::as_array_mut)
        .unwrap()[0]
        .as_object_mut()
        .unwrap();
    movement.remove("food_supply");
    movement.remove("wounded_troops");
    movement.remove("siege_started_turn");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();

    assert_eq!(loaded.cities["xiapi"].wounded_troops.total(), 0);
    assert_eq!(loaded.army_movements[0].food_supply, 0);
    assert_eq!(loaded.army_movements[0].wounded_troops.total(), 0);
    assert_eq!(loaded.army_movements[0].siege_started_turn, None);
}

#[test]
fn old_pending_expedition_missing_food_supply_defaults_to_zero() {
    let mut game = sample_game();
    queue_player_command(
        &mut game,
        command(
            "xiapi",
            "zhang_fei",
            CommandKind::Expedition {
                target_city_id: "xuchang".to_string(),
                assignments: vec![ExpeditionAssignment::commander(
                    "zhang_fei".to_string(),
                    TroopKind::Infantry,
                    500,
                )],
                food_supply: 100,
            },
        ),
    )
    .unwrap();

    let mut game_json = serde_json::to_value(game).unwrap();
    let expedition_kind = game_json
        .get_mut("pending_commands")
        .and_then(serde_json::Value::as_array_mut)
        .unwrap()[0]
        .get_mut("kind")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .get_mut("Expedition")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap();
    expedition_kind.remove("food_supply");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();
    let CommandKind::Expedition { food_supply, .. } = &loaded.pending_commands[0].kind else {
        panic!("expected pending expedition");
    };
    assert_eq!(*food_supply, 0);
}

#[test]
fn old_save_json_missing_technologies_still_deserializes() {
    let mut game_json = serde_json::to_value(sample_game()).unwrap();
    game_json.as_object_mut().unwrap().remove("technologies");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();

    assert!(loaded.technologies.is_empty());
}

#[test]
fn old_save_json_missing_events_still_deserializes() {
    let mut game_json = serde_json::to_value(sample_game()).unwrap();
    let object = game_json.as_object_mut().unwrap();
    object.remove("events");
    object.remove("next_event_sequence");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();

    assert!(loaded.events.is_empty());
    assert_eq!(loaded.next_event_sequence, 0);
}

#[test]
fn old_save_json_missing_officer_recruitments_still_deserializes() {
    let mut game_json = serde_json::to_value(sample_game()).unwrap();
    let object = game_json.as_object_mut().unwrap();
    object.remove("officer_recruitments");
    object.remove("next_officer_recruitment_sequence");

    let loaded: GameState = serde_json::from_value(game_json).unwrap();

    assert!(loaded.officer_recruitments.is_empty());
    assert_eq!(loaded.next_officer_recruitment_sequence, 0);
}

#[test]
fn save_load_preserves_officer_recruitment_state() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    add_wild_officer(&mut game, "wild_xun", "pingyuan", 60);
    start_officer_recruitment(&mut game, "liu_bei", "pingyuan", "liu_bei", "wild_xun").unwrap();

    manager
        .save_slot("officer_recruitment", "登用", &game)
        .unwrap();
    let loaded = manager.load_slot("officer_recruitment").unwrap();

    assert_eq!(loaded.officer_recruitments.len(), 1);
    assert_eq!(loaded.officer_recruitments[0].target_officer_id, "wild_xun");
}

#[test]
fn save_load_preserves_event_state() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.food = 0;
        city.gold = 200;
    }
    resolve_command_batch(&mut game, Vec::new());
    assert_eq!(pending_event_count(&game), 1);

    manager.save_slot("events", "事件", &game).unwrap();
    let loaded = manager.load_slot("events").unwrap();

    assert_eq!(pending_event_count(&loaded), 1);
    assert_eq!(loaded.events[0].kind, GameEventKind::Famine);
    assert_eq!(loaded.next_event_sequence, 1);
}

#[test]
fn monthly_incident_can_trigger_without_starvation() {
    let mut game = sample_game();

    for _ in 0..3 {
        resolve_command_batch(&mut game, Vec::new());
    }

    assert!(
        game.events
            .iter()
            .any(|event| event.kind == GameEventKind::Famine)
    );
    assert_eq!(pending_event_count(&game), 1);
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
fn technology_completion_records_player_event() {
    let mut game = sample_game();
    start_research(&mut game, "liu_bei", TechnologyId::MilitiaDrill).unwrap();
    resolve_command_batch(&mut game, Vec::new());
    resolve_command_batch(&mut game, Vec::new());

    assert!(
        game.events
            .iter()
            .any(|event| event.kind == GameEventKind::TechnologyCompleted
                && event.summary.contains("乡勇操练"))
    );
}

#[test]
fn famine_relief_spends_gold_and_resolves_event() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.food = 0;
        city.gold = 200;
        city.order = 40;
    }
    resolve_command_batch(&mut game, Vec::new());
    let event_id = event_by_kind(&game, GameEventKind::Famine).id.clone();
    let before_gold = game.cities["pingyuan"].gold;
    let before_order = game.cities["pingyuan"].order;

    resolve_event_decision(&mut game, &event_id, "relief").unwrap();

    assert_eq!(game.cities["pingyuan"].gold, before_gold - 120);
    assert_eq!(game.cities["pingyuan"].order, before_order + 6);
    assert_eq!(pending_event_count(&game), 0);
    assert!(matches!(
        game.events
            .iter()
            .find(|event| event.id == event_id)
            .unwrap()
            .resolution,
        EventResolution::Resolved { .. }
    ));
}

#[test]
fn famine_expiry_applies_default_choice() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.food = 0;
        city.gold = 200;
        city.population = 50_000;
        city.order = 40;
    }
    resolve_command_batch(&mut game, Vec::new());
    let event_id = event_by_kind(&game, GameEventKind::Famine).id.clone();
    let before_population = game.cities["pingyuan"].population;
    let before_order = game.cities["pingyuan"].order;

    game.turn += 1;
    expire_due_event_decisions(&mut game);

    assert_eq!(game.cities["pingyuan"].population, before_population - 500);
    assert_eq!(game.cities["pingyuan"].order, before_order - 10);
    assert!(matches!(
        game.events
            .iter()
            .find(|event| event.id == event_id)
            .unwrap()
            .resolution,
        EventResolution::Expired { .. }
    ));
}

#[test]
fn famine_decision_cancels_if_city_is_no_longer_player_owned() {
    let mut game = sample_game();
    {
        let city = game.cities.get_mut("pingyuan").unwrap();
        city.food = 0;
        city.gold = 200;
    }
    resolve_command_batch(&mut game, Vec::new());
    let event_id = event_by_kind(&game, GameEventKind::Famine).id.clone();
    game.cities.get_mut("pingyuan").unwrap().faction_id = "cao_cao".to_string();

    resolve_event_decision(&mut game, &event_id, "relief").unwrap();

    assert!(matches!(
        game.events
            .iter()
            .find(|event| event.id == event_id)
            .unwrap()
            .resolution,
        EventResolution::Cancelled { .. }
    ));
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

#[test]
fn minor_officer_matures_at_eighteen_and_can_act() {
    let mut game = sample_game();
    game.year = 217;
    game.month = 12;
    let mut child = game.officers["liu_bei"].clone();
    child.id = "liu_an".to_string();
    child.name = "刘安".to_string();
    child.birth_year = 200;
    child.status = OfficerStatus::Minor;
    child.city_id = Some("pingyuan".to_string());
    game.officers.insert(child.id.clone(), child);

    assert_eq!(
        validate_command_for_state(&game, &command("pingyuan", "liu_an", CommandKind::Train))
            .unwrap_err()
            .to_string(),
        "刘安 当前不可行动"
    );

    resolve_command_batch(&mut game, Vec::new());

    assert_eq!(game.officers["liu_an"].status, OfficerStatus::Active);
    assert!(game.officers["liu_an"].is_adult_at(game.year));
}

#[test]
fn marriage_allows_multiple_spouses_and_rejects_invalid_pairs() {
    let mut game = sample_game();
    let mut lady_gan = game.officers["liu_bei"].clone();
    lady_gan.id = "lady_gan".to_string();
    lady_gan.name = "甘夫人".to_string();
    lady_gan.birth_year = 178;
    lady_gan.gender = OfficerGender::Female;
    lady_gan.city_id = Some("pingyuan".to_string());
    game.officers.insert(lady_gan.id.clone(), lady_gan);
    let mut lady_mi = game.officers["liu_bei"].clone();
    lady_mi.id = "lady_mi".to_string();
    lady_mi.name = "糜夫人".to_string();
    lady_mi.birth_year = 180;
    lady_mi.gender = OfficerGender::Female;
    lady_mi.city_id = Some("xiapi".to_string());
    game.officers.insert(lady_mi.id.clone(), lady_mi);

    marry_officers(&mut game, "liu_bei", "liu_bei", "lady_gan").unwrap();
    marry_officers(&mut game, "liu_bei", "liu_bei", "lady_mi").unwrap();

    assert_eq!(spouse_ids_for_officer(&game, "liu_bei").len(), 2);
    assert!(marry_officers(&mut game, "liu_bei", "guan_yu", "zhang_fei").is_err());
    assert!(marry_officers(&mut game, "liu_bei", "liu_bei", "lady_gan").is_err());

    game.family_relationships.push(FamilyRelationship {
        parent_id: "liu_bei".to_string(),
        child_id: "lady_mi".to_string(),
    });
    assert!(marry_officers(&mut game, "liu_bei", "liu_bei", "lady_mi").is_err());
}

#[test]
fn annual_birth_creates_minor_child_with_parent_relationships() {
    let mut game = sample_game();
    game.year = 200;
    game.month = 1;
    game.last_lifecycle_year = None;
    let mut mother = game.officers["liu_bei"].clone();
    mother.id = "lady_gan".to_string();
    mother.name = "甘夫人".to_string();
    mother.birth_year = 178;
    mother.gender = OfficerGender::Female;
    mother.city_id = Some("pingyuan".to_string());
    game.officers.insert(mother.id.clone(), mother);
    marry_officers(&mut game, "liu_bei", "liu_bei", "lady_gan").unwrap();
    let mut report = TurnReport::new(&game);

    apply_annual_lifecycle_with_config(
        &mut game,
        &mut report,
        &RuleBasedChildGenerator,
        LifecycleConfig {
            birth_chance_percent: 100,
            max_death_chance_percent: 0,
            ..LifecycleConfig::default()
        },
    );

    let child = game
        .officers
        .values()
        .find(|officer| officer.status == OfficerStatus::Minor)
        .unwrap();
    assert!(child.name.starts_with('刘'));
    assert_eq!(child.birth_year, 200);
    assert!(child.stats.leadership >= 1 && child.stats.leadership <= 100);
    assert!(child_ids_for_parent(&game, "liu_bei").contains(&child.id));
    assert!(child_ids_for_parent(&game, "lady_gan").contains(&child.id));
}

#[test]
fn ruler_death_records_succession_decision_and_choice_updates_ruler() {
    let mut game = sample_game();
    game.year = 200;
    set_default_heir(&mut game, "liu_bei", "guan_yu").unwrap();
    let mut report = TurnReport::new(&game);

    mark_officer_dead(&mut game, "liu_bei", &mut report, "测试").unwrap();

    assert_eq!(game.officers["liu_bei"].status, OfficerStatus::Dead);
    assert_eq!(game.factions["liu_bei"].ruler_id, "guan_yu");
    let event = event_by_kind(&game, GameEventKind::Succession);
    let choices = match &event.resolution {
        EventResolution::PendingDecision { choices, .. } => choices,
        _ => panic!("expected succession decision"),
    };
    assert!(choices.iter().any(|choice| choice.id == "zhang_fei"));
    let event_id = event.id.clone();

    resolve_event_decision(&mut game, &event_id, "zhang_fei").unwrap();

    assert_eq!(game.factions["liu_bei"].ruler_id, "zhang_fei");
}

#[test]
fn save_load_preserves_personnel_state() {
    let temp = tempfile::tempdir().unwrap();
    let manager = SaveManager::new(temp.path());
    let mut game = sample_game();
    game.factions.get_mut("liu_bei").unwrap().heir_id = Some("guan_yu".to_string());
    game.next_generated_officer_sequence = 3;
    game.last_lifecycle_year = Some(201);
    game.marriages.push(Marriage::new(
        "liu_bei".to_string(),
        "lady_gan".to_string(),
        200,
        1,
    ));
    game.family_relationships.push(FamilyRelationship {
        parent_id: "liu_bei".to_string(),
        child_id: "liu_an".to_string(),
    });

    manager.save_slot("personnel", "人事", &game).unwrap();
    let loaded = manager.load_slot("personnel").unwrap();

    assert_eq!(
        loaded.factions["liu_bei"].heir_id.as_deref(),
        Some("guan_yu")
    );
    assert_eq!(loaded.marriages.len(), 1);
    assert_eq!(loaded.family_relationships.len(), 1);
    assert_eq!(loaded.next_generated_officer_sequence, 3);
    assert_eq!(loaded.last_lifecycle_year, Some(201));
}

fn first_available_player_city(game: &GameState) -> Option<String> {
    game.cities
        .values()
        .find(|city| city.faction_id == game.player_faction_id)
        .map(|city| city.id.clone())
}
