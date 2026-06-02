use crate::ai::{BAILIAN_DEFAULT_IMAGE_MODEL, OPENAI_DEFAULT_API_URL, OpenAiApiType};
use bevy::window::{
    MonitorSelection, PresentMode, VideoModeSelection, Window, WindowMode, WindowResolution,
};
use bevy_egui::egui;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::HUD_MARGIN;
use super::audio::{MainMenuAudio, available_output_device_names};
use super::i18n::{Translator, UiLanguage, args};
use super::shortcuts::{ShortcutAction, ShortcutBinding, ShortcutSettings, shortcut_groups};
use super::state::{GameUiState, SettingsTab};
use super::style::{modal_title_bar, war_gold, war_panel_frame, war_text_muted, war_warning};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub(super) struct GameSettings {
    pub(super) general: GeneralSettings,
    pub(super) display: DisplaySettings,
    pub(super) audio: AudioSettings,
    pub(super) gameplay: GameplaySettings,
    pub(super) shortcuts: ShortcutSettings,
    pub(super) ai: AiSettings,
}

impl GameSettings {
    pub(super) fn validated(self) -> Result<Self, GameSettingsError> {
        Ok(Self {
            general: self.general,
            display: self.display.validate()?,
            audio: self.audio.normalized(),
            gameplay: self.gameplay,
            shortcuts: self
                .shortcuts
                .validated()
                .map_err(GameSettingsError::Invalid)?,
            ai: self.ai.normalized()?,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct GeneralSettings {
    pub(super) ui_language: UiLanguage,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct GameplaySettings {
    pub(super) autosave_enabled: bool,
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
        || value.get("gameplay").is_some()
        || value.get("shortcuts").is_some()
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
            gameplay: GameplaySettings::default(),
            shortcuts: ShortcutSettings::default(),
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

#[derive(Debug, thiserror::Error)]
pub(super) enum GameSettingsError {
    #[error("设置 IO 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("设置 JSON 失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}

pub(super) fn settings_modal(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
    let t = Translator::new(ui_state.pending_settings.general.ui_language);
    if !ui_state.audio_output_devices_refresh_attempted {
        refresh_audio_output_devices(ui_state);
    }

    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::new("settings_modal_scrim"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
            );
            if response.clicked() {
                ui_state.settings_open = false;
            }
        });

    let mut apply_settings = false;
    let modal_width = (screen.width() - HUD_MARGIN * 2.0).clamp(340.0, 720.0);
    egui::Area::new(egui::Id::new("settings_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                if modal_title_bar(ui, &t, &t.text("settings-title")) {
                    ui_state.settings_open = false;
                }
                ui.separator();
                apply_settings |= settings_controls(ui, ui_state, &t);
            });
        });
    apply_settings
}

pub(super) fn refresh_audio_output_devices(ui_state: &mut GameUiState) {
    ui_state.audio_output_devices_refresh_attempted = true;
    match available_output_device_names() {
        Ok(devices) => {
            ui_state.audio_output_devices = devices;
            ui_state.audio_output_devices_error = None;
        }
        Err(error) => {
            ui_state.audio_output_devices.clear();
            ui_state.audio_output_devices_error = Some(error);
        }
    }
}

pub(super) fn settings_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
) -> bool {
    let mut apply_settings = false;
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(t.text_args(
                "settings-config-path",
                &args([("path", ui_state.settings_store.path().display().to_string())]),
            ))
            .color(war_text_muted()),
        );
        ui.add_space(8.0);

        settings_tabs(ui, ui_state, t);
        ui.add_space(10.0);

        match ui_state.settings_tab {
            SettingsTab::Display => display_settings_controls(ui, ui_state, t),
            SettingsTab::Audio => audio_settings_controls(ui, ui_state, t),
            SettingsTab::Language => language_settings_controls(ui, ui_state, t),
            SettingsTab::Gameplay => gameplay_settings_controls(ui, ui_state, t),
            SettingsTab::Shortcuts => shortcut_settings_controls(ui, ui_state, t),
            SettingsTab::Ai => ai_settings_controls(ui, ui_state, t),
        }

