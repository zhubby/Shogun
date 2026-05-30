use super::city::{City, CityCatalog, CityProfile, CityScale, SourceConfidence};
use super::ids::{CityId, FactionId, OfficerId, ScenarioId};
use super::model::{
    diplomacy_key, Controller, DiplomaticRelation, Faction, GameState, GameStatus, MapPosition,
    Road, SAVE_VERSION,
};
use super::officer::{
    Officer, OfficerCatalog, OfficerGender, OfficerProfile, OfficerRelationship,
    OfficerRelationshipKind, OfficerStats, OfficerStatus,
};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sqlx::migrate::Migration;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const HISTORY_DB_FILE_NAME: &str = "database.sqlite";
pub const HISTORY_DB_SCHEMA_VERSION: u32 = 3;
const LEGACY_UNVERSIONED_DB_VERSION: u32 = 1;
static HISTORY_DB_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("assets/data/migrations");
const REQUIRED_HISTORY_TABLES: &[&str] = &[
    "cities",
    "factions",
    "officers",
    "officer_external_ids",
    "officer_relationships",
    "roads",
    "scenarios",
    "scenario_faction_states",
    "scenario_city_states",
    "officer_life_events",
    "scenario_diplomacy",
];
const REQUIRED_LEGACY_HISTORY_TABLES: &[&str] = &[
    "cities",
    "factions",
    "officers",
    "roads",
    "scenarios",
    "scenario_faction_states",
    "scenario_city_states",
    "officer_life_events",
    "scenario_diplomacy",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoricalScenario {
    pub id: ScenarioId,
    pub name: String,
    pub year: i32,
    pub month: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifeEvent {
    pub id: String,
    pub officer_id: OfficerId,
    pub year: i32,
    pub month: u8,
    pub kind: LifeEventKind,
    pub faction_id: Option<FactionId>,
    pub city_id: Option<CityId>,
    pub loyalty: Option<u8>,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum LifeEventKind {
    Appear,
    ServeFaction,
    MoveToCity,
    BecomeUnavailable,
    Die,
}

pub trait HistoricalCatalog:
    CityCatalog<Error = HistoryDbError> + OfficerCatalog<Error = HistoryDbError>
{
    fn scenarios(&self) -> Result<Vec<HistoricalScenario>, HistoryDbError>;
    fn selectable_factions(&self, scenario_id: &str) -> Result<Vec<Faction>, HistoryDbError>;
    fn build_game(
        &self,
        scenario_id: &str,
        player_faction_id: &str,
    ) -> Result<GameState, HistoryDbError>;
    fn life_events_until(&self, year: i32, month: u8) -> Result<Vec<LifeEvent>, HistoryDbError>;
}

pub struct SqliteHistoricalCatalog {
    conn: Connection,
}

impl SqliteHistoricalCatalog {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, HistoryDbError> {
        let conn = Connection::open(path).map_err(HistoryDbError::Sql)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(HistoryDbError::Sql)?;
        Ok(Self { conn })
    }

    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, HistoryDbError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(HistoryDbError::Io)?;
        }
        migrate_history_database(path)?;
        let conn = Connection::open(path).map_err(HistoryDbError::Sql)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(HistoryDbError::Sql)?;
        validate_history_database(&conn)?;
        Ok(Self { conn })
    }

    pub fn open_default() -> Result<Self, HistoryDbError> {
        Self::open_or_create(Self::default_path())
    }

    pub fn open_asset() -> Result<Self, HistoryDbError> {
        Self::open_default()
    }

    pub fn default_path() -> PathBuf {
        Self::database_path_in(Self::default_data_dir())
    }

    pub fn default_data_dir() -> PathBuf {
        ProjectDirs::from("", "", "Shogun")
            .map(|dirs| dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(Self::fallback_data_dir)
    }

    pub fn fallback_data_dir() -> PathBuf {
        PathBuf::from(".shogun_data")
    }

    pub fn database_path_in(data_dir: impl AsRef<Path>) -> PathBuf {
        data_dir.as_ref().join(HISTORY_DB_FILE_NAME)
    }

    pub fn in_memory_from_seed() -> Result<Self, HistoryDbError> {
        let mut conn = Connection::open_in_memory().map_err(HistoryDbError::Sql)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(HistoryDbError::Sql)?;
        initialize_history_database(&mut conn)?;
        Ok(Self { conn })
    }

    pub fn officer_relationships(
        &self,
        officer_id: &str,
    ) -> Result<Vec<OfficerRelationship>, HistoryDbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT r.target_officer_id, target.name, r.relationship_kind,
                        r.confidence, r.notes, r.source
                 FROM officer_relationships r
                 JOIN officers target ON target.id = r.target_officer_id
                 WHERE r.source_officer_id = ?1
                 ORDER BY r.relationship_kind, target.name",
            )
            .map_err(HistoryDbError::Sql)?;
        let rows = stmt
            .query_map(params![officer_id], |row| {
                Ok(OfficerRelationship {
                    target_id: row.get(0)?,
                    target_name: row.get(1)?,
                    kind: parse_relationship_kind(&row.get::<_, String>(2)?),
                    confidence: parse_confidence(&row.get::<_, String>(3)?),
                    notes: row.get(4)?,
                    source: row.get(5)?,
                })
            })
            .map_err(HistoryDbError::Sql)?;
        collect_rows(rows)
    }
}

