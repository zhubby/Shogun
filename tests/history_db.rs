use shogun::game::*;
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn all_facility_kinds_have_valid_specs() {
    assert!(ALL_FACILITY_KINDS.contains(&FacilityKind::Medical));
    for kind in ALL_FACILITY_KINDS {
        let cost = facility_upgrade_cost(kind, 1);
        assert!(cost.gold > 0);
        assert!(cost.food >= 0);
        assert!(cost.materials > 0);
        let facility = CityFacility { kind, level: 1 };
        let json = serde_json::to_string(&facility).unwrap();
        let loaded: CityFacility = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, facility);
    }
}

async fn open_pool(path: &Path) -> sqlx::SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap()
}

async fn query_count(pool: &sqlx::SqlitePool, table: &str) -> i64 {
    let sql = format!("SELECT count(*) AS count FROM {table}");
    query_count_sql(pool, &sql).await
}

async fn query_count_sql(pool: &sqlx::SqlitePool, sql: &str) -> i64 {
    sqlx::query(sql).fetch_one(pool).await.unwrap().get("count")
}

async fn applied_sqlx_migration_versions(pool: &sqlx::SqlitePool) -> Vec<i64> {
    sqlx::query("SELECT version FROM _sqlx_migrations ORDER BY version")
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get("version"))
        .collect()
}

#[test]
fn history_database_builds_with_integrity_counts_and_indexes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("database.sqlite");
    build_history_database(&path).unwrap();

    runtime().block_on(async {
        let pool = open_pool(&path).await;
        assert_eq!(
            applied_sqlx_migration_versions(&pool).await,
            [1, 2, 3, 4, 5, 6, 7, 8, 9]
        );

        let fk_rows = sqlx::query("PRAGMA foreign_key_check")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert!(fk_rows.is_empty());

        let city_count = query_count(&pool, "cities").await;
        let officer_count = query_count(&pool, "officers").await;
        assert!((60..=90).contains(&city_count));
        assert_eq!(officer_count, 432);
        assert_eq!(
            query_count_sql(&pool, "SELECT count(*) AS count FROM officers WHERE gender = 'Male'")
                .await,
            362
        );
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count FROM officers WHERE gender = 'Female'",
            )
            .await,
            70
        );
        assert_eq!(query_count(&pool, "scenarios").await, 5);
        assert!(query_count(&pool, "officer_life_events").await >= officer_count);
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officers
                 WHERE id = 'zhao_yun_early'
                    OR id LIKE 'supplemental_%'
                    OR name = '名不详'
                    OR notes LIKE '%低置信度补充人物%'",
            )
            .await,
            0
        );
        assert_eq!(
            sqlx::query("SELECT count(*) AS count FROM officers WHERE gender NOT IN ('Male', 'Female')")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get::<i64, _>("count"),
            0
        );
        assert_eq!(
            sqlx::query(
                "SELECT count(*) AS count FROM officer_external_ids WHERE source = 'characters_of_the_three_kingdoms'",
            )
            .fetch_one(&pool)
            .await
            .unwrap()
            .get::<i64, _>("count"),
            81
        );
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officer_tags
                 WHERE tag_id = 'batch:expansion_003'",
            )
            .await,
            263
        );
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officers o
                 JOIN officer_tags t ON t.officer_id = o.id
                 WHERE t.tag_id = 'batch:expansion_003'
                   AND biography LIKE '%收录用于扩充三国时期群雄、宗族与幕府网络%'",
            )
            .await,
            0
        );
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officers
                 WHERE id = 'liu_bian'
                   AND biography LIKE '%弘农王%'",
            )
            .await,
            1
        );
        let zhang_daoling = sqlx::query(
            "SELECT name, biography
             FROM officers
             WHERE id = 'ctk_5f20_9053_9675'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(zhang_daoling.get::<String, _>("name"), "张道陵");
        assert!(zhang_daoling
            .get::<String, _>("biography")
            .contains("初名张陵"));
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officers o
                 JOIN officer_tags t ON t.officer_id = o.id
                 WHERE t.tag_id = 'batch:expansion_003'
                   AND NOT EXISTS (
                       SELECT 1
                       FROM officer_external_ids e
                       WHERE e.officer_id = o.id
                   )",
            )
            .await,
            0
        );
        let officer_columns: BTreeSet<String> = sqlx::query("PRAGMA table_info(officers)")
            .fetch_all(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get("name"))
            .collect();
        assert!(!officer_columns.contains("tags"));
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officer_tags
                 WHERE tag_id = 'female'
                    OR tag_id LIKE 'faction:%'
                    OR tag_id IN ('蜀', '魏', '东吴', '西晋')",
            )
            .await,
            0
        );
        assert!(query_count(&pool, "officer_tag_definitions").await >= 30);
        assert_eq!(query_count(&pool, "officer_tag_aliases").await, 54);
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officers o
                 WHERE o.gender = 'Female'
                   AND NOT EXISTS (
                       SELECT 1
                       FROM officer_relationships r
                       WHERE r.source_officer_id = o.id
                          OR r.target_officer_id = o.id
                   )",
            )
            .await,
            0
        );
        assert_eq!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officer_relationships r
                 WHERE r.relationship_kind IN ('Spouse', 'Sibling', 'SwornSibling', 'Enemy')
                   AND NOT EXISTS (
                       SELECT 1
                       FROM officer_relationships rev
                       WHERE rev.source_officer_id = r.target_officer_id
                         AND rev.target_officer_id = r.source_officer_id
                         AND rev.relationship_kind = r.relationship_kind
                   )",
            )
            .await,
            0
        );
        assert!(query_count(&pool, "officer_relationships").await >= 300);
        assert!(
            query_count_sql(
                &pool,
                "SELECT count(*) AS count
                 FROM officer_relationships
                 WHERE relationship_kind = 'Spouse'",
            )
            .await
                >= 100
        );
        assert!(
            sqlx::query("SELECT count(*) AS count FROM officer_life_events WHERE loyalty IS NOT NULL")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get::<i64, _>("count")
                > 0
        );

        let indexes: BTreeSet<String> = sqlx::query("SELECT name FROM sqlite_master WHERE type = 'index'")
            .fetch_all(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get("name"))
            .collect();
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
            "idx_officer_tags_tag",
            "idx_officer_tags_officer",
            "idx_officer_tag_definitions_category",
            "idx_cities_province",
        ] {
            assert!(indexes.contains(index), "missing index {index}");
        }
        pool.close().await;
    });
}