        if ui_state.pending_settings != ui_state.applied_settings {
            ui.add_space(8.0);
            ui.colored_label(war_gold(), t.text("settings-unsaved-changes"));
        }

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            let footer_button_size = egui::vec2(132.0, 34.0);
            if ui
                .add_sized(
                    footer_button_size,
                    egui::Button::new(t.text("settings-apply-save")),
                )
                .clicked()
            {
                apply_settings = true;
                ui_state.settings_open = false;
            }
            if ui
                .add_sized(
                    footer_button_size,
                    egui::Button::new(t.text("settings-restore-defaults")),
                )
                .clicked()
            {
                ui_state.pending_settings = GameSettings::default();
                ui_state.shortcut_capture_action = None;
                ui_state.message = t.text("message-settings-restored");
            }
        });
    });
    apply_settings
}

fn settings_tabs(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal(|ui| {
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Display,
            t.text("settings-tab-display"),
        );
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Audio,
            t.text("settings-tab-audio"),
        );
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Language,
            t.text("settings-tab-language"),
        );
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Gameplay,
            t.text("settings-tab-gameplay"),
        );
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Shortcuts,
            t.text("settings-tab-shortcuts"),
        );
        ui.selectable_value(
            &mut ui_state.settings_tab,
            SettingsTab::Ai,
            t.text("settings-tab-ai"),
        );
    });
}

fn display_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(t.text("settings-display-resolution")).color(war_text_muted()),
            );
            egui::ComboBox::from_id_salt("display_resolution")
                .selected_text(ui_state.pending_settings.display.resolution.to_string())
                .show_ui(ui, |ui| {
                    for resolution in DisplayResolution::presets() {
                        ui.selectable_value(
                            &mut ui_state.pending_settings.display.resolution,
                            *resolution,
                            resolution.to_string(),
                        );
                    }
                });
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.text("settings-display-mode")).color(war_text_muted()));
            for mode in DisplayMode::variants() {
                ui.radio_value(
                    &mut ui_state.pending_settings.display.display_mode,
                    *mode,
                    mode.label(t),
                );
            }
        });

        ui.checkbox(
            &mut ui_state.pending_settings.display.vsync,
            t.text("settings-vsync"),
        );
    });
}

fn audio_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.text("settings-master-volume")).color(war_text_muted()));
            let mut volume = ui_state.pending_settings.audio.master_volume;
            let changed = ui
                .add(
                    egui::Slider::new(&mut volume, 0.0..=1.0)
                        .show_value(false)
                        .fixed_decimals(0),
                )
                .changed();
            if changed {
                ui_state.pending_settings.audio.master_volume = normalize_master_volume(volume);
            }
            ui.label(format!(
                "{}%",
                (ui_state.pending_settings.audio.master_volume * 100.0).round() as u32
            ));
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new(t.text("settings-output-device")).color(war_text_muted()));
            let selected_text = ui_state
                .pending_settings
                .audio
                .output_device_name
                .clone()
                .unwrap_or_else(|| t.text("settings-system-default"));
            let device_names = ui_state.audio_output_devices.clone();
            egui::ComboBox::from_id_salt("audio_output_device")
                .width(260.0)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut ui_state.pending_settings.audio.output_device_name,
                        None,
                        t.text("settings-system-default"),
                    );
                    for device_name in device_names {
                        ui.selectable_value(
                            &mut ui_state.pending_settings.audio.output_device_name,
                            Some(device_name.clone()),
                            &device_name,
                        );
                    }
                });

            if ui.button(t.text("settings-refresh")).clicked() {
                refresh_audio_output_devices(ui_state);
            }
        });

        if let Some(error) = &ui_state.audio_output_devices_error {
            ui.colored_label(war_gold(), error);
        } else if ui_state.audio_output_devices.is_empty() {
            ui.label(
                egui::RichText::new(t.text("settings-no-extra-output-devices"))
                    .color(war_text_muted()),
            );
        }
    });
}

fn language_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(t.text("settings-ui-language")).color(war_text_muted()));
        egui::ComboBox::from_id_salt("ui_language")
            .width(180.0)
            .selected_text(ui_state.pending_settings.general.ui_language.label())
            .show_ui(ui, |ui| {
                for language in UiLanguage::available() {
                    ui.selectable_value(
                        &mut ui_state.pending_settings.general.ui_language,
                        *language,
                        language.label(),
                    );
                }
            });
    });
    ui.colored_label(war_text_muted(), t.text("settings-language-apply-hint"));
}