pub fn build_history_database(path: impl AsRef<Path>) -> Result<(), HistoryDbError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(HistoryDbError::Io)?;
    }
    if path.exists() {
        fs::remove_file(path).map_err(HistoryDbError::Io)?;
    }
    migrate_history_database(path)?;
    Ok(())
}

fn migrate_history_database(path: &Path) -> Result<(), HistoryDbError> {
    {
        let conn = Connection::open(path).map_err(HistoryDbError::Sql)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(HistoryDbError::Sql)?;
        prepare_history_database_for_sqlx_migrations(&conn)?;
    }

    sqlx::test_block_on(async {
        let mut conn = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(false)
            .connect()
            .await?;
        HISTORY_DB_MIGRATOR.run_direct(&mut conn).await
    })
    .map_err(HistoryDbError::Migrate)?;

    let conn = Connection::open(path).map_err(HistoryDbError::Sql)?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(HistoryDbError::Sql)?;
    update_history_database_user_version(&conn)?;
    validate_history_database(&conn)
}

fn prepare_history_database_for_sqlx_migrations(conn: &Connection) -> Result<(), HistoryDbError> {
    let mut current_version = database_user_version(conn)?;
    if current_version > HISTORY_DB_SCHEMA_VERSION {
        return Err(HistoryDbError::Invalid(format!(
            "历史资料库版本 {current_version} 高于当前程序支持版本 {HISTORY_DB_SCHEMA_VERSION}"
        )));
    }

    if current_version == 0 && database_is_empty(conn)? {
        return Ok(());
    }

    if current_version == 0 && !database_is_empty(conn)? {
        adopt_legacy_v1_database(conn)?;
        current_version = database_user_version(conn)?;
    }

    record_applied_user_version_migrations(conn, current_version)
}

fn initialize_history_database(conn: &mut Connection) -> Result<(), HistoryDbError> {
    conn.pragma_update(None, "foreign_keys", "OFF")
        .map_err(HistoryDbError::Sql)?;
    let transaction = conn.transaction().map_err(HistoryDbError::Sql)?;
    for migration in HISTORY_DB_MIGRATOR.iter() {
        transaction
            .execute_batch(&migration.sql)
            .map_err(HistoryDbError::Sql)?;
    }
    transaction.commit().map_err(HistoryDbError::Sql)?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(HistoryDbError::Sql)?;
    update_history_database_user_version(conn)?;
    validate_history_database(conn)
}

fn record_applied_user_version_migrations(
    conn: &Connection,
    current_version: u32,
) -> Result<(), HistoryDbError> {
    if current_version == 0 {
        return Ok(());
    }

    create_sqlx_migrations_table(conn)?;
    for migration in HISTORY_DB_MIGRATOR
        .iter()
        .filter(|migration| migration.version <= i64::from(current_version))
    {
        record_applied_migration(conn, migration)?;
    }
    Ok(())
}

fn create_sqlx_migrations_table(conn: &Connection) -> Result<(), HistoryDbError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        );
        "#,
    )
    .map_err(HistoryDbError::Sql)
}