#[test]
fn open_or_create_creates_database_and_runs_initial_migration() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("nested").join("database.sqlite");

    let catalog = SqliteHistoricalCatalog::open_or_create(&path).unwrap();

    assert!(path.exists());
    assert_eq!(catalog.scenarios().unwrap().len(), 5);
    runtime().block_on(async {
        let pool = open_pool(&path).await;
        assert_eq!(
            applied_sqlx_migration_versions(&pool).await,
            [1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        pool.close().await;
    });
}

#[test]
fn historical_road_network_matches_map_revisions() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();
    let game = catalog.build_game("ad200", "liu_bei").unwrap();

    assert!(game.cities["pingyuan"].position.x > game.cities["shangdang"].position.x);
    assert!(game.are_adjacent("shangdang", "jinyang"));
    assert!(game.are_adjacent("pingyuan", "shangdang"));
    assert!(game.are_adjacent("pingyuan", "ye"));
    assert!(!game.are_adjacent("pingyuan", "ganling"));
    assert!(!game.are_adjacent("zhongshan", "ganling"));
    assert!(game.are_adjacent("zhongshan", "ye"));
    assert!(game.are_adjacent("yingchuan", "luoyang"));
    assert!(game.are_adjacent("yuzhang", "danyang"));
    assert!(!game.are_adjacent("yuzhang", "jianye"));
    assert!(game.are_adjacent("lingling", "cangwu"));
    assert!(game.are_adjacent("guiyang", "cangwu"));
    assert!(game.are_adjacent("yuzhang", "changsha"));
}

#[test]
fn officer_profiles_include_gender_biography_sources_and_relationships() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    let liu_bei = catalog.officer_profile("liu_bei").unwrap().unwrap();
    assert_eq!(liu_bei.gender, OfficerGender::Male);
    assert!(liu_bei.biography.contains("刘备"));
    assert!(liu_bei.relationships.iter().any(|relationship| {
        relationship.kind == OfficerRelationshipKind::SwornSibling
            && relationship.target_id == "guan_yu"
    }));
    assert!(liu_bei.relationships.iter().any(|relationship| {
        relationship.kind == OfficerRelationshipKind::Spouse && relationship.target_id == "lady_gan"
    }));

    let lady_gan = catalog.officer_profile("lady_gan").unwrap().unwrap();
    assert_eq!(lady_gan.gender, OfficerGender::Female);
    assert!(lady_gan.biography.contains("刘禅生母"));
}

