use rusqlite::Connection;
use shogun::game::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SCHEMA_SQL: &str = include_str!("../assets/data/schema.sql");
const CORE_SEED_SQL: &str = include_str!("../assets/data/seeds/001_core.sql");

fn query_count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

fn query_user_version(conn: &Connection) -> i64 {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap()
}

fn build_legacy_unversioned_database(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.execute_batch(&legacy_v1_schema_sql()).unwrap();
    conn.execute_batch(CORE_SEED_SQL).unwrap();
    assert_eq!(query_user_version(&conn), 0);
}

#[test]
fn history_database_builds_with_integrity_counts_and_indexes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("database.sqlite");
    build_history_database(&path).unwrap();

    let conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    assert_eq!(query_user_version(&conn), HISTORY_DB_SCHEMA_VERSION as i64);

    let mut fk_check = conn.prepare("PRAGMA foreign_key_check").unwrap();
    let mut fk_rows = fk_check.query([]).unwrap();
    assert!(fk_rows.next().unwrap().is_none());

    let city_count = query_count(&conn, "cities");
    let officer_count = query_count(&conn, "officers");
    assert!((60..=90).contains(&city_count));
    assert_eq!(officer_count, 158);
    assert_eq!(query_count(&conn, "scenarios"), 4);
    assert!(query_count(&conn, "officer_life_events") >= officer_count);
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM officers WHERE id LIKE 'supplemental_%'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM officers WHERE gender NOT IN ('Male', 'Female')",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        0
    );
    assert_eq!(
        conn.query_row(
            "SELECT count(*) FROM officer_external_ids WHERE source = 'characters_of_the_three_kingdoms'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap(),
        81
    );
    assert!(query_count(&conn, "officer_relationships") >= 80);
    assert!(
        conn.query_row(
            "SELECT count(*) FROM officer_life_events WHERE loyalty IS NOT NULL",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap()
            > 0
    );

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
        "idx_officer_relationships_target",
        "idx_officer_relationships_kind",
        "idx_officer_external_ids_source",
        "idx_cities_province",
    ] {
        assert!(indexes.contains(index), "missing index {index}");
    }
}

#[test]
fn open_or_create_creates_database_and_runs_initial_migration() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("nested").join("database.sqlite");

    let catalog = SqliteHistoricalCatalog::open_or_create(&path).unwrap();

    assert!(path.exists());
    assert_eq!(catalog.scenarios().unwrap().len(), 4);
    let conn = Connection::open(&path).unwrap();
    assert_eq!(query_user_version(&conn), HISTORY_DB_SCHEMA_VERSION as i64);
}

#[test]
fn open_or_create_adopts_legacy_unversioned_database() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("database.sqlite");
    build_legacy_unversioned_database(&path);

    let catalog = SqliteHistoricalCatalog::open_or_create(&path).unwrap();

    assert_eq!(catalog.scenarios().unwrap().len(), 4);
    let conn = Connection::open(&path).unwrap();
    assert_eq!(query_user_version(&conn), HISTORY_DB_SCHEMA_VERSION as i64);
    assert_eq!(query_count(&conn, "officer_external_ids"), 86);
    assert_eq!(query_count(&conn, "officer_relationships"), 85);
}

#[test]
fn open_or_create_migrates_old_v2_unknown_gender_to_two_value_schema() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("database.sqlite");
    build_legacy_v2_database_with_unknown_gender(&path);

    SqliteHistoricalCatalog::open_or_create(&path).unwrap();

    let conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    assert_eq!(query_user_version(&conn), HISTORY_DB_SCHEMA_VERSION as i64);
    assert_eq!(
        conn.query_row(
            "SELECT gender FROM officers WHERE id = 'liu_bei'",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap(),
        "Male"
    );
    assert!(conn
        .execute(
            "UPDATE officers SET gender = 'Unknown' WHERE id = 'liu_bei'",
            [],
        )
        .is_err());
}

#[test]
fn open_or_create_rejects_newer_database_versions() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("database.sqlite");
    let conn = Connection::open(&path).unwrap();
    conn.pragma_update(None, "user_version", HISTORY_DB_SCHEMA_VERSION + 1)
        .unwrap();
    drop(conn);

    let error = match SqliteHistoricalCatalog::open_or_create(&path) {
        Ok(_) => panic!("newer database should be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("高于当前程序支持版本"));
}

#[test]
fn officer_profiles_include_gender_biography_sources_and_relationships() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    let liu_bei = catalog.officer_profile("liu_bei").unwrap().unwrap();
    assert_eq!(liu_bei.gender, OfficerGender::Male);
    assert!(liu_bei.biography.contains("刘备"));
    assert!(liu_bei
        .relationships
        .iter()
        .any(
            |relationship| relationship.kind == OfficerRelationshipKind::SwornSibling
                && relationship.target_id == "guan_yu"
        ));
    assert!(liu_bei
        .relationships
        .iter()
        .any(
            |relationship| relationship.kind == OfficerRelationshipKind::Spouse
                && relationship.target_id == "lady_gan"
        ));

    let lady_gan = catalog.officer_profile("lady_gan").unwrap().unwrap();
    assert_eq!(lady_gan.gender, OfficerGender::Female);
    assert!(lady_gan.biography.contains("刘禅生母"));
}