fn gameplay_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.checkbox(
            &mut ui_state.pending_settings.gameplay.autosave_enabled,
            t.text("settings-gameplay-autosave"),
        );
        ui.colored_label(war_text_muted(), t.text("settings-gameplay-autosave-hint"));
    });
}

fn shortcut_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.label(egui::RichText::new(t.text("settings-shortcuts-hint")).color(war_text_muted()));
    if let Err(error) = ui_state.pending_settings.shortcuts.clone().validated() {
        ui.colored_label(
            war_warning(),
            t.text_args("settings-shortcuts-invalid", &args([("error", error)])),
        );
    }
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_salt("settings_shortcuts")
        .max_height(360.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for group in shortcut_groups() {
                ui.label(egui::RichText::new(t.text(group.label_key)).strong());
                ui.add_space(4.0);
                for action in group.actions {
                    shortcut_binding_row(ui, ui_state, *action, t);
                }
                ui.add_space(8.0);
            }
        });
}

fn shortcut_binding_row(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    action: ShortcutAction,
    t: &Translator,
) {
    let capturing = ui_state.shortcut_capture_action == Some(action);
    let binding = ui_state.pending_settings.shortcuts.binding(action);
    let binding_text = if capturing {
        t.text("settings-shortcut-press-key")
    } else {
        shortcut_binding_text(binding, t)
    };

    ui.horizontal_wrapped(|ui| {
        ui.add_sized(
            [210.0, 26.0],
            egui::Label::new(t.text(action.label_key())).truncate(),
        );
        ui.add_sized(
            [118.0, 26.0],
            egui::Label::new(
                egui::RichText::new(binding_text)
                    .monospace()
                    .color(if capturing {
                        war_gold()
                    } else {
                        war_text_muted()
                    }),
            )
            .truncate(),
        );
        if ui
            .add_sized(
                [82.0, 26.0],
                egui::Button::new(t.text("settings-shortcut-rebind")),
            )
            .clicked()
        {
            ui_state.shortcut_capture_action = Some(action);
            ui_state.message = t.text_args(
                "message-shortcut-capturing",
                &args([("action", t.text(action.label_key()))]),
            );
        }
        if ui
            .add_sized(
                [64.0, 26.0],
                egui::Button::new(t.text("settings-shortcut-clear")),
            )
            .clicked()
        {
            *ui_state.pending_settings.shortcuts.binding_mut(action) = ShortcutBinding::unbound();
            if ui_state.shortcut_capture_action == Some(action) {
                ui_state.shortcut_capture_action = None;
            }
        }
        if ui
            .add_sized(
                [88.0, 26.0],
                egui::Button::new(t.text("settings-shortcut-default")),
            )
            .clicked()
        {
            ui_state.pending_settings.shortcuts.reset_binding(action);
            if ui_state.shortcut_capture_action == Some(action) {
                ui_state.shortcut_capture_action = None;
            }
        }
    });
}

fn shortcut_binding_text(binding: &ShortcutBinding, t: &Translator) -> String {
    binding
        .display_name()
        .unwrap_or_else(|| t.text("settings-shortcut-unbound"))
}

const AI_SETTINGS_LABEL_WIDTH: f32 = 140.0;
const AI_SETTINGS_FIELD_MIN_WIDTH: f32 = 160.0;
const AI_SETTINGS_FIELD_MAX_WIDTH: f32 = 430.0;
const AI_SETTINGS_GRID_COLUMN_GAP: f32 = 18.0;

fn ai_settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        let field_width = ai_settings_field_width(ui.available_width());
        egui::Grid::new("settings_ai_grid")
            .num_columns(2)
            .spacing(egui::vec2(AI_SETTINGS_GRID_COLUMN_GAP, 8.0))
            .show(ui, |ui| {
                ai_settings_section_row(ui, t, "settings-ai-reasoning");
                openai_api_type_row(ui, ui_state, t, field_width);
                ai_settings_text_field_row(
                    ui,
                    t,
                    "settings-ai-openai-api-url",
                    "ai_reasoning_api_url",
                    &mut ui_state.pending_settings.ai.reasoning.api_url,
                    false,
                    field_width,
                );
                ai_settings_text_field_row(
                    ui,
                    t,
                    "settings-ai-openai-token",
                    "ai_reasoning_token",
                    &mut ui_state.pending_settings.ai.reasoning.token,
                    true,
                    field_width,
                );
                ai_settings_text_field_row(
                    ui,
                    t,
                    "settings-ai-openai-model",
                    "ai_reasoning_model",
                    &mut ui_state.pending_settings.ai.reasoning.model_name,
                    false,
                    field_width,
                );

                ai_settings_spacer_row(ui);
                ai_settings_section_row(ui, t, "settings-ai-multimodal");
                ai_settings_text_field_row(
                    ui,
                    t,
                    "settings-ai-bailian-api-key",
                    "ai_multimodal_api_key",
                    &mut ui_state.pending_settings.ai.multimodal.api_key,
                    true,
                    field_width,
                );
                ai_settings_text_field_row(
                    ui,
                    t,
                    "settings-ai-bailian-model",
                    "ai_multimodal_model",
                    &mut ui_state.pending_settings.ai.multimodal.model_name,
                    false,
                    field_width,
                );
            });
    });
}

