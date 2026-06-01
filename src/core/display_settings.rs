use crate::ai::{BAILIAN_DEFAULT_IMAGE_MODEL, OPENAI_DEFAULT_API_URL, OpenAiApiType};
use bevy::window::{
    MonitorSelection, PresentMode, VideoModeSelection, Window, WindowMode, WindowResolution,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::i18n::{Translator, UiLanguage, args};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(super) struct GameSettings {
    pub(super) general: GeneralSettings,
    pub(super) display: DisplaySettings,
    pub(super) audio: AudioSettings,
    pub(super) ai: AiSettings,
}

impl GameSettings {
    pub(super) fn validated(self) -> Result<Self, GameSettingsError> {
        Ok(Self {
            general: self.general,
            display: self.display.validate()?,
            audio: self.audio.normalized(),
            ai: self.ai.normalized()?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct GeneralSettings {
    pub(super) ui_language: UiLanguage,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(super) struct AudioSettings {
    pub(super) master_volume: f32,
    pub(super) output_device_name: Option<String>,
}

impl AudioSettings {
    pub(super) fn normalized(mut self) -> Self {
        self.master_volume = normalize_master_volume(self.master_volume);
        self.output_device_name = normalize_output_device_name(self.output_device_name);
        self
    }
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            output_device_name: None,
        }
    }
}

pub(super) fn normalize_master_volume(volume: f32) -> f32 {
    if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        AudioSettings::default().master_volume
    }
}

pub(super) fn normalize_output_device_name(device_name: Option<String>) -> Option<String> {
    device_name.and_then(|name| {
        let trimmed = name.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct AiSettings {
    pub(super) reasoning: ReasoningAiSettings,
    pub(super) multimodal: MultimodalAiSettings,
}

impl AiSettings {
    pub(super) fn normalized(self) -> Result<Self, GameSettingsError> {
        Ok(Self {
            reasoning: self.reasoning.normalized()?,
            multimodal: self.multimodal.normalized(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct ReasoningAiSettings {
    pub(super) api_type: OpenAiApiType,
    pub(super) api_url: String,
    pub(super) token: String,
    pub(super) model_name: String,
}

impl ReasoningAiSettings {
    fn normalized(mut self) -> Result<Self, GameSettingsError> {
        self.api_url = self.api_url.trim().to_string();
        self.token = self.token.trim().to_string();
        self.model_name = self.model_name.trim().to_string();
        if !self.api_url.is_empty() {
            let parsed = reqwest::Url::parse(&self.api_url).map_err(|_| {
                GameSettingsError::Invalid("OpenAI API URL 必须是完整 URL".to_string())
            })?;
            if !matches!(parsed.scheme(), "http" | "https") {
                return Err(GameSettingsError::Invalid(
                    "OpenAI API URL 必须使用 http 或 https".to_string(),
                ));
            }
        }
        Ok(self)
    }
}

impl Default for ReasoningAiSettings {
    fn default() -> Self {
        Self {
            api_type: OpenAiApiType::default(),
            api_url: OPENAI_DEFAULT_API_URL.to_string(),
            token: String::new(),
            model_name: String::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct MultimodalAiSettings {
    pub(super) api_key: String,
    pub(super) model_name: String,
}

impl MultimodalAiSettings {
    fn normalized(mut self) -> Self {
        self.api_key = self.api_key.trim().to_string();
        self.model_name = self.model_name.trim().to_string();
        self
    }
}

impl Default for MultimodalAiSettings {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model_name: BAILIAN_DEFAULT_IMAGE_MODEL.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct DisplaySettings {
    pub(super) resolution: DisplayResolution,
    pub(super) display_mode: DisplayMode,
    pub(super) vsync: bool,
}

impl DisplaySettings {
    pub(super) fn window_resolution(self) -> WindowResolution {
        WindowResolution::new(self.resolution.width, self.resolution.height)
    }

    pub(super) fn window_mode(self) -> WindowMode {
        match self.display_mode {
            DisplayMode::Windowed => WindowMode::Windowed,
            DisplayMode::BorderlessFullscreen => {
                WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
            }
            DisplayMode::ExclusiveFullscreen => {
                WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current)
            }
        }
    }

    pub(super) fn present_mode(self) -> PresentMode {
        if self.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        }
    }

    pub(super) fn apply_to_window(self, window: &mut Window) {
        window.resizable = false;
        window.enabled_buttons.maximize = false;
        window.mode = self.window_mode();
        window.present_mode = self.present_mode();
        if self.display_mode == DisplayMode::Windowed {
            window
                .resolution
                .set(self.resolution.width as f32, self.resolution.height as f32);
        }
    }

    fn validate(self) -> Result<Self, GameSettingsError> {
        if !self.resolution.is_preset() {
            return Err(GameSettingsError::Invalid(format!(
                "不支持的分辨率 {}",
                self.resolution
            )));
        }
        Ok(self)
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            resolution: DisplayResolution::new(1280, 820),
            display_mode: DisplayMode::Windowed,
            vsync: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct DisplayResolution {
    pub(super) width: u32,
    pub(super) height: u32,
}

impl DisplayResolution {
    pub(super) const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub(super) fn presets() -> &'static [Self] {
        &DISPLAY_RESOLUTION_PRESETS
    }

    fn is_preset(self) -> bool {
        DISPLAY_RESOLUTION_PRESETS.contains(&self)
    }
}

impl Default for DisplayResolution {
    fn default() -> Self {
        DisplaySettings::default().resolution
    }
}

impl std::fmt::Display for DisplayResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

const DISPLAY_RESOLUTION_PRESETS: [DisplayResolution; 7] = [
    DisplayResolution::new(1024, 768),
    DisplayResolution::new(1280, 720),
    DisplayResolution::new(1280, 820),
    DisplayResolution::new(1366, 768),
    DisplayResolution::new(1600, 900),
    DisplayResolution::new(1920, 1080),
    DisplayResolution::new(2560, 1440),
];

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum DisplayMode {
    #[default]
    Windowed,
    BorderlessFullscreen,
    ExclusiveFullscreen,
}

impl DisplayMode {
    pub(super) fn variants() -> &'static [Self] {
        &DISPLAY_MODE_VARIANTS
    }

    pub(super) fn label(self, t: &Translator) -> String {
        match self {
            DisplayMode::Windowed => t.text("display-mode-windowed"),
            DisplayMode::BorderlessFullscreen => t.text("display-mode-borderless-fullscreen"),
            DisplayMode::ExclusiveFullscreen => t.text("display-mode-exclusive-fullscreen"),
        }
    }
}

const DISPLAY_MODE_VARIANTS: [DisplayMode; 3] = [
    DisplayMode::Windowed,
    DisplayMode::BorderlessFullscreen,
    DisplayMode::ExclusiveFullscreen,
];

#[derive(Clone, Debug)]
pub(super) struct GameSettingsStore {
    path: PathBuf,
}

impl GameSettingsStore {
    pub(super) fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub(super) fn default_path() -> PathBuf {
        ProjectDirs::from("", "", "Shogun")
            .map(|dirs| dirs.config_dir().join("settings.json"))
            .unwrap_or_else(|| PathBuf::from(".shogun_settings.json"))
    }

    pub(super) fn with_default_path() -> Self {
        Self::new(Self::default_path())
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn load(&self) -> LoadedGameSettings {
        match fs::read_to_string(&self.path) {
            Ok(body) => match parse_game_settings(&body) {
                Ok(settings) => LoadedGameSettings::loaded(settings),
                Err(error) => LoadedGameSettings::fallback(GameSettingsLoadMessage::Invalid {
                    error: error.to_string(),
                }),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                LoadedGameSettings::fallback(GameSettingsLoadMessage::Missing)
            }
            Err(error) => LoadedGameSettings::fallback(GameSettingsLoadMessage::ReadFailed {
                error: error.to_string(),
            }),
        }
    }

    pub(super) fn save(&self, settings: GameSettings) -> Result<(), GameSettingsError> {
        let settings = settings.validated()?;
        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(GameSettingsError::Io)?;
        }
        let body = serde_json::to_string_pretty(&settings).map_err(GameSettingsError::Json)?;
        fs::write(&self.path, body).map_err(GameSettingsError::Io)
    }
}

impl Default for GameSettingsStore {
    fn default() -> Self {
        Self::with_default_path()
    }
}

fn parse_game_settings(body: &str) -> Result<GameSettings, GameSettingsError> {
    let value = serde_json::from_str::<serde_json::Value>(body).map_err(GameSettingsError::Json)?;
    let settings = if value.get("general").is_some()
        || value.get("display").is_some()
        || value.get("audio").is_some()
        || value.get("ai").is_some()
    {
        serde_json::from_value::<GameSettings>(value).map_err(GameSettingsError::Json)?
    } else {
        let display =
            serde_json::from_value::<DisplaySettings>(value).map_err(GameSettingsError::Json)?;
        GameSettings {
            general: GeneralSettings::default(),
            display,
            audio: AudioSettings::default(),
            ai: AiSettings::default(),
        }
    };
    settings.validated()
}

#[derive(Clone, Debug)]
pub(super) struct LoadedGameSettings {
    pub(super) settings: GameSettings,
    pub(super) message: Option<GameSettingsLoadMessage>,
}

impl LoadedGameSettings {
    fn loaded(settings: GameSettings) -> Self {
        Self {
            settings,
            message: None,
        }
    }

    fn fallback(message: GameSettingsLoadMessage) -> Self {
        Self {
            settings: GameSettings::default(),
            message: Some(message),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum GameSettingsLoadMessage {
    Invalid { error: String },
    Missing,
    ReadFailed { error: String },
}

impl GameSettingsLoadMessage {
    pub(super) fn localized(&self, t: &Translator) -> String {
        match self {
            Self::Invalid { error } => t.text_args(
                "message-settings-invalid-defaulted",
                &args([("error", error.clone())]),
            ),
            Self::Missing => t.text("message-settings-missing-defaulted"),
            Self::ReadFailed { error } => t.text_args(
                "message-settings-read-failed-defaulted",
                &args([("error", error.clone())]),
            ),
        }
    }
}

#[derive(Debug)]
pub(super) enum GameSettingsError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl std::fmt::Display for GameSettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "设置 IO 失败: {error}"),
            Self::Json(error) => write!(f, "设置 JSON 失败: {error}"),
            Self::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for GameSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_windowed_1280_by_820_with_vsync() {
        let settings = DisplaySettings::default();

        assert_eq!(settings.resolution, DisplayResolution::new(1280, 820));
        assert_eq!(settings.display_mode, DisplayMode::Windowed);
        assert!(settings.vsync);
    }

    #[test]
    fn settings_round_trip_through_json() {
        let temp = tempfile::tempdir().unwrap();
        let store = GameSettingsStore::new(temp.path().join("settings.json"));
        let settings = GameSettings {
            general: GeneralSettings {
                ui_language: UiLanguage::English,
            },
            display: DisplaySettings {
                resolution: DisplayResolution::new(1600, 900),
                display_mode: DisplayMode::BorderlessFullscreen,
                vsync: false,
            },
            audio: AudioSettings {
                master_volume: 0.42,
                output_device_name: Some("Built-in Output".to_string()),
            },
            ai: AiSettings {
                reasoning: ReasoningAiSettings {
                    api_url: "https://proxy.example/v1/responses".to_string(),
                    token: "test-token".to_string(),
                    model_name: "reasoning-model".to_string(),
                },
                multimodal: MultimodalAiSettings {
                    api_key: "dashscope-key".to_string(),
                    model_name: "wan2.7-image-pro".to_string(),
                },
            },
        };

        store.save(settings.clone()).unwrap();
        let loaded = store.load();

        assert_eq!(loaded.settings, settings);
        assert!(loaded.message.is_none());
    }

    #[test]
    fn legacy_display_settings_json_loads_with_default_audio() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "resolution": { "width": 1600, "height": 900 },
  "display_mode": "borderless_fullscreen",
  "vsync": false
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(
            loaded.settings.display,
            DisplaySettings {
                resolution: DisplayResolution::new(1600, 900),
                display_mode: DisplayMode::BorderlessFullscreen,
                vsync: false,
            }
        );
        assert_eq!(loaded.settings.audio, AudioSettings::default());
        assert_eq!(loaded.settings.general, GeneralSettings::default());
        assert_eq!(loaded.settings.ai, AiSettings::default());
        assert!(loaded.message.is_none());
    }

    #[test]
    fn nested_display_audio_settings_load_with_default_language() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "display": {
    "resolution": { "width": 1366, "height": 768 },
    "display_mode": "windowed",
    "vsync": false
  },
  "audio": {
    "master_volume": 0.5,
    "output_device_name": "Built-in Output"
  }
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(loaded.settings.general, GeneralSettings::default());
        assert_eq!(
            loaded.settings.display.resolution,
            DisplayResolution::new(1366, 768)
        );
        assert_eq!(loaded.settings.audio.master_volume, 0.5);
        assert_eq!(loaded.settings.ai, AiSettings::default());
        assert!(loaded.message.is_none());
    }

    #[test]
    fn ai_settings_load_and_trim_api_values() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "ai": {
    "reasoning": {
      "api_url": " https://proxy.example/v1/responses ",
      "token": " token-value ",
      "model_name": " reasoning-model "
    },
    "multimodal": {
      "api_key": " bailian-key ",
      "model_name": " wan2.7-image-pro "
    }
  }
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(
            loaded.settings.ai.reasoning.api_url,
            "https://proxy.example/v1/responses"
        );
        assert_eq!(loaded.settings.ai.reasoning.token, "token-value");
        assert_eq!(loaded.settings.ai.reasoning.model_name, "reasoning-model");
        assert_eq!(loaded.settings.ai.multimodal.api_key, "bailian-key");
        assert_eq!(loaded.settings.ai.multimodal.model_name, "wan2.7-image-pro");
        assert!(loaded.message.is_none());
    }

    #[test]
    fn general_ui_language_loads_from_settings() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "general": { "ui_language": "en-US" },
  "display": {
    "resolution": { "width": 1280, "height": 820 },
    "display_mode": "windowed",
    "vsync": true
  },
  "audio": {
    "master_volume": 1.0,
    "output_device_name": null
  }
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(loaded.settings.general.ui_language, UiLanguage::English);
        assert!(loaded.message.is_none());
    }

    #[test]
    fn missing_settings_file_falls_back_to_default() {
        let temp = tempfile::tempdir().unwrap();
        let store = GameSettingsStore::new(temp.path().join("missing.json"));

        let loaded = store.load();

        assert_eq!(loaded.settings, GameSettings::default());
        assert!(loaded.message.is_some());
    }

    #[test]
    fn invalid_json_falls_back_to_default() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(&path, "{invalid json").unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(loaded.settings, GameSettings::default());
        assert!(loaded.message.is_some());
    }

    #[test]
    fn unsupported_resolution_falls_back_to_default() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "resolution": { "width": 1111, "height": 777 },
  "display_mode": "windowed",
  "vsync": true
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(loaded.settings, GameSettings::default());
        assert!(loaded.message.is_some());
    }

    #[test]
    fn audio_settings_normalize_volume_and_empty_device_name() {
        let settings = AudioSettings {
            master_volume: 1.4,
            output_device_name: Some("  ".to_string()),
        }
        .normalized();

        assert_eq!(settings.master_volume, 1.0);
        assert_eq!(settings.output_device_name, None);

        let settings = AudioSettings {
            master_volume: -0.25,
            output_device_name: Some("  External DAC  ".to_string()),
        }
        .normalized();

        assert_eq!(settings.master_volume, 0.0);
        assert_eq!(
            settings.output_device_name,
            Some("External DAC".to_string())
        );
    }

    #[test]
    fn settings_map_to_bevy_window_types() {
        let mut settings = DisplaySettings {
            resolution: DisplayResolution::new(1920, 1080),
            display_mode: DisplayMode::Windowed,
            vsync: false,
        };

        let resolution = settings.window_resolution();
        assert_eq!(resolution.physical_width(), 1920);
        assert_eq!(resolution.physical_height(), 1080);
        assert!(matches!(settings.window_mode(), WindowMode::Windowed));
        assert_eq!(settings.present_mode(), PresentMode::AutoNoVsync);

        settings.display_mode = DisplayMode::BorderlessFullscreen;
        assert!(matches!(
            settings.window_mode(),
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        ));

        settings.display_mode = DisplayMode::ExclusiveFullscreen;
        assert!(matches!(
            settings.window_mode(),
            WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current)
        ));
    }

    #[test]
    fn applying_windowed_settings_sets_size_and_locks_resizing() {
        let settings = DisplaySettings {
            resolution: DisplayResolution::new(1600, 900),
            display_mode: DisplayMode::Windowed,
            vsync: true,
        };
        let mut window = Window {
            resolution: WindowResolution::new(1024, 768),
            resizable: true,
            enabled_buttons: bevy::window::EnabledButtons {
                maximize: true,
                ..Default::default()
            },
            ..Default::default()
        };

        settings.apply_to_window(&mut window);

        assert!(!window.resizable);
        assert!(!window.enabled_buttons.maximize);
        assert_eq!(window.resolution.physical_width(), 1600);
        assert_eq!(window.resolution.physical_height(), 900);
    }

    #[test]
    fn applying_windowed_settings_preserves_logical_size_on_scaled_displays() {
        let settings = DisplaySettings {
            resolution: DisplayResolution::new(1600, 900),
            display_mode: DisplayMode::Windowed,
            vsync: true,
        };
        let mut window = Window {
            resolution: WindowResolution::new(1024, 768),
            ..Default::default()
        };
        window.resolution.set_scale_factor(2.0);

        settings.apply_to_window(&mut window);

        assert_eq!(window.resolution.width(), 1600.0);
        assert_eq!(window.resolution.height(), 900.0);
        assert_eq!(window.resolution.physical_width(), 3200);
        assert_eq!(window.resolution.physical_height(), 1800);
    }
}