#[test]
fn historical_relationships_include_curated_family_and_enemy_links() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    for (source, target, kind) in [
        ("sun_jian", "sun_ce", OfficerRelationshipKind::ParentChild),
        ("sun_jian", "sun_quan", OfficerRelationshipKind::ParentChild),
        ("cao_cao", "cao_pi", OfficerRelationshipKind::ParentChild),
        ("liu_bei", "lady_gan", OfficerRelationshipKind::Spouse),
        ("liu_bei", "lady_mi", OfficerRelationshipKind::Spouse),
        ("liu_bei", "lady_sun", OfficerRelationshipKind::Spouse),
        ("cao_cao", "lu_bu", OfficerRelationshipKind::Enemy),
        ("guan_yu", "lu_meng", OfficerRelationshipKind::Enemy),
        ("zhuge_liang", "sima_yi", OfficerRelationshipKind::Enemy),
    ] {
        let relationships = catalog.officer_relationships(source).unwrap();
        assert!(
            relationships
                .iter()
                .any(|relationship| relationship.target_id == target && relationship.kind == kind),
            "missing {source} -> {target} {kind:?}"
        );
    }
}

#[test]
fn database_path_helpers_are_testable_without_user_data_dir() {
    let temp = tempfile::tempdir().unwrap();

    assert_eq!(
        SqliteHistoricalCatalog::database_path_in(temp.path()),
        temp.path().join("database.sqlite")
    );
    assert_eq!(
        SqliteHistoricalCatalog::fallback_data_dir(),
        PathBuf::from(".shogun_data")
    );
    assert_eq!(
        SqliteHistoricalCatalog::database_path_in(SqliteHistoricalCatalog::fallback_data_dir()),
        PathBuf::from(".shogun_data").join("database.sqlite")
    );
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

#[test]
fn life_events_with_loyalty_apply_initial_loyalty() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    let mut game = catalog.build_game("ad208", "liu_bei").unwrap();
    assert!(!game.officers.contains_key("ctk_5f20_6e29_32"));
    game.year = 210;
    game.month = 12;

    resolve_command_batch_with_history(&mut game, Vec::new(), &catalog);

    let officer = game.officers.get("ctk_5f20_6e29_32").unwrap();
    assert_eq!(officer.name, "张温");
    assert_eq!(officer.loyalty, 76);
}

fn legacy_v1_schema_sql() -> String {
    SCHEMA_SQL
        .replace(
            "    gender TEXT NOT NULL DEFAULT 'Male' CHECK (gender IN ('Male', 'Female')),\n",
            "",
        )
        .replace("    biography TEXT NOT NULL DEFAULT '',\n", "")
        .replace(
            "    loyalty INTEGER CHECK (loyalty BETWEEN 1 AND 100),\n",
            "",
        )
        .replace(
            r#"CREATE TABLE officer_external_ids (
    officer_id TEXT NOT NULL REFERENCES officers(id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    external_id TEXT NOT NULL,
    source_url TEXT NOT NULL DEFAULT '',
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (officer_id, source, external_id)
);

"#,
            "",
        )
        .replace(
            r#"CREATE TABLE officer_relationships (
    source_officer_id TEXT NOT NULL REFERENCES officers(id) ON DELETE CASCADE,
    target_officer_id TEXT NOT NULL REFERENCES officers(id) ON DELETE CASCADE,
    relationship_kind TEXT NOT NULL CHECK (relationship_kind IN (
        'RulerSubject',
        'ParentChild',
        'AdoptiveParentChild',
        'Spouse',
        'Sibling',
        'SwornSibling',
        'Enemy'
    )),
    confidence TEXT NOT NULL CHECK (confidence IN ('High', 'Medium', 'Low')),
    notes TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (source_officer_id, target_officer_id, relationship_kind),
    CHECK (source_officer_id <> target_officer_id)
);

"#,
            "",
        )
        .replace(
            "CREATE INDEX idx_officer_relationships_target ON officer_relationships(target_officer_id);\n",
            "",
        )
        .replace(
            "CREATE INDEX idx_officer_relationships_kind ON officer_relationships(relationship_kind);\n",
            "",
        )
        .replace(
            "CREATE INDEX idx_officer_external_ids_source ON officer_external_ids(source, external_id);\n",
            "",
        )
}

fn build_legacy_v2_database_with_unknown_gender(path: &Path) {
    let conn = Connection::open(path).unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.execute_batch(&legacy_v2_unknown_gender_schema_sql())
        .unwrap();
    conn.execute_batch(CORE_SEED_SQL).unwrap();
    conn.execute_batch(include_str!(
        "../assets/data/seeds/002_three_kingdoms_import.sql"
    ))
    .unwrap();
    conn.execute_batch(include_str!(
        "../assets/data/seeds/003_officer_relationships.sql"
    ))
    .unwrap();
    conn.execute(
        "UPDATE officers SET gender = 'Unknown' WHERE id = 'liu_bei'",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 2).unwrap();
}

fn legacy_v2_unknown_gender_schema_sql() -> String {
    SCHEMA_SQL.replace(
        "gender TEXT NOT NULL DEFAULT 'Male' CHECK (gender IN ('Male', 'Female'))",
        "gender TEXT NOT NULL DEFAULT 'Unknown' CHECK (gender IN ('Male', 'Female', 'Unknown'))",
    )
}