fn record_applied_migration(
    conn: &Connection,
    migration: &Migration,
) -> Result<(), HistoryDbError> {
    conn.execute(
        "INSERT OR IGNORE INTO _sqlx_migrations
         (version, description, success, checksum, execution_time)
         VALUES (?1, ?2, TRUE, ?3, 0)",
        (
            migration.version,
            migration.description.as_ref(),
            migration.checksum.as_ref(),
        ),
    )
    .map_err(HistoryDbError::Sql)?;
    Ok(())
}

fn update_history_database_user_version(conn: &Connection) -> Result<(), HistoryDbError> {
    conn.pragma_update(None, "user_version", HISTORY_DB_SCHEMA_VERSION)
        .map_err(HistoryDbError::Sql)
}

fn validate_history_database(conn: &Connection) -> Result<(), HistoryDbError> {
    validate_required_tables(conn)?;
    validate_foreign_keys(conn)
}

fn adopt_legacy_v1_database(conn: &Connection) -> Result<(), HistoryDbError> {
    validate_tables(conn, REQUIRED_LEGACY_HISTORY_TABLES)?;
    validate_foreign_keys(conn)?;
    conn.pragma_update(None, "user_version", LEGACY_UNVERSIONED_DB_VERSION)
        .map_err(HistoryDbError::Sql)
}

fn database_user_version(conn: &Connection) -> Result<u32, HistoryDbError> {
    let version = conn
        .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
        .map_err(HistoryDbError::Sql)?;
    Ok(version as u32)
}

fn database_is_empty(conn: &Connection) -> Result<bool, HistoryDbError> {
    let table_count = conn
        .query_row(
            "SELECT count(*)
             FROM sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(HistoryDbError::Sql)?;
    Ok(table_count == 0)
}

fn validate_required_tables(conn: &Connection) -> Result<(), HistoryDbError> {
    validate_tables(conn, REQUIRED_HISTORY_TABLES)
}

fn validate_tables(conn: &Connection, required_tables: &[&str]) -> Result<(), HistoryDbError> {
    let tables = history_table_names(conn)?;
    let missing = required_tables
        .iter()
        .copied()
        .filter(|table| !tables.contains(*table))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(HistoryDbError::Invalid(format!(
            "历史资料库缺少表: {}",
            missing.join(", ")
        )))
    }
}

fn history_table_names(conn: &Connection) -> Result<BTreeSet<String>, HistoryDbError> {
    let mut stmt = conn
        .prepare(
            "SELECT name
             FROM sqlite_master
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )
        .map_err(HistoryDbError::Sql)?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(HistoryDbError::Sql)?;
    rows.collect::<Result<BTreeSet<_>, _>>()
        .map_err(HistoryDbError::Sql)
}

fn validate_foreign_keys(conn: &Connection) -> Result<(), HistoryDbError> {
    let mut stmt = conn
        .prepare("PRAGMA foreign_key_check")
        .map_err(HistoryDbError::Sql)?;
    let mut rows = stmt.query([]).map_err(HistoryDbError::Sql)?;
    if rows.next().map_err(HistoryDbError::Sql)?.is_some() {
        Err(HistoryDbError::Invalid(
            "历史资料库外键校验失败".to_string(),
        ))
    } else {
        Ok(())
    }
}

impl CityCatalog for SqliteHistoricalCatalog {
    type Error = HistoryDbError;

    fn city_profiles(&self) -> Result<Vec<CityProfile>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, province, commandery, x, y, scale, strategic_rank,
                        agriculture_base, commerce_base, defense_base, population_min,
                        population_max, confidence, notes
                 FROM cities
                 ORDER BY province, commandery, name",
            )
            .map_err(HistoryDbError::Sql)?;
        let rows = stmt
            .query_map([], city_profile_from_row)
            .map_err(HistoryDbError::Sql)?;
        collect_rows(rows)
    }

    fn city_profile(&self, city_id: &str) -> Result<Option<CityProfile>, Self::Error> {
        self.conn
            .query_row(
                "SELECT id, name, province, commandery, x, y, scale, strategic_rank,
                        agriculture_base, commerce_base, defense_base, population_min,
                        population_max, confidence, notes
                 FROM cities
                 WHERE id = ?1",
                params![city_id],
                city_profile_from_row,
            )
            .optional()
            .map_err(HistoryDbError::Sql)
    }
}