#[test]
fn officer_profiles_bulk_load_tags_and_relationships() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    let profiles = catalog.officer_profiles().unwrap();
    let liu_bei = profiles
        .iter()
        .find(|profile| profile.id == "liu_bei")
        .unwrap();

    assert!(liu_bei.tags.iter().any(|tag| tag == "role:ruler"));
    assert!(liu_bei.relationships.iter().any(|relationship| {
        relationship.kind == OfficerRelationshipKind::SwornSibling
            && relationship.target_id == "guan_yu"
    }));
}

#[test]
fn update_officer_profile_persists_basic_fields() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();
    let profile = catalog.officer_profile("liu_bei").unwrap().unwrap();
    let mut update = OfficerProfileUpdate::from_profile(&profile);
    update.name = "刘玄德".to_string();
    update.courtesy_name = Some("玄德公".to_string());
    update.native_place = None;
    update.birth_year = Some(160);
    update.death_year = None;
    update.gender = OfficerGender::Male;
    update.stats = OfficerStats {
        leadership: 88,
        strength: 77,
        intelligence: 81,
        politics: 83,
        charm: 99,
    };
    update.tags = vec![
        "role:ruler".to_string(),
        "source:manual_curated".to_string(),
    ];
    update.confidence = SourceConfidence::Medium;
    update.biography = "编辑后的刘备生平摘要".to_string();
    update.notes = "编辑测试备注".to_string();

    let saved = catalog.update_officer_profile("liu_bei", &update).unwrap();
    let loaded = catalog.officer_profile("liu_bei").unwrap().unwrap();

    assert_eq!(saved, loaded);
    assert_eq!(loaded.name, "刘玄德");
    assert_eq!(loaded.courtesy_name.as_deref(), Some("玄德公"));
    assert_eq!(loaded.native_place, None);
    assert_eq!(loaded.birth_year, Some(160));
    assert_eq!(loaded.death_year, None);
    assert_eq!(loaded.stats.leadership, 88);
    assert_eq!(loaded.tags, ["role:ruler", "source:manual_curated"]);
    assert_eq!(loaded.confidence, SourceConfidence::Medium);
    assert_eq!(loaded.biography, "编辑后的刘备生平摘要");
    assert_eq!(loaded.notes, "编辑测试备注");
}

#[test]
fn update_officer_profile_rejects_invalid_values() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();
    let profile = catalog.officer_profile("liu_bei").unwrap().unwrap();

    let mut blank_name = OfficerProfileUpdate::from_profile(&profile);
    blank_name.name = "   ".to_string();
    assert!(
        catalog
            .update_officer_profile("liu_bei", &blank_name)
            .is_err()
    );

    let mut zero_stat = OfficerProfileUpdate::from_profile(&profile);
    zero_stat.stats.leadership = 0;
    assert!(
        catalog
            .update_officer_profile("liu_bei", &zero_stat)
            .is_err()
    );

    let mut unknown_tag = OfficerProfileUpdate::from_profile(&profile);
    unknown_tag.tags = vec!["not_a_real_tag".to_string()];
    let error = catalog
        .update_officer_profile("liu_bei", &unknown_tag)
        .unwrap_err()
        .to_string();
    assert!(error.contains("未知武将标签"));
}

