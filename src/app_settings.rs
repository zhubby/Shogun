use bevy::window::{
    MonitorSelection, PresentMode, VideoModeSelection, Window, WindowMode, WindowResolution,
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(crate) struct AppSettings {
    pub(crate) resolution: DisplayResolution,
    pub(crate) display_mode: DisplayMode,
    pub(crate) vsync: bool,
}

impl AppSettings {
    pub(crate) fn window_resolution(self) -> WindowResolution {
        WindowResolution::new(self.resolution.width, self.resolution.height)
    }

    pub(crate) fn window_mode(self) -> WindowMode {
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

    pub(crate) fn present_mode(self) -> PresentMode {
        if self.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        }
    }

    pub(crate) fn apply_to_window(self, window: &mut Window) {
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

    fn validate(self) -> Result<Self, AppSettingsError> {
        if !self.resolution.is_preset() {
            return Err(AppSettingsError::Invalid(format!(
                "不支持的分辨率 {}",
                self.resolution
            )));
        }
        Ok(self)
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            resolution: DisplayResolution::new(1280, 820),
            display_mode: DisplayMode::Windowed,
            vsync: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DisplayResolution {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl DisplayResolution {
    pub(crate) const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub(crate) fn presets() -> &'static [Self] {
        &DISPLAY_RESOLUTION_PRESETS
    }

    fn is_preset(self) -> bool {
        DISPLAY_RESOLUTION_PRESETS.contains(&self)
    }
}

impl Default for DisplayResolution {
    fn default() -> Self {
        AppSettings::default().resolution
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
pub(crate) enum DisplayMode {
    #[default]
    Windowed,
    BorderlessFullscreen,
    ExclusiveFullscreen,
}

impl DisplayMode {
    pub(crate) fn variants() -> &'static [Self] {
        &DISPLAY_MODE_VARIANTS
    }

    pub(crate) fn label(self) -> &'static str {
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
pub(crate) struct AppSettingsStore {
    path: PathBuf,
}

impl AppSettingsStore {
    pub(crate) fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub(crate) fn default_path() -> PathBuf {
        ProjectDirs::from("dev", "zhubby", "Shogun")
            .map(|dirs| dirs.config_dir().join("settings.json"))
            .unwrap_or_else(|| PathBuf::from(".shogun_settings.json"))
    }

    pub(crate) fn with_default_path() -> Self {
        Self::new(Self::default_path())
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn load(&self) -> LoadedAppSettings {
        match fs::read_to_string(&self.path) {
            Ok(body) => match serde_json::from_str::<AppSettings>(&body) {
                Ok(settings) => match settings.validate() {
                    Ok(settings) => LoadedAppSettings::loaded(settings),
                    Err(error) => LoadedAppSettings::fallback(format!(
                        "显示设置无效，已使用默认设置: {error}"
                    )),
                },
                Err(error) => LoadedAppSettings::fallback(format!(
                    "显示设置格式无效，已使用默认设置: {error}"
                )),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                LoadedAppSettings::fallback("未找到显示设置，已使用默认设置".to_string())
            }
            Err(error) => {
                LoadedAppSettings::fallback(format!("读取显示设置失败，已使用默认设置: {error}"))
            }
        }
    }

    pub(crate) fn save(&self, settings: AppSettings) -> Result<(), AppSettingsError> {
        let settings = settings.validate()?;
        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(AppSettingsError::Io)?;
        }
        let body = serde_json::to_string_pretty(&settings).map_err(AppSettingsError::Json)?;
        fs::write(&self.path, body).map_err(AppSettingsError::Io)
    }
}

impl Default for AppSettingsStore {
    fn default() -> Self {
        Self::with_default_path()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedAppSettings {
    pub(crate) settings: AppSettings,
    pub(crate) message: Option<String>,
}

impl LoadedAppSettings {
    fn loaded(settings: AppSettings) -> Self {
        Self {
            settings,
            message: None,
        }
    }

    fn fallback(message: String) -> Self {
        Self {
            settings: AppSettings::default(),
            message: Some(message),
        }
    }
}

#[derive(Debug)]
pub(crate) enum AppSettingsError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl std::fmt::Display for AppSettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "设置 IO 失败: {error}"),
            Self::Json(error) => write!(f, "设置 JSON 失败: {error}"),
            Self::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_windowed_1280_by_820_with_vsync() {
        let settings = AppSettings::default();

        assert_eq!(settings.resolution, DisplayResolution::new(1280, 820));
        assert_eq!(settings.display_mode, DisplayMode::Windowed);
        assert!(settings.vsync);
    }

    #[test]
    fn settings_round_trip_through_json() {
        let temp = tempfile::tempdir().unwrap();
        let store = AppSettingsStore::new(temp.path().join("settings.json"));
        let settings = AppSettings {
            resolution: DisplayResolution::new(1600, 900),
            display_mode: DisplayMode::BorderlessFullscreen,
            vsync: false,
        };

        store.save(settings).unwrap();
        let loaded = store.load();

        assert_eq!(loaded.settings, settings);
        assert!(loaded.message.is_none());
    }

    #[test]
    fn missing_settings_file_falls_back_to_default() {
        let temp = tempfile::tempdir().unwrap();
        let store = AppSettingsStore::new(temp.path().join("missing.json"));

        let loaded = store.load();

        assert_eq!(loaded.settings, AppSettings::default());
        assert!(loaded.message.is_some());
    }

    #[test]
    fn invalid_json_falls_back_to_default() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(&path, "{invalid json").unwrap();
        let store = AppSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(loaded.settings, AppSettings::default());
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
        let store = AppSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(loaded.settings, AppSettings::default());
        assert!(loaded.message.is_some());
    }

    #[test]
    fn settings_map_to_bevy_window_types() {
        let mut settings = AppSettings {
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
        let settings = AppSettings {
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