impl OfficerCatalog for SqliteHistoricalCatalog {
    type Error = HistoryDbError;

    fn officer_profiles(&self) -> Result<Vec<OfficerProfile>, Self::Error> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, courtesy_name, native_place, birth_year, death_year,
                        gender, leadership, strength, intelligence, politics, charm, tags,
                        confidence, biography, notes
                 FROM officers
                 ORDER BY id",
            )
            .map_err(HistoryDbError::Sql)?;
        let rows = stmt
            .query_map([], officer_profile_from_row)
            .map_err(HistoryDbError::Sql)?;
        let mut profiles = collect_rows(rows)?;
        for profile in &mut profiles {
            profile.relationships = self.officer_relationships(&profile.id)?;
        }
        Ok(profiles)
    }

    fn officer_profile(&self, officer_id: &str) -> Result<Option<OfficerProfile>, Self::Error> {
        let mut profile = self
            .conn
            .query_row(
                "SELECT id, name, courtesy_name, native_place, birth_year, death_year,
                        gender, leadership, strength, intelligence, politics, charm, tags,
                        confidence, biography, notes
                 FROM officers
                 WHERE id = ?1",
                params![officer_id],
                officer_profile_from_row,
            )
            .optional()
            .map_err(HistoryDbError::Sql)?;
        if let Some(profile) = &mut profile {
            profile.relationships = self.officer_relationships(&profile.id)?;
        }
        Ok(profile)
    }
}