fn ai_settings_field_width(available_width: f32) -> f32 {
    (available_width - AI_SETTINGS_LABEL_WIDTH - AI_SETTINGS_GRID_COLUMN_GAP)
        .clamp(AI_SETTINGS_FIELD_MIN_WIDTH, AI_SETTINGS_FIELD_MAX_WIDTH)
}

fn ai_settings_section_row(ui: &mut egui::Ui, t: &Translator, label_key: &str) {
    ai_settings_grid_label(
        ui,
        egui::RichText::new(t.text(label_key))
            .strong()
            .color(war_gold()),
    );
    ui.add_space(0.0);
    ui.end_row();
}

fn ai_settings_spacer_row(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.add_space(4.0);
    ui.end_row();
}

fn ai_settings_grid_label(ui: &mut egui::Ui, label: egui::RichText) {
    ui.scope(|ui| {
        ui.set_width(AI_SETTINGS_LABEL_WIDTH);
        ui.label(label);
    });
}

fn openai_api_type_row(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    field_width: f32,
) {
    ai_settings_grid_label(
        ui,
        egui::RichText::new(t.text("settings-ai-openai-api-type")).color(war_text_muted()),
    );
    let previous = ui_state.pending_settings.ai.reasoning.api_type;
    egui::ComboBox::from_id_salt("ai_reasoning_api_type")
        .width(field_width)
        .selected_text(openai_api_type_label(previous, t))
        .show_ui(ui, |ui| {
            for api_type in OpenAiApiType::variants() {
                ui.selectable_value(
                    &mut ui_state.pending_settings.ai.reasoning.api_type,
                    *api_type,
                    openai_api_type_label(*api_type, t),
                );
            }
        });
    let selected = ui_state.pending_settings.ai.reasoning.api_type;
    if selected != previous {
        let api_url = &mut ui_state.pending_settings.ai.reasoning.api_url;
        if api_url.trim().is_empty() {
            *api_url = selected.default_api_url();
        } else if let Some(updated) = selected.api_url_from_existing_url(api_url) {
            *api_url = updated;
        }
    }
    ui.end_row();
}

fn openai_api_type_label(api_type: OpenAiApiType, t: &Translator) -> String {
    match api_type {
        OpenAiApiType::Responses => t.text("openai-api-type-responses"),
        OpenAiApiType::ChatCompletions => t.text("openai-api-type-chat-completions"),
        OpenAiApiType::Completions => t.text("openai-api-type-completions"),
    }
}

fn ai_settings_text_field_row(
    ui: &mut egui::Ui,
    t: &Translator,
    label_key: &str,
    id_salt: &str,
    value: &mut String,
    password: bool,
    field_width: f32,
) {
    ai_settings_grid_label(
        ui,
        egui::RichText::new(t.text(label_key)).color(war_text_muted()),
    );
    let edit = egui::TextEdit::singleline(value)
        .id_salt(id_salt)
        .desired_width(field_width);
    if password {
        ui.add(edit.password(true));
    } else {
        ui.add(edit);
    }
    ui.end_row();
}

