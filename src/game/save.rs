use super::ids::FactionId;
use super::model::*;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct SaveManager {
    base_dir: PathBuf,
}

impl SaveManager {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn default_dir() -> PathBuf {
        ProjectDirs::from("", "", "Shogun")
            .map(|dirs| dirs.data_local_dir().join("saves"))
            .unwrap_or_else(|| PathBuf::from(".shogun_saves"))
    }

    pub fn with_default_dir() -> Self {
        Self::new(Self::default_dir())
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn list_slots(&self) -> Result<Vec<SaveSlotMeta>, SaveError> {
        let mut slots = self.read_manifest()?.slots;
        slots.sort_by_key(|slot| std::cmp::Reverse(slot.saved_at_unix));
        Ok(slots)
    }

    pub fn save_slot(
        &self,
        slot_id: &str,
        display_name: &str,
        state: &GameState,
    ) -> Result<SaveSlotMeta, SaveError> {
        validate_slot_id(slot_id)?;
        fs::create_dir_all(self.slots_dir()).map_err(SaveError::Io)?;
        let meta = SaveSlotMeta {
            slot_id: slot_id.to_string(),
            display_name: display_name.to_string(),
            scenario_id: state.scenario_id.clone(),
            scenario_name: state.scenario_name.clone(),
            player_faction_id: state.player_faction_id.clone(),
            turn: state.turn,
            year: state.year,
            month: state.month,
            saved_at_unix: now_unix(),
        };

        let envelope = SaveEnvelope {
            version: SAVE_VERSION,
            meta: meta.clone(),
            state: state.clone(),
        };
        let body = serde_json::to_string_pretty(&envelope).map_err(SaveError::Json)?;
        fs::write(self.slot_path(slot_id), body).map_err(SaveError::Io)?;

        let mut manifest = self.read_manifest()?;
        manifest.slots.retain(|slot| slot.slot_id != slot_id);
        manifest.slots.push(meta.clone());
        self.write_manifest(&manifest)?;
        Ok(meta)
    }

    pub fn load_slot(&self, slot_id: &str) -> Result<GameState, SaveError> {
        validate_slot_id(slot_id)?;
        let body = fs::read_to_string(self.slot_path(slot_id)).map_err(SaveError::Io)?;
        let envelope: SaveEnvelope = serde_json::from_str(&body).map_err(SaveError::Json)?;
        if envelope.version != SAVE_VERSION {
            return Err(SaveError::Version {
                expected: SAVE_VERSION,
                found: envelope.version,
            });
        }
        Ok(envelope.state)
    }

    pub fn delete_slot(&self, slot_id: &str) -> Result<(), SaveError> {
        validate_slot_id(slot_id)?;
        let path = self.slot_path(slot_id);
        if path.exists() {
            fs::remove_file(path).map_err(SaveError::Io)?;
        }
        let mut manifest = self.read_manifest()?;
        manifest.slots.retain(|slot| slot.slot_id != slot_id);
        self.write_manifest(&manifest)
    }

    fn read_manifest(&self) -> Result<SaveSlotManifest, SaveError> {
        let path = self.manifest_path();
        if !path.exists() {
            return Ok(SaveSlotManifest::default());
        }
        let body = fs::read_to_string(path).map_err(SaveError::Io)?;
        serde_json::from_str(&body).map_err(SaveError::Json)
    }

    fn write_manifest(&self, manifest: &SaveSlotManifest) -> Result<(), SaveError> {
        fs::create_dir_all(&self.base_dir).map_err(SaveError::Io)?;
        let body = serde_json::to_string_pretty(manifest).map_err(SaveError::Json)?;
        fs::write(self.manifest_path(), body).map_err(SaveError::Io)
    }

    fn manifest_path(&self) -> PathBuf {
        self.base_dir.join("manifest.json")
    }

    fn slots_dir(&self) -> PathBuf {
        self.base_dir.join("slots")
    }

    fn slot_path(&self, slot_id: &str) -> PathBuf {
        self.slots_dir().join(format!("{slot_id}.json"))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveSlotManifest {
    pub slots: Vec<SaveSlotMeta>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SaveSlotMeta {
    pub slot_id: String,
    pub display_name: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub player_faction_id: FactionId,
    pub turn: u32,
    pub year: i32,
    pub month: u8,
    pub saved_at_unix: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct SaveEnvelope {
    version: u32,
    meta: SaveSlotMeta,
    state: GameState,
}

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidSlotId(String),
    Version { expected: u32, found: u32 },
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(error) => write!(f, "存档 IO 失败: {error}"),
            SaveError::Json(error) => write!(f, "存档 JSON 失败: {error}"),
            SaveError::InvalidSlotId(slot_id) => write!(f, "非法存档槽位: {slot_id}"),
            SaveError::Version { expected, found } => {
                write!(f, "存档版本不匹配: 需要 {expected}, 实际 {found}")
            }
        }
    }
}

impl std::error::Error for SaveError {}

fn validate_slot_id(slot_id: &str) -> Result<(), SaveError> {
    let valid = !slot_id.is_empty()
        && slot_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(SaveError::InvalidSlotId(slot_id.to_string()))
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