impl HistoricalCatalog for SqliteHistoricalCatalog {
    fn scenarios(&self) -> Result<Vec<HistoricalScenario>, HistoryDbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, year, month FROM scenarios ORDER BY year, month")
            .map_err(HistoryDbError::Sql)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(HistoricalScenario {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    year: row.get(2)?,
                    month: row.get(3)?,
                })
            })
            .map_err(HistoryDbError::Sql)?;
        collect_rows(rows)
    }

    fn selectable_factions(&self, scenario_id: &str) -> Result<Vec<Faction>, HistoryDbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT f.id, f.name, s.ruler_id, f.color_r, f.color_g, f.color_b, s.selectable
                 FROM scenario_faction_states s
                 JOIN factions f ON f.id = s.faction_id
                 WHERE s.scenario_id = ?1 AND s.exists_in_scenario = 1
                 ORDER BY f.id",
            )
            .map_err(HistoryDbError::Sql)?;
        let rows = stmt
            .query_map(params![scenario_id], |row| {
                Ok(Faction {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    ruler_id: row.get(2)?,
                    color: [row.get(3)?, row.get(4)?, row.get(5)?],
                    selectable: row.get::<_, i64>(6)? != 0,
                    controlled_by: Controller::RuleAi,
                })
            })
            .map_err(HistoryDbError::Sql)?;
        collect_rows(rows)
    }

    fn build_game(
        &self,
        scenario_id: &str,
        player_faction_id: &str,
    ) -> Result<GameState, HistoryDbError> {
        let scenario = self
            .scenarios()?
            .into_iter()
            .find(|scenario| scenario.id == scenario_id)
            .ok_or_else(|| HistoryDbError::Invalid(format!("剧本 {scenario_id} 不存在")))?;

        let mut factions = BTreeMap::new();
        for mut faction in self.selectable_factions(scenario_id)? {
            faction.controlled_by = if faction.id == player_faction_id {
                Controller::Player
            } else {
                Controller::RuleAi
            };
            factions.insert(faction.id.clone(), faction);
        }
        if !factions
            .get(player_faction_id)
            .is_some_and(|faction| faction.selectable)
        {
            return Err(HistoryDbError::Invalid(format!(
                "势力 {player_faction_id} 不可选"
            )));
        }

        let mut cities = BTreeMap::new();
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.id, c.name, c.province, c.commandery, c.x, c.y, c.scale,
                        c.strategic_rank, c.agriculture_base, c.commerce_base,
                        c.defense_base, c.population_min, c.population_max, c.confidence,
                        c.notes, s.faction_id, s.population, s.gold, s.food, s.troops,
                        s.training, s.agriculture, s.commerce, s.defense, s.city_order,
                        s.governor_id
                 FROM scenario_city_states s
                 JOIN cities c ON c.id = s.city_id
                 WHERE s.scenario_id = ?1",
            )
            .map_err(HistoryDbError::Sql)?;
        let rows = stmt
            .query_map(params![scenario_id], |row| {
                let profile = CityProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    province: row.get(2)?,
                    commandery: row.get(3)?,
                    position: MapPosition {
                        x: row.get(4)?,
                        y: row.get(5)?,
                    },
                    scale: parse_city_scale(&row.get::<_, String>(6)?),
                    strategic_rank: row.get::<_, i64>(7)? as u8,
                    agriculture_base: row.get::<_, i64>(8)? as u16,
                    commerce_base: row.get::<_, i64>(9)? as u16,
                    defense_base: row.get::<_, i64>(10)? as u16,
                    population_min: row.get::<_, i64>(11)? as u32,
                    population_max: row.get::<_, i64>(12)? as u32,
                    confidence: parse_confidence(&row.get::<_, String>(13)?),
                    notes: row.get(14)?,
                };
                Ok(City {
                    id: profile.id.clone(),
                    name: profile.name.clone(),
                    faction_id: row.get(15)?,
                    position: profile.position,
                    population: row.get::<_, i64>(16)? as u32,
                    gold: row.get::<_, i64>(17)? as i32,
                    food: row.get::<_, i64>(18)? as i32,
                    troops: row.get::<_, i64>(19)? as u32,
                    training: row.get::<_, i64>(20)? as u8,
                    agriculture: row.get::<_, i64>(21)? as u16,
                    commerce: row.get::<_, i64>(22)? as u16,
                    defense: row.get::<_, i64>(23)? as u16,
                    order: row.get::<_, i64>(24)? as u8,
                    governor_id: row.get(25)?,
                    profile: Some(profile),
                })
            })
            .map_err(HistoryDbError::Sql)?;
        for city in collect_rows(rows)? {
            cities.insert(city.id.clone(), city);
        }

        let mut roads = Vec::new();
        let mut road_stmt = self
            .conn
            .prepare("SELECT from_city_id, to_city_id FROM roads ORDER BY from_city_id, to_city_id")
            .map_err(HistoryDbError::Sql)?;
        let road_rows = road_stmt
            .query_map([], |row| {
                Ok(Road {
                    from: row.get(0)?,
                    to: row.get(1)?,
                })
            })
            .map_err(HistoryDbError::Sql)?;
        for road in collect_rows(road_rows)? {
            if cities.contains_key(&road.from) && cities.contains_key(&road.to) {
                roads.push(road);
            }
        }

        let mut diplomacy = BTreeMap::new();
        let mut dip_stmt = self
            .conn
            .prepare(
                "SELECT faction_a, faction_b, score, truce_until_turn
                 FROM scenario_diplomacy
                 WHERE scenario_id = ?1",
            )
            .map_err(HistoryDbError::Sql)?;
        let dip_rows = dip_stmt
            .query_map(params![scenario_id], |row| {
                Ok(DiplomaticRelation {
                    faction_a: row.get(0)?,
                    faction_b: row.get(1)?,
                    score: row.get::<_, i64>(2)? as i16,
                    truce_until_turn: row.get::<_, Option<i64>>(3)?.map(|value| value as u32),
                })
            })
            .map_err(HistoryDbError::Sql)?;
        for relation in collect_rows(dip_rows)? {
            diplomacy.insert(
                diplomacy_key(&relation.faction_a, &relation.faction_b),
                relation,
            );
        }

        let cutoff_events = self.life_events_until(scenario.year, scenario.month)?;
        let mut applied_event_ids = BTreeSet::new();
        let mut officer_states: BTreeMap<OfficerId, Officer> = BTreeMap::new();
        for event in cutoff_events {
            applied_event_ids.insert(event.id.clone());
            let Some(profile) = self.officer_profile(&event.officer_id)? else {
                continue;
            };
            apply_initial_life_event(&mut officer_states, &cities, &factions, &profile, &event);
        }
        clean_invalid_governors(&mut cities, &officer_states);

        Ok(GameState {
            version: SAVE_VERSION,
            scenario_id: scenario.id,
            scenario_name: scenario.name,
            year: scenario.year,
            month: scenario.month,
            turn: 1,
            player_faction_id: player_faction_id.to_string(),
            factions,
            cities,
            officers: officer_states,
            roads,
            diplomacy,
            pending_commands: Vec::new(),
            applied_event_ids,
            reports: Vec::new(),
            status: GameStatus::Running,
        })
    }

    fn life_events_until(&self, year: i32, month: u8) -> Result<Vec<LifeEvent>, HistoryDbError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, officer_id, event_year, event_month, event_kind,
                        faction_id, city_id, loyalty, notes
                 FROM officer_life_events
                 WHERE event_year < ?1 OR (event_year = ?1 AND event_month <= ?2)
                 ORDER BY event_year, event_month, id",
            )
            .map_err(HistoryDbError::Sql)?;
        let rows = stmt
            .query_map(params![year, month], life_event_from_row)
            .map_err(HistoryDbError::Sql)?;
        collect_rows(rows)
    }
}