#[test]
fn update_officer_profile_keeps_ids_relationships_sources_and_events() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("database.sqlite");
    build_history_database(&path).unwrap();

    let before = runtime().block_on(async {
        let pool = open_pool(&path).await;
        let counts = (
            query_count(&pool, "officer_external_ids").await,
            query_count(&pool, "officer_relationships").await,
            query_count(&pool, "officer_life_events").await,
        );
        pool.close().await;
        counts
    });

    let catalog = SqliteHistoricalCatalog::open_or_create(&path).unwrap();
    let profile = catalog.officer_profile("liu_bei").unwrap().unwrap();
    let mut update = OfficerProfileUpdate::from_profile(&profile);
    update.name = "刘备编辑".to_string();
    catalog.update_officer_profile("liu_bei", &update).unwrap();
    drop(catalog);

    runtime().block_on(async {
        let pool = open_pool(&path).await;
        let after = (
            query_count(&pool, "officer_external_ids").await,
            query_count(&pool, "officer_relationships").await,
            query_count(&pool, "officer_life_events").await,
        );
        assert_eq!(after, before);
        assert!(
            sqlx::query("UPDATE officers SET gender = 'Unknown' WHERE id = 'liu_bei'")
                .execute(&pool)
                .await
                .is_err()
        );
        assert_eq!(
            sqlx::query("SELECT id FROM officers WHERE name = '刘备编辑'")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get::<String, _>("id"),
            "liu_bei"
        );
        pool.close().await;
    });
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
    assert_eq!(scenario_ids, ["ad180", "ad190", "ad200", "ad208", "ad220"]);

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
            assert!(
                game.cities
                    .values()
                    .all(|city| game.factions.contains_key(&city.faction_id))
            );
            assert!(
                game.officers
                    .values()
                    .all(|officer| officer.profile.is_some())
            );
            assert!(
                game.officers
                    .values()
                    .all(|officer| officer.birth_year != 0)
            );

            for city in game.cities.values() {
                assert!((1..=CITY_MAX_LEVEL).contains(&city.level));
                assert!(city.materials >= 0);
                assert!(city.facilities.len() <= city.facility_slots());
                for facility in &city.facilities {
                    assert!((1..=FACILITY_MAX_LEVEL).contains(&facility.level));
                    assert!(facility.level <= city.level);
                }
                if let Some(governor_id) = &city.governor_id {
                    let governor = game.officers.get(governor_id).unwrap();
                    assert!(governor.is_active());
                    assert_eq!(governor.faction_id, city.faction_id);
                    assert_eq!(governor.city_id.as_deref(), Some(city.id.as_str()));
                }
            }

            for road in &game.roads {
                let distance = game.road_distance_li(&road.from, &road.to).unwrap();
                let months = game.travel_months_between(&road.from, &road.to).unwrap();
                assert!(distance > 0);
                assert!((1..=MAX_TRAVEL_MONTHS).contains(&months));
            }
        }
    }
}

