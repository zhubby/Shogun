use bevy::window::{
    MonitorSelection, PresentMode, VideoModeSelection, Window, WindowMode, WindowResolution,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(super) struct GameSettings {
    pub(super) display: DisplaySettings,
    pub(super) audio: AudioSettings,
}

impl GameSettings {
    pub(super) fn validated(self) -> Result<Self, GameSettingsError> {
        Ok(Self {
            display: self.display.validate()?,
            audio: self.audio.normalized(),
        })
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            display: DisplaySettings::default(),
            audio: AudioSettings::default(),
        }
    }
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
                .set_physical_resolution(self.resolution.width, self.resolution.height);
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

    pub(super) fn label(self) -> &'static str {
        match self {
            DisplayMode::Windowed => "窗口",
            DisplayMode::BorderlessFullscreen => "无边框全屏",
            DisplayMode::ExclusiveFullscreen => "独占全屏",
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
                Err(error) => {
                    LoadedGameSettings::fallback(format!("游戏设置无效，已使用默认设置: {error}"))
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                LoadedGameSettings::fallback("未找到游戏设置，已使用默认设置".to_string())
            }
            Err(error) => {
                LoadedGameSettings::fallback(format!("读取游戏设置失败，已使用默认设置: {error}"))
            }
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
    let settings = if value.get("display").is_some() || value.get("audio").is_some() {
        serde_json::from_value::<GameSettings>(value).map_err(GameSettingsError::Json)?
    } else {
        let display =
            serde_json::from_value::<DisplaySettings>(value).map_err(GameSettingsError::Json)?;
        GameSettings {
            display,
            audio: AudioSettings::default(),
        }
    };
    settings.validated()
}

#[derive(Clone, Debug)]
pub(super) struct LoadedGameSettings {
    pub(super) settings: GameSettings,
    pub(super) message: Option<String>,
}

impl LoadedGameSettings {
    fn loaded(settings: GameSettings) -> Self {
        Self {
            settings,
            message: None,
        }
    }

    fn fallback(message: String) -> Self {
        Self {
            settings: GameSettings::default(),
            message: Some(message),
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
            display: DisplaySettings {
                resolution: DisplayResolution::new(1600, 900),
                display_mode: DisplayMode::BorderlessFullscreen,
                vsync: false,
            },
            audio: AudioSettings {
                master_volume: 0.42,
                output_device_name: Some("Built-in Output".to_string()),
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
}