fn apply_initial_life_event(
    officers: &mut BTreeMap<OfficerId, Officer>,
    cities: &BTreeMap<CityId, City>,
    factions: &BTreeMap<FactionId, Faction>,
    profile: &OfficerProfile,
    event: &LifeEvent,
) {
    match event.kind {
        LifeEventKind::Appear | LifeEventKind::ServeFaction | LifeEventKind::MoveToCity => {
            let faction_id = event
                .faction_id
                .clone()
                .or_else(|| {
                    event
                        .city_id
                        .as_ref()
                        .and_then(|city_id| cities.get(city_id))
                        .map(|city| city.faction_id.clone())
                })
                .unwrap_or_else(|| "wild".to_string());
            let city_id = event
                .city_id
                .as_ref()
                .and_then(|city_id| cities.get(city_id))
                .filter(|city| city.faction_id == faction_id)
                .map(|city| city.id.clone())
                .or_else(|| {
                    cities
                        .values()
                        .find(|city| city.faction_id == faction_id)
                        .map(|city| city.id.clone())
                });
            if factions.contains_key(&faction_id) && city_id.is_some() {
                officers.insert(
                    profile.id.clone(),
                    Officer {
                        id: profile.id.clone(),
                        name: profile.name.clone(),
                        faction_id,
                        city_id,
                        stats: profile.stats,
                        loyalty: event.loyalty.unwrap_or(80),
                        gender: profile.gender.clone(),
                        status: OfficerStatus::Active,
                        profile: Some(profile.clone()),
                    },
                );
            } else {
                officers.remove(&profile.id);
            }
        }
        LifeEventKind::BecomeUnavailable | LifeEventKind::Die => {
            officers.remove(&profile.id);
        }
    }
}

fn clean_invalid_governors(
    cities: &mut BTreeMap<CityId, City>,
    officers: &BTreeMap<OfficerId, Officer>,
) {
    for city in cities.values_mut() {
        let governor_valid = city.governor_id.as_ref().is_some_and(|governor_id| {
            officers.get(governor_id).is_some_and(|officer| {
                officer.is_active()
                    && officer.faction_id == city.faction_id
                    && officer.city_id.as_deref() == Some(city.id.as_str())
            })
        });
        if !governor_valid {
            city.governor_id = None;
        }
    }
}

fn city_profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CityProfile> {
    Ok(CityProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        province: row.get(2)?,
        commandery: row.get(3)?,
        position: MapPosition {
            x: row.get(4)?,
            y: row.get(5)?,
        },
        scale: parse_city_scale(&row.get::<_, String>(6)?),
        strategic_rank: row.get::<_, i64>(7)? as u8,
        agriculture_base: row.get::<_, i64>(8)? as u16,
        commerce_base: row.get::<_, i64>(9)? as u16,
        defense_base: row.get::<_, i64>(10)? as u16,
        population_min: row.get::<_, i64>(11)? as u32,
        population_max: row.get::<_, i64>(12)? as u32,
        confidence: parse_confidence(&row.get::<_, String>(13)?),
        notes: row.get(14)?,
    })
}

