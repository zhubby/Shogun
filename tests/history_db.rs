use rusqlite::Connection;
use shogun::game::*;
use std::collections::BTreeSet;

fn query_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

#[test]
fn history_database_builds_with_integrity_counts_and_indexes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("history.sqlite");
    build_history_database(&path).unwrap();

    let conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();

    let mut fk_check = conn.prepare("PRAGMA foreign_key_check").unwrap();
    let mut fk_rows = fk_check.query([]).unwrap();
    assert!(fk_rows.next().unwrap().is_none());

    let city_count = query_count(&conn, "cities");
    let officer_count = query_count(&conn, "officers");
    assert!((60..=90).contains(&city_count));
    assert!((500..=700).contains(&officer_count));
    assert_eq!(query_count(&conn, "scenarios"), 4);
    assert!(query_count(&conn, "officer_life_events") >= officer_count);

    let indexes: BTreeSet<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'index'")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    for index in [
        "idx_roads_from",
        "idx_roads_to",
        "idx_scenario_city_states_scenario",
        "idx_scenario_city_states_faction",
        "idx_scenario_faction_states_scenario",
        "idx_officer_life_events_date",
        "idx_officer_life_events_officer",
        "idx_officers_name",
        "idx_cities_province",
    ] {
        assert!(indexes.contains(index), "missing index {index}");
    }
}

#[test]
fn fixed_scenarios_build_with_valid_selectable_factions_and_governors() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();
    let scenarios = catalog.scenarios().unwrap();
    let scenario_ids: Vec<_> = scenarios
        .iter()
        .map(|scenario| scenario.id.as_str())
        .collect();
    assert_eq!(scenario_ids, ["ad190", "ad200", "ad208", "ad220"]);

    for scenario in scenarios {
        let factions = catalog.selectable_factions(&scenario.id).unwrap();
        let selectable: Vec<_> = factions
            .iter()
            .filter(|faction| faction.selectable)
            .collect();
        assert!(
            !selectable.is_empty(),
            "{} should have selectable factions",
            scenario.id
        );

        for faction in selectable {
            let game = catalog.build_game(&scenario.id, &faction.id).unwrap();
            assert_eq!(game.cities.len(), 70);
            assert!(
                game.faction_alive(&faction.id),
                "{} selectable faction {} has no city",
                scenario.id,
                faction.id
            );
            assert!(game
                .cities
                .values()
                .all(|city| game.factions.contains_key(&city.faction_id)));
            assert!(game
                .officers
                .values()
                .all(|officer| officer.profile.is_some()));

            for city in game.cities.values() {
                if let Some(governor_id) = &city.governor_id {
                    let governor = game.officers.get(governor_id).unwrap();
                    assert!(governor.is_active());
                    assert_eq!(governor.faction_id, city.faction_id);
                    assert_eq!(governor.city_id.as_deref(), Some(city.id.as_str()));
                }
            }
        }
    }
}

#[test]
fn life_events_apply_appearances_deaths_and_do_not_repeat() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    let mut appear_game = catalog.build_game("ad208", "liu_bei").unwrap();
    assert!(!appear_game.officers.contains_key("jiang_wei"));
    appear_game.year = 219;
    appear_game.month = 12;

    resolve_command_batch_with_history(&mut appear_game, Vec::new(), &catalog);

    let jiang_wei = appear_game.officers.get("jiang_wei").unwrap();
    assert!(jiang_wei.is_active());
    assert_eq!(jiang_wei.faction_id, "liu_bei");
    assert!(appear_game.applied_event_ids.contains("start_jiang_wei"));

    let mut death_game = catalog.build_game("ad200", "sun_quan").unwrap();
    assert!(death_game.officers["sun_ce"].is_active());
    death_game.year = 200;
    death_game.month = 11;

    let first_report = resolve_command_batch_with_history(&mut death_game, Vec::new(), &catalog);

    assert!(death_game.applied_event_ids.contains("death_sun_ce"));
    assert_eq!(death_game.officers["sun_ce"].status, OfficerStatus::Dead);
    assert!(first_report
        .entries
        .iter()
        .any(|entry| entry.message.contains("孙策")));

    let applied_count = death_game.applied_event_ids.len();
    death_game.year = 200;
    death_game.month = 11;
    let second_report = resolve_command_batch_with_history(&mut death_game, Vec::new(), &catalog);

    assert_eq!(death_game.applied_event_ids.len(), applied_count);
    assert!(!second_report
        .entries
        .iter()
        .any(|entry| entry.message.contains("孙策")));
}
