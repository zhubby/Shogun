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
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::Row;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};

const HISTORY_DB_FILE_NAME: &str = "database.sqlite";
static HISTORY_DB_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
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
    pool: SqlitePool,
    runtime: tokio::runtime::Runtime,
}

impl SqliteHistoricalCatalog {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, HistoryDbError> {
        let runtime = history_db_runtime()?;
        let pool = runtime.block_on(async {
            let pool = open_history_pool(path.as_ref(), false).await?;
            validate_history_database(&pool).await?;
            Ok::<_, HistoryDbError>(pool)
        })?;
        Ok(Self { pool, runtime })
    }

    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, HistoryDbError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(HistoryDbError::Io)?;
        }
        let runtime = history_db_runtime()?;
        let pool = runtime.block_on(async {
            let pool = open_history_pool(path, true).await?;
            run_history_migrations(&pool).await?;
            validate_history_database(&pool).await?;
            Ok::<_, HistoryDbError>(pool)
        })?;
        Ok(Self { pool, runtime })
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
        let runtime = history_db_runtime()?;
        let pool = runtime.block_on(async {
            let pool = open_memory_history_pool().await?;
            run_history_migrations(&pool).await?;
            validate_history_database(&pool).await?;
            Ok::<_, HistoryDbError>(pool)
        })?;
        Ok(Self { pool, runtime })
    }

    fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    pub fn officer_relationships(
        &self,
        officer_id: &str,
    ) -> Result<Vec<OfficerRelationship>, HistoryDbError> {
        self.block_on(async { officer_relationships(&self.pool, officer_id).await })
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

    let runtime = history_db_runtime()?;
    runtime.block_on(async {
        let pool = open_history_pool(path, true).await?;
        run_history_migrations(&pool).await?;
        validate_history_database(&pool).await?;
        pool.close().await;
        Ok(())
    })
}

fn history_db_runtime() -> Result<tokio::runtime::Runtime, HistoryDbError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(HistoryDbError::Runtime)
}

async fn open_history_pool(
    path: &Path,
    create_if_missing: bool,
) -> Result<SqlitePool, HistoryDbError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(create_if_missing)
        .foreign_keys(true);
    open_pool_with_options(options).await
}

async fn open_memory_history_pool() -> Result<SqlitePool, HistoryDbError> {
    let options = SqliteConnectOptions::new()
        .in_memory(true)
        .shared_cache(true)
        .foreign_keys(true);
    open_pool_with_options(options).await
}

async fn open_pool_with_options(
    options: SqliteConnectOptions,
) -> Result<SqlitePool, HistoryDbError> {
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(HistoryDbError::Sqlx)
}

async fn run_history_migrations(pool: &SqlitePool) -> Result<(), HistoryDbError> {
    HISTORY_DB_MIGRATOR
        .run(pool)
        .await
        .map_err(HistoryDbError::Migrate)
}

async fn validate_history_database(pool: &SqlitePool) -> Result<(), HistoryDbError> {
    validate_required_tables(pool).await?;
    validate_foreign_keys(pool).await
}

async fn validate_required_tables(pool: &SqlitePool) -> Result<(), HistoryDbError> {
    validate_tables(pool, REQUIRED_HISTORY_TABLES).await
}