fn officer_profile_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OfficerProfile> {
    Ok(OfficerProfile {
        id: row.get(0)?,
        name: row.get(1)?,
        courtesy_name: row.get(2)?,
        native_place: row.get(3)?,
        birth_year: row.get(4)?,
        death_year: row.get(5)?,
        gender: parse_gender(&row.get::<_, String>(6)?),
        stats: OfficerStats {
            leadership: row.get::<_, i64>(7)? as u8,
            strength: row.get::<_, i64>(8)? as u8,
            intelligence: row.get::<_, i64>(9)? as u8,
            politics: row.get::<_, i64>(10)? as u8,
            charm: row.get::<_, i64>(11)? as u8,
        },
        tags: row
            .get::<_, String>(12)?
            .split(',')
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect(),
        confidence: parse_confidence(&row.get::<_, String>(13)?),
        biography: row.get(14)?,
        relationships: Vec::new(),
        notes: row.get(15)?,
    })
}

fn life_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LifeEvent> {
    Ok(LifeEvent {
        id: row.get(0)?,
        officer_id: row.get(1)?,
        year: row.get(2)?,
        month: row.get::<_, i64>(3)? as u8,
        kind: parse_life_event_kind(&row.get::<_, String>(4)?),
        faction_id: row.get(5)?,
        city_id: row.get(6)?,
        loyalty: row.get::<_, Option<i64>>(7)?.map(|value| value as u8),
        notes: row.get(8)?,
    })
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>, HistoryDbError> {
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(HistoryDbError::Sql)
}

fn parse_city_scale(value: &str) -> CityScale {
    match value {
        "County" => CityScale::County,
        "RegionalCapital" => CityScale::RegionalCapital,
        "ImperialCapital" => CityScale::ImperialCapital,
        _ => CityScale::Commandery,
    }
}

fn parse_confidence(value: &str) -> SourceConfidence {
    match value {
        "High" => SourceConfidence::High,
        "Low" => SourceConfidence::Low,
        _ => SourceConfidence::Medium,
    }
}

fn parse_gender(value: &str) -> OfficerGender {
    match value {
        "Female" => OfficerGender::Female,
        _ => OfficerGender::Male,
    }
}

fn parse_relationship_kind(value: &str) -> OfficerRelationshipKind {
    match value {
        "RulerSubject" => OfficerRelationshipKind::RulerSubject,
        "ParentChild" => OfficerRelationshipKind::ParentChild,
        "AdoptiveParentChild" => OfficerRelationshipKind::AdoptiveParentChild,
        "Spouse" => OfficerRelationshipKind::Spouse,
        "Sibling" => OfficerRelationshipKind::Sibling,
        "SwornSibling" => OfficerRelationshipKind::SwornSibling,
        "Enemy" => OfficerRelationshipKind::Enemy,
        _ => OfficerRelationshipKind::RulerSubject,
    }
}

fn parse_life_event_kind(value: &str) -> LifeEventKind {
    match value {
        "Appear" => LifeEventKind::Appear,
        "ServeFaction" => LifeEventKind::ServeFaction,
        "MoveToCity" => LifeEventKind::MoveToCity,
        "BecomeUnavailable" => LifeEventKind::BecomeUnavailable,
        "Die" => LifeEventKind::Die,
        _ => LifeEventKind::Appear,
    }
}

#[derive(Debug)]
pub enum HistoryDbError {
    Io(std::io::Error),
    Sql(rusqlite::Error),
    Migrate(sqlx::migrate::MigrateError),
    Invalid(String),
}

impl std::fmt::Display for HistoryDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryDbError::Io(error) => write!(f, "历史资料库 IO 失败: {error}"),
            HistoryDbError::Sql(error) => write!(f, "历史资料库 SQL 失败: {error}"),
            HistoryDbError::Migrate(error) => write!(f, "历史资料库迁移失败: {error}"),
            HistoryDbError::Invalid(message) => write!(f, "历史资料库数据无效: {message}"),
        }
    }
}

impl std::error::Error for HistoryDbError {}