#[test]
fn taipingdao_scenario_builds_yellow_turban_roster() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();
    let factions = catalog.selectable_factions("ad180").unwrap();
    let selectable_ids = factions
        .iter()
        .filter(|faction| faction.selectable)
        .map(|faction| faction.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        selectable_ids,
        BTreeSet::from([
            "dong_zhuo",
            "gongsun_zan",
            "han_court",
            "liu_biao",
            "liu_yan",
            "ma_teng",
            "shi_xie",
            "sun_quan",
            "tao_qian",
            "yellow_turban",
        ])
    );

    let game = catalog.build_game("ad180", "yellow_turban").unwrap();
    assert_eq!(game.scenario_name, "光和三年 太平道将兴");
    assert_eq!(game.year, 180);
    assert_eq!(game.factions["yellow_turban"].ruler_id, "ctk_5f20_89d2");
    assert_eq!(game.factions["han_court"].ruler_id, "ctk_5218_5b8f");
    assert_eq!(game.factions["sun_quan"].ruler_id, "sun_jian");
    assert_eq!(game.factions["liu_yan"].ruler_id, "ctk_5218_7109");
    assert_eq!(game.cities["ganling"].faction_id, "yellow_turban");
    assert_eq!(game.cities["zhongshan"].faction_id, "yellow_turban");
    assert_eq!(game.cities["runan"].faction_id, "yellow_turban");
    assert_eq!(game.cities["anding"].faction_id, "dong_zhuo");
    assert_eq!(game.cities["wuwei"].faction_id, "ma_teng");
    assert_eq!(game.cities["youbeiping"].faction_id, "gongsun_zan");
    assert_eq!(game.cities["xiapi"].faction_id, "tao_qian");
    assert_eq!(game.cities["xiangyang"].faction_id, "liu_biao");
    assert_eq!(game.cities["chengdu"].faction_id, "liu_yan");
    assert_eq!(game.cities["wu"].faction_id, "sun_quan");
    assert_eq!(game.cities["jiaozhi"].faction_id, "shi_xie");

    let yellow_roster = game
        .officers
        .values()
        .filter(|officer| officer.faction_id == "yellow_turban" && officer.is_active())
        .map(|officer| officer.id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(yellow_roster.len() >= 10);
    for officer_id in [
        "ctk_5f20_89d2",
        "ctk_5f20_5b9d",
        "ctk_5f20_6881",
        "ctk_5f20_71d5",
        "bo_cai",
        "ma_yuanyi",
        "bu_ji",
        "peng_tuo",
        "guan_hai",
        "zhang_mancheng",
        "zhao_hong_yellow",
        "han_zhong_yellow",
        "sun_xia_yellow",
        "huang_shao",
        "he_man",
        "pei_yuanshao",
    ] {
        assert!(yellow_roster.contains(officer_id), "missing {officer_id}");
        let officer = &game.officers[officer_id];
        assert!(officer.profile.is_some());
        assert!(officer.birth_year != 0);
        assert!(matches!(
            officer.city_id.as_deref(),
            Some("ganling" | "zhongshan" | "runan")
        ));
    }

    let events = catalog.life_events_until(180, 1).unwrap();
    for officer_id in [
        "bo_cai",
        "ma_yuanyi",
        "bu_ji",
        "peng_tuo",
        "guan_hai",
        "zhang_mancheng",
        "zhao_hong_yellow",
        "han_zhong_yellow",
        "sun_xia_yellow",
        "huang_shao",
        "he_man",
        "pei_yuanshao",
    ] {
        let profile = catalog.officer_profile(officer_id).unwrap().unwrap();
        assert!(profile.birth_year.is_some());
        assert!(
            events.iter().any(|event| event.officer_id == officer_id
                && event.faction_id.as_deref() == Some("yellow_turban")
                && event.city_id.is_some()),
            "missing 180 life event for {officer_id}"
        );
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
    assert_eq!(jiang_wei.status, OfficerStatus::Active);
    assert_eq!(jiang_wei.faction_id, "liu_bei");
    assert!(jiang_wei.city_id.is_some());
    assert!(appear_game.applied_event_ids.contains("start_jiang_wei"));

    let mut death_game = catalog.build_game("ad200", "sun_quan").unwrap();
    assert!(death_game.officers["sun_ce"].is_active());
    death_game.year = 200;
    death_game.month = 11;

    let first_report = resolve_command_batch_with_history(&mut death_game, Vec::new(), &catalog);

    assert!(death_game.applied_event_ids.contains("death_sun_ce"));
    assert_eq!(death_game.officers["sun_ce"].status, OfficerStatus::Active);
    assert!(
        first_report
            .entries
            .iter()
            .any(|entry| entry.message.contains("历史卒年已记录为资料"))
    );

    let applied_count = death_game.applied_event_ids.len();
    death_game.year = 200;
    death_game.month = 11;
    let second_report = resolve_command_batch_with_history(&mut death_game, Vec::new(), &catalog);

    assert_eq!(death_game.applied_event_ids.len(), applied_count);
    assert!(
        !second_report
            .entries
            .iter()
            .any(|entry| entry.message.contains("孙策"))
    );
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
    assert_eq!(officer.status, OfficerStatus::Wild);
    assert_eq!(officer.faction_id, WILD_FACTION_ID);
    assert!(officer.city_id.is_some());
}

#[test]
fn life_event_officer_becomes_wild_when_target_faction_has_no_city() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    let mut game = catalog.build_game("ad208", "cao_cao").unwrap();
    for city in game.cities.values_mut() {
        if city.faction_id == "liu_bei" {
            city.faction_id = "cao_cao".to_string();
        }
    }
    game.year = 219;
    game.month = 12;

    resolve_command_batch_with_history(&mut game, Vec::new(), &catalog);

    let jiang_wei = game.officers.get("jiang_wei").unwrap();
    assert_eq!(jiang_wei.status, OfficerStatus::Wild);
    assert_eq!(jiang_wei.faction_id, WILD_FACTION_ID);
    assert!(jiang_wei.city_id.is_some());
}

#[test]
fn starting_scenario_keeps_cutoff_wild_life_event_officers() {
    let catalog = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();

    let game = catalog.build_game("ad208", "liu_bei").unwrap();

    assert!(
        game.officers
            .values()
            .any(|officer| officer.status == OfficerStatus::Wild
                && officer.faction_id == WILD_FACTION_ID
                && officer.city_id.is_some())
    );
}