async fn validate_tables(
    pool: &SqlitePool,
    required_tables: &[&str],
) -> Result<(), HistoryDbError> {
    let tables = history_table_names(pool).await?;
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

async fn history_table_names(pool: &SqlitePool) -> Result<BTreeSet<String>, HistoryDbError> {
    let rows = sqlx::query(
        "SELECT name
         FROM sqlite_master
         WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(pool)
    .await
    .map_err(HistoryDbError::Sqlx)?;
    Ok(rows.into_iter().map(|row| row.get("name")).collect())
}

async fn validate_foreign_keys(pool: &SqlitePool) -> Result<(), HistoryDbError> {
    let rows = sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(pool)
        .await
        .map_err(HistoryDbError::Sqlx)?;
    if rows.is_empty() {
        Ok(())
    } else {
        Err(HistoryDbError::Invalid(
            "历史资料库外键校验失败".to_string(),
        ))
    }
}

impl CityCatalog for SqliteHistoricalCatalog {
    type Error = HistoryDbError;

    fn city_profiles(&self) -> Result<Vec<CityProfile>, Self::Error> {
        self.block_on(async {
            let rows = sqlx::query(
                "SELECT id, name, province, commandery, x, y, scale, strategic_rank,
                        agriculture_base, commerce_base, defense_base, population_min,
                        population_max, confidence, notes
                 FROM cities
                 ORDER BY province, commandery, name",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)?;
            rows.into_iter().map(city_profile_from_row).collect()
        })
    }

    fn city_profile(&self, city_id: &str) -> Result<Option<CityProfile>, Self::Error> {
        self.block_on(async {
            let row = sqlx::query(
                "SELECT id, name, province, commandery, x, y, scale, strategic_rank,
                        agriculture_base, commerce_base, defense_base, population_min,
                        population_max, confidence, notes
                 FROM cities
                 WHERE id = ?1",
            )
            .bind(city_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)?;
            row.map(city_profile_from_row).transpose()
        })
    }
}

impl OfficerCatalog for SqliteHistoricalCatalog {
    type Error = HistoryDbError;

    fn officer_profiles(&self) -> Result<Vec<OfficerProfile>, Self::Error> {
        self.block_on(async {
            let rows = sqlx::query(
                "SELECT id, name, courtesy_name, native_place, birth_year, death_year,
                        gender, leadership, strength, intelligence, politics, charm, tags,
                        confidence, biography, notes
                 FROM officers
                 ORDER BY id",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)?;
            let mut profiles = rows
                .into_iter()
                .map(officer_profile_from_row)
                .collect::<Result<Vec<_>, _>>()?;
            for profile in &mut profiles {
                profile.relationships = officer_relationships(&self.pool, &profile.id).await?;
            }
            Ok(profiles)
        })
    }

    fn officer_profile(&self, officer_id: &str) -> Result<Option<OfficerProfile>, Self::Error> {
        self.block_on(async {
            let row = sqlx::query(
                "SELECT id, name, courtesy_name, native_place, birth_year, death_year,
                        gender, leadership, strength, intelligence, politics, charm, tags,
                        confidence, biography, notes
                 FROM officers
                 WHERE id = ?1",
            )
            .bind(officer_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)?;
            let Some(row) = row else {
                return Ok(None);
            };
            let mut profile = officer_profile_from_row(row)?;
            profile.relationships = officer_relationships(&self.pool, &profile.id).await?;
            Ok(Some(profile))
        })
    }
}

impl HistoricalCatalog for SqliteHistoricalCatalog {
    fn scenarios(&self) -> Result<Vec<HistoricalScenario>, HistoryDbError> {
        self.block_on(async {
            let rows =
                sqlx::query("SELECT id, name, year, month FROM scenarios ORDER BY year, month")
                    .fetch_all(&self.pool)
                    .await
                    .map_err(HistoryDbError::Sqlx)?;
            Ok(rows
                .into_iter()
                .map(|row| HistoricalScenario {
                    id: row.get("id"),
                    name: row.get("name"),
                    year: row.get("year"),
                    month: row.get::<i64, _>("month") as u8,
                })
                .collect())
        })
    }

    fn selectable_factions(&self, scenario_id: &str) -> Result<Vec<Faction>, HistoryDbError> {
        self.block_on(async {
            let rows = sqlx::query(
                "SELECT f.id, f.name, s.ruler_id, f.color_r, f.color_g, f.color_b, s.selectable
                 FROM scenario_faction_states s
                 JOIN factions f ON f.id = s.faction_id
                 WHERE s.scenario_id = ?1 AND s.exists_in_scenario = 1
                 ORDER BY f.id",
            )
            .bind(scenario_id)
            .fetch_all(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)?;
            Ok(rows
                .into_iter()
                .map(|row| Faction {
                    id: row.get("id"),
                    name: row.get("name"),
                    ruler_id: row.get("ruler_id"),
                    color: [row.get("color_r"), row.get("color_g"), row.get("color_b")],
                    selectable: row.get::<bool, _>("selectable"),
                    controlled_by: Controller::RuleAi,
                })
                .collect())
        })
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
        let city_rows = self.block_on(async {
            sqlx::query(
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
            .bind(scenario_id)
            .fetch_all(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)
        })?;
        for row in city_rows {
            let profile = CityProfile {
                id: row.get("id"),
                name: row.get("name"),
                province: row.get("province"),
                commandery: row.get("commandery"),
                position: MapPosition {
                    x: row.get("x"),
                    y: row.get("y"),
                },
                scale: parse_city_scale(row.get::<String, _>("scale").as_str()),
                strategic_rank: row.get::<i64, _>("strategic_rank") as u8,
                agriculture_base: row.get::<i64, _>("agriculture_base") as u16,
                commerce_base: row.get::<i64, _>("commerce_base") as u16,
                defense_base: row.get::<i64, _>("defense_base") as u16,
                population_min: row.get::<i64, _>("population_min") as u32,
                population_max: row.get::<i64, _>("population_max") as u32,
                confidence: parse_confidence(row.get::<String, _>("confidence").as_str()),
                notes: row.get("notes"),
            };
            let city = City {
                id: profile.id.clone(),
                name: profile.name.clone(),
                faction_id: row.get("faction_id"),
                position: profile.position,
                population: row.get::<i64, _>("population") as u32,
                gold: row.get::<i64, _>("gold") as i32,
                food: row.get::<i64, _>("food") as i32,
                troops: row.get::<i64, _>("troops") as u32,
                training: row.get::<i64, _>("training") as u8,
                agriculture: row.get::<i64, _>("agriculture") as u16,
                commerce: row.get::<i64, _>("commerce") as u16,
                defense: row.get::<i64, _>("defense") as u16,
                order: row.get::<i64, _>("city_order") as u8,
                governor_id: row.get("governor_id"),
                profile: Some(profile),
            };
            cities.insert(city.id.clone(), city);
        }

        let mut roads = Vec::new();
        let road_rows = self.block_on(async {
            sqlx::query(
                "SELECT from_city_id, to_city_id FROM roads ORDER BY from_city_id, to_city_id",
            )
            .fetch_all(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)
        })?;
        for row in road_rows {
            let road = Road {
                from: row.get("from_city_id"),
                to: row.get("to_city_id"),
            };
            if cities.contains_key(&road.from) && cities.contains_key(&road.to) {
                roads.push(road);
            }
        }

        let mut diplomacy = BTreeMap::new();
        let dip_rows = self.block_on(async {
            sqlx::query(
                "SELECT faction_a, faction_b, score, truce_until_turn
                 FROM scenario_diplomacy
                 WHERE scenario_id = ?1",
            )
            .bind(scenario_id)
            .fetch_all(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)
        })?;
        for row in dip_rows {
            let relation = DiplomaticRelation {
                faction_a: row.get("faction_a"),
                faction_b: row.get("faction_b"),
                score: row.get::<i64, _>("score") as i16,
                truce_until_turn: row
                    .get::<Option<i64>, _>("truce_until_turn")
                    .map(|value| value as u32),
            };
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
        self.block_on(async {
            let rows = sqlx::query(
                "SELECT id, officer_id, event_year, event_month, event_kind,
                        faction_id, city_id, loyalty, notes
                 FROM officer_life_events
                 WHERE event_year < ?1 OR (event_year = ?1 AND event_month <= ?2)
                 ORDER BY event_year, event_month, id",
            )
            .bind(year)
            .bind(month)
            .fetch_all(&self.pool)
            .await
            .map_err(HistoryDbError::Sqlx)?;
            rows.into_iter().map(life_event_from_row).collect()
        })
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

async fn officer_relationships(
    pool: &SqlitePool,
    officer_id: &str,
) -> Result<Vec<OfficerRelationship>, HistoryDbError> {
    let rows = sqlx::query(
        "SELECT r.target_officer_id, target.name AS target_name, r.relationship_kind,
                r.confidence, r.notes, r.source
         FROM officer_relationships r
         JOIN officers target ON target.id = r.target_officer_id
         WHERE r.source_officer_id = ?1
         ORDER BY r.relationship_kind, target.name",
    )
    .bind(officer_id)
    .fetch_all(pool)
    .await
    .map_err(HistoryDbError::Sqlx)?;
    rows.into_iter().map(relationship_from_row).collect()
}

fn city_profile_from_row(row: SqliteRow) -> Result<CityProfile, HistoryDbError> {
    Ok(CityProfile {
        id: row.get("id"),
        name: row.get("name"),
        province: row.get("province"),
        commandery: row.get("commandery"),
        position: MapPosition {
            x: row.get("x"),
            y: row.get("y"),
        },
        scale: parse_city_scale(row.get::<String, _>("scale").as_str()),
        strategic_rank: row.get::<i64, _>("strategic_rank") as u8,
        agriculture_base: row.get::<i64, _>("agriculture_base") as u16,
        commerce_base: row.get::<i64, _>("commerce_base") as u16,
        defense_base: row.get::<i64, _>("defense_base") as u16,
        population_min: row.get::<i64, _>("population_min") as u32,
        population_max: row.get::<i64, _>("population_max") as u32,
        confidence: parse_confidence(row.get::<String, _>("confidence").as_str()),
        notes: row.get("notes"),
    })
}

fn officer_profile_from_row(row: SqliteRow) -> Result<OfficerProfile, HistoryDbError> {
    Ok(OfficerProfile {
        id: row.get("id"),
        name: row.get("name"),
        courtesy_name: row.get("courtesy_name"),
        native_place: row.get("native_place"),
        birth_year: row.get("birth_year"),
        death_year: row.get("death_year"),
        gender: parse_gender(row.get::<String, _>("gender").as_str()),
        stats: OfficerStats {
            leadership: row.get::<i64, _>("leadership") as u8,
            strength: row.get::<i64, _>("strength") as u8,
            intelligence: row.get::<i64, _>("intelligence") as u8,
            politics: row.get::<i64, _>("politics") as u8,
            charm: row.get::<i64, _>("charm") as u8,
        },
        tags: row
            .get::<String, _>("tags")
            .split(',')
            .filter(|tag| !tag.is_empty())
            .map(str::to_string)
            .collect(),
        confidence: parse_confidence(row.get::<String, _>("confidence").as_str()),
        biography: row.get("biography"),
        relationships: Vec::new(),
        notes: row.get("notes"),
    })
}

fn relationship_from_row(row: SqliteRow) -> Result<OfficerRelationship, HistoryDbError> {
    Ok(OfficerRelationship {
        target_id: row.get("target_officer_id"),
        target_name: row.get("target_name"),
        kind: parse_relationship_kind(row.get::<String, _>("relationship_kind").as_str()),
        confidence: parse_confidence(row.get::<String, _>("confidence").as_str()),
        notes: row.get("notes"),
        source: row.get("source"),
    })
}

fn life_event_from_row(row: SqliteRow) -> Result<LifeEvent, HistoryDbError> {
    Ok(LifeEvent {
        id: row.get("id"),
        officer_id: row.get("officer_id"),
        year: row.get("event_year"),
        month: row.get::<i64, _>("event_month") as u8,
        kind: parse_life_event_kind(row.get::<String, _>("event_kind").as_str()),
        faction_id: row.get("faction_id"),
        city_id: row.get("city_id"),
        loyalty: row
            .get::<Option<i64>, _>("loyalty")
            .map(|value| value as u8),
        notes: row.get("notes"),
    })
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
    Runtime(std::io::Error),
    Sqlx(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
    Invalid(String),
}

impl std::fmt::Display for HistoryDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryDbError::Io(error) => write!(f, "历史资料库 IO 失败: {error}"),
            HistoryDbError::Runtime(error) => write!(f, "历史资料库运行时初始化失败: {error}"),
            HistoryDbError::Sqlx(error) => write!(f, "历史资料库 SQLx 失败: {error}"),
            HistoryDbError::Migrate(error) => write!(f, "历史资料库迁移失败: {error}"),
            HistoryDbError::Invalid(message) => write!(f, "历史资料库数据无效: {message}"),
        }
    }
}

impl std::error::Error for HistoryDbError {}