pub(super) fn apply_pending_game_settings(
    ui_state: &mut GameUiState,
    window: &mut Window,
    main_menu_audio: &mut MainMenuAudio,
) {
    let settings = match ui_state.pending_settings.clone().validated() {
        Ok(settings) => settings,
        Err(error) => {
            let t = Translator::new(ui_state.pending_settings.general.ui_language);
            ui_state.message = t.text_args(
                "message-settings-invalid",
                &args([("error", error.to_string())]),
            );
            return;
        }
    };
    let t = Translator::new(settings.general.ui_language);
    settings.display.apply_to_window(window);
    ui_state.pending_settings = settings.clone();
    ui_state.applied_settings = settings.clone();
    ui_state.shortcut_capture_action = None;

    let audio_message = match main_menu_audio.sync(
        ui_state.screen,
        ui_state.main_menu_bgm_enabled,
        &settings.audio,
    ) {
        Ok(Some(warning)) => Some(warning),
        Ok(None) => None,
        Err(error) => Some(t.text_args("message-bgm-unavailable", &args([("error", error)]))),
    };

    match ui_state.settings_store.save(settings) {
        Ok(()) => {
            ui_state.message = t.text_args(
                "message-settings-saved",
                &args([("path", ui_state.settings_store.path().display().to_string())]),
            );
        }
        Err(error) => {
            ui_state.message = t.text_args(
                "message-settings-save-failed",
                &args([("error", error.to_string())]),
            );
        }
    }

    if let Some(audio_message) = audio_message {
        if !ui_state.message.is_empty() {
            ui_state.message.push('\n');
        }
        ui_state.message.push_str(&audio_message);
    }
}

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
            gameplay: GameplaySettings {
                autosave_enabled: true,
            },
            shortcuts: ShortcutSettings {
                toggle_city_list: ShortcutBinding::new("KeyL").with_ctrl(),
                ..ShortcutSettings::default()
            },
            ai: AiSettings {
                reasoning: ReasoningAiSettings {
                    api_type: OpenAiApiType::Responses,
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
        assert_eq!(loaded.settings.gameplay, GameplaySettings::default());
        assert_eq!(loaded.settings.shortcuts, ShortcutSettings::default());
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
        assert_eq!(loaded.settings.gameplay, GameplaySettings::default());
        assert_eq!(loaded.settings.shortcuts, ShortcutSettings::default());
        assert_eq!(loaded.settings.ai, AiSettings::default());
        assert!(loaded.message.is_none());
    }

    #[test]
    fn default_gameplay_settings_disable_autosave() {
        assert!(!GameSettings::default().gameplay.autosave_enabled);
    }

    #[test]
    fn gameplay_settings_load_from_settings() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "gameplay": {
    "autosave_enabled": true
  }
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert!(loaded.settings.gameplay.autosave_enabled);
        assert!(loaded.message.is_none());
    }

    #[test]
    fn shortcuts_load_from_settings() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "shortcuts": {
    "toggle_city_list": { "key": "KeyL", "ctrl": true },
    "return_main_menu": { "key": "F12" }
  }
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(
            loaded.settings.shortcuts.toggle_city_list,
            ShortcutBinding::new("KeyL").with_ctrl()
        );
        assert_eq!(
            loaded.settings.shortcuts.return_main_menu,
            ShortcutBinding::new("F12")
        );
        assert_eq!(
            loaded.settings.shortcuts.close_panel,
            ShortcutSettings::default().close_panel
        );
        assert!(loaded.message.is_none());
    }

    #[test]
    fn duplicate_shortcuts_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("settings.json");
        fs::write(
            &path,
            r#"{
  "shortcuts": {
    "toggle_city_list": { "key": "KeyC" },
    "toggle_events": { "key": "KeyC" }
  }
}"#,
        )
        .unwrap();
        let store = GameSettingsStore::new(path);

        let loaded = store.load();

        assert_eq!(loaded.settings, GameSettings::default());
        assert!(loaded.message.is_some());
    }

    #[test]
    fn default_settings_restore_default_shortcuts() {
        let settings = GameSettings {
            shortcuts: ShortcutSettings {
                toggle_city_list: ShortcutBinding::new("KeyL"),
                ..ShortcutSettings::default()
            },
            ..GameSettings::default()
        };

        assert_ne!(settings.shortcuts, ShortcutSettings::default());
        assert_eq!(
            GameSettings::default().shortcuts,
            ShortcutSettings::default()
        );
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
      "api_type": "chat_completions",
      "api_url": " https://proxy.example/v1/chat/completions ",
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
            loaded.settings.ai.reasoning.api_type,
            OpenAiApiType::ChatCompletions
        );
        assert_eq!(
            loaded.settings.ai.reasoning.api_url,
            "https://proxy.example/v1/chat/completions"
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
