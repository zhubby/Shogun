use bevy::prelude::{ButtonInput, KeyCode};
use bevy_egui::egui;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::MAP_ZOOM_STEP;
use super::actions::{clear_pending_commands, finish_current_turn, open_city};
use super::i18n::{Translator, args};
use super::map::{reset_map_view, zoom_map};
use super::state::{GameUiState, Screen};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShortcutAction {
    ClosePanel,
    ToggleSettings,
    ToggleSavePanel,
    ToggleCityList,
    ToggleOfficerBrowser,
    ToggleRetainers,
    ToggleShrine,
    ToggleTechnology,
    ToggleEvents,
    ToggleReports,
    OpenCouncilHall,
    ZoomMapIn,
    ZoomMapOut,
    ResetMapView,
    ToggleMapBoundaries,
    EndMonth,
    ClearCommands,
    ReturnMainMenu,
}

impl ShortcutAction {
    pub(super) fn all() -> &'static [Self] {
        &SHORTCUT_ACTIONS
    }

    pub(super) fn label_key(self) -> &'static str {
        match self {
            Self::ClosePanel => "shortcut-action-close-panel",
            Self::ToggleSettings => "shortcut-action-toggle-settings",
            Self::ToggleSavePanel => "shortcut-action-toggle-save-panel",
            Self::ToggleCityList => "shortcut-action-toggle-city-list",
            Self::ToggleOfficerBrowser => "shortcut-action-toggle-officer-browser",
            Self::ToggleRetainers => "shortcut-action-toggle-retainers",
            Self::ToggleShrine => "shortcut-action-toggle-shrine",
            Self::ToggleTechnology => "shortcut-action-toggle-technology",
            Self::ToggleEvents => "shortcut-action-toggle-events",
            Self::ToggleReports => "shortcut-action-toggle-reports",
            Self::OpenCouncilHall => "shortcut-action-open-council-hall",
            Self::ZoomMapIn => "shortcut-action-zoom-map-in",
            Self::ZoomMapOut => "shortcut-action-zoom-map-out",
            Self::ResetMapView => "shortcut-action-reset-map-view",
            Self::ToggleMapBoundaries => "shortcut-action-toggle-map-boundaries",
            Self::EndMonth => "shortcut-action-end-month",
            Self::ClearCommands => "shortcut-action-clear-commands",
            Self::ReturnMainMenu => "shortcut-action-return-main-menu",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::ClosePanel => "close_panel",
            Self::ToggleSettings => "toggle_settings",
            Self::ToggleSavePanel => "toggle_save_panel",
            Self::ToggleCityList => "toggle_city_list",
            Self::ToggleOfficerBrowser => "toggle_officer_browser",
            Self::ToggleRetainers => "toggle_retainers",
            Self::ToggleShrine => "toggle_shrine",
            Self::ToggleTechnology => "toggle_technology",
            Self::ToggleEvents => "toggle_events",
            Self::ToggleReports => "toggle_reports",
            Self::OpenCouncilHall => "open_council_hall",
            Self::ZoomMapIn => "zoom_map_in",
            Self::ZoomMapOut => "zoom_map_out",
            Self::ResetMapView => "reset_map_view",
            Self::ToggleMapBoundaries => "toggle_map_boundaries",
            Self::EndMonth => "end_month",
            Self::ClearCommands => "clear_commands",
            Self::ReturnMainMenu => "return_main_menu",
        }
    }

    fn default_binding(self) -> ShortcutBinding {
        match self {
            Self::ClosePanel => ShortcutBinding::new("Escape"),
            Self::ToggleSettings => ShortcutBinding::new("F10"),
            Self::ToggleSavePanel => ShortcutBinding::new("KeyS").with_ctrl(),
            Self::ToggleCityList => ShortcutBinding::new("KeyC"),
            Self::ToggleOfficerBrowser => ShortcutBinding::new("KeyO"),
            Self::ToggleRetainers => ShortcutBinding::new("KeyV"),
            Self::ToggleShrine => ShortcutBinding::new("KeyH"),
            Self::ToggleTechnology => ShortcutBinding::new("KeyT"),
            Self::ToggleEvents => ShortcutBinding::new("KeyE"),
            Self::ToggleReports => ShortcutBinding::new("KeyR"),
            Self::OpenCouncilHall => ShortcutBinding::new("Enter"),
            Self::ZoomMapIn => ShortcutBinding::new("Equal"),
            Self::ZoomMapOut => ShortcutBinding::new("Minus"),
            Self::ResetMapView => ShortcutBinding::new("Digit0"),
            Self::ToggleMapBoundaries => ShortcutBinding::new("KeyB"),
            Self::EndMonth => ShortcutBinding::new("Enter").with_ctrl(),
            Self::ClearCommands => ShortcutBinding::new("Backspace").with_ctrl(),
            Self::ReturnMainMenu => ShortcutBinding::unbound(),
        }
    }
}

const SHORTCUT_ACTIONS: [ShortcutAction; 18] = [
    ShortcutAction::ClosePanel,
    ShortcutAction::ToggleSettings,
    ShortcutAction::ToggleSavePanel,
    ShortcutAction::ToggleCityList,
    ShortcutAction::ToggleOfficerBrowser,
    ShortcutAction::ToggleRetainers,
    ShortcutAction::ToggleShrine,
    ShortcutAction::ToggleTechnology,
    ShortcutAction::ToggleEvents,
    ShortcutAction::ToggleReports,
    ShortcutAction::OpenCouncilHall,
    ShortcutAction::ZoomMapIn,
    ShortcutAction::ZoomMapOut,
    ShortcutAction::ResetMapView,
    ShortcutAction::ToggleMapBoundaries,
    ShortcutAction::EndMonth,
    ShortcutAction::ClearCommands,
    ShortcutAction::ReturnMainMenu,
];

pub(super) struct ShortcutGroup {
    pub(super) label_key: &'static str,
    pub(super) actions: &'static [ShortcutAction],
}

pub(super) fn shortcut_groups() -> &'static [ShortcutGroup] {
    &SHORTCUT_GROUPS
}

const GENERAL_SHORTCUT_ACTIONS: [ShortcutAction; 5] = [
    ShortcutAction::ClosePanel,
    ShortcutAction::ToggleSettings,
    ShortcutAction::ToggleSavePanel,
    ShortcutAction::OpenCouncilHall,
    ShortcutAction::ReturnMainMenu,
];
const PANEL_SHORTCUT_ACTIONS: [ShortcutAction; 7] = [
    ShortcutAction::ToggleCityList,
    ShortcutAction::ToggleOfficerBrowser,
    ShortcutAction::ToggleRetainers,
    ShortcutAction::ToggleShrine,
    ShortcutAction::ToggleTechnology,
    ShortcutAction::ToggleEvents,
    ShortcutAction::ToggleReports,
];
const MAP_SHORTCUT_ACTIONS: [ShortcutAction; 4] = [
    ShortcutAction::ZoomMapIn,
    ShortcutAction::ZoomMapOut,
    ShortcutAction::ResetMapView,
    ShortcutAction::ToggleMapBoundaries,
];
const TURN_SHORTCUT_ACTIONS: [ShortcutAction; 2] =
    [ShortcutAction::EndMonth, ShortcutAction::ClearCommands];

const SHORTCUT_GROUPS: [ShortcutGroup; 4] = [
    ShortcutGroup {
        label_key: "shortcut-group-general",
        actions: &GENERAL_SHORTCUT_ACTIONS,
    },
    ShortcutGroup {
        label_key: "shortcut-group-panels",
        actions: &PANEL_SHORTCUT_ACTIONS,
    },
    ShortcutGroup {
        label_key: "shortcut-group-map",
        actions: &MAP_SHORTCUT_ACTIONS,
    },
    ShortcutGroup {
        label_key: "shortcut-group-turn",
        actions: &TURN_SHORTCUT_ACTIONS,
    },
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub(super) struct ShortcutSettings {
    #[serde(default = "default_close_panel")]
    pub(super) close_panel: ShortcutBinding,
    #[serde(default = "default_toggle_settings")]
    pub(super) toggle_settings: ShortcutBinding,
    #[serde(default = "default_toggle_save_panel")]
    pub(super) toggle_save_panel: ShortcutBinding,
    #[serde(default = "default_toggle_city_list")]
    pub(super) toggle_city_list: ShortcutBinding,
    #[serde(default = "default_toggle_officer_browser")]
    pub(super) toggle_officer_browser: ShortcutBinding,
    #[serde(default = "default_toggle_retainers")]
    pub(super) toggle_retainers: ShortcutBinding,
    #[serde(default = "default_toggle_shrine")]
    pub(super) toggle_shrine: ShortcutBinding,
    #[serde(default = "default_toggle_technology")]
    pub(super) toggle_technology: ShortcutBinding,
    #[serde(default = "default_toggle_events")]
    pub(super) toggle_events: ShortcutBinding,
    #[serde(default = "default_toggle_reports")]
    pub(super) toggle_reports: ShortcutBinding,
    #[serde(default = "default_open_council_hall")]
    pub(super) open_council_hall: ShortcutBinding,
    #[serde(default = "default_zoom_map_in")]
    pub(super) zoom_map_in: ShortcutBinding,
    #[serde(default = "default_zoom_map_out")]
    pub(super) zoom_map_out: ShortcutBinding,
    #[serde(default = "default_reset_map_view")]
    pub(super) reset_map_view: ShortcutBinding,
    #[serde(default = "default_toggle_map_boundaries")]
    pub(super) toggle_map_boundaries: ShortcutBinding,
    #[serde(default = "default_end_month")]
    pub(super) end_month: ShortcutBinding,
    #[serde(default = "default_clear_commands")]
    pub(super) clear_commands: ShortcutBinding,
    #[serde(default = "default_return_main_menu")]
    pub(super) return_main_menu: ShortcutBinding,
}

impl ShortcutSettings {
    pub(super) fn validated(mut self) -> Result<Self, String> {
        for action in ShortcutAction::all() {
            let normalized = self.binding(*action).clone().normalized()?;
            *self.binding_mut(*action) = normalized;
        }

        let mut seen = BTreeMap::new();
        for action in ShortcutAction::all() {
            let binding = self.binding(*action);
            if !binding.is_bound() {
                continue;
            }
            if let Some(previous) = seen.insert(binding.clone(), *action) {
                let display = binding
                    .display_name()
                    .unwrap_or_else(|| "unbound".to_string());
                return Err(format!(
                    "快捷键冲突: {} 与 {} 都绑定到 {}",
                    previous.id(),
                    action.id(),
                    display
                ));
            }
        }

        Ok(self)
    }

    pub(super) fn binding(&self, action: ShortcutAction) -> &ShortcutBinding {
        match action {
            ShortcutAction::ClosePanel => &self.close_panel,
            ShortcutAction::ToggleSettings => &self.toggle_settings,
            ShortcutAction::ToggleSavePanel => &self.toggle_save_panel,
            ShortcutAction::ToggleCityList => &self.toggle_city_list,
            ShortcutAction::ToggleOfficerBrowser => &self.toggle_officer_browser,
            ShortcutAction::ToggleRetainers => &self.toggle_retainers,
            ShortcutAction::ToggleShrine => &self.toggle_shrine,
            ShortcutAction::ToggleTechnology => &self.toggle_technology,
            ShortcutAction::ToggleEvents => &self.toggle_events,
            ShortcutAction::ToggleReports => &self.toggle_reports,
            ShortcutAction::OpenCouncilHall => &self.open_council_hall,
            ShortcutAction::ZoomMapIn => &self.zoom_map_in,
            ShortcutAction::ZoomMapOut => &self.zoom_map_out,
            ShortcutAction::ResetMapView => &self.reset_map_view,
            ShortcutAction::ToggleMapBoundaries => &self.toggle_map_boundaries,
            ShortcutAction::EndMonth => &self.end_month,
            ShortcutAction::ClearCommands => &self.clear_commands,
            ShortcutAction::ReturnMainMenu => &self.return_main_menu,
        }
    }

    pub(super) fn binding_mut(&mut self, action: ShortcutAction) -> &mut ShortcutBinding {
        match action {
            ShortcutAction::ClosePanel => &mut self.close_panel,
            ShortcutAction::ToggleSettings => &mut self.toggle_settings,
            ShortcutAction::ToggleSavePanel => &mut self.toggle_save_panel,
            ShortcutAction::ToggleCityList => &mut self.toggle_city_list,
            ShortcutAction::ToggleOfficerBrowser => &mut self.toggle_officer_browser,
            ShortcutAction::ToggleRetainers => &mut self.toggle_retainers,
            ShortcutAction::ToggleShrine => &mut self.toggle_shrine,
            ShortcutAction::ToggleTechnology => &mut self.toggle_technology,
            ShortcutAction::ToggleEvents => &mut self.toggle_events,
            ShortcutAction::ToggleReports => &mut self.toggle_reports,
            ShortcutAction::OpenCouncilHall => &mut self.open_council_hall,
            ShortcutAction::ZoomMapIn => &mut self.zoom_map_in,
            ShortcutAction::ZoomMapOut => &mut self.zoom_map_out,
            ShortcutAction::ResetMapView => &mut self.reset_map_view,
            ShortcutAction::ToggleMapBoundaries => &mut self.toggle_map_boundaries,
            ShortcutAction::EndMonth => &mut self.end_month,
            ShortcutAction::ClearCommands => &mut self.clear_commands,
            ShortcutAction::ReturnMainMenu => &mut self.return_main_menu,
        }
    }

    pub(super) fn reset_binding(&mut self, action: ShortcutAction) {
        *self.binding_mut(action) = action.default_binding();
    }

    fn action_for_input(&self, input: &ButtonInput<KeyCode>) -> Option<ShortcutAction> {
        ShortcutAction::all()
            .iter()
            .copied()
            .find(|action| self.binding(*action).is_triggered(input))
    }
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            close_panel: default_close_panel(),
            toggle_settings: default_toggle_settings(),
            toggle_save_panel: default_toggle_save_panel(),
            toggle_city_list: default_toggle_city_list(),
            toggle_officer_browser: default_toggle_officer_browser(),
            toggle_retainers: default_toggle_retainers(),
            toggle_shrine: default_toggle_shrine(),
            toggle_technology: default_toggle_technology(),
            toggle_events: default_toggle_events(),
            toggle_reports: default_toggle_reports(),
            open_council_hall: default_open_council_hall(),
            zoom_map_in: default_zoom_map_in(),
            zoom_map_out: default_zoom_map_out(),
            reset_map_view: default_reset_map_view(),
            toggle_map_boundaries: default_toggle_map_boundaries(),
            end_month: default_end_month(),
            clear_commands: default_clear_commands(),
            return_main_menu: default_return_main_menu(),
        }
    }
}

fn default_close_panel() -> ShortcutBinding {
    ShortcutAction::ClosePanel.default_binding()
}

fn default_toggle_settings() -> ShortcutBinding {
    ShortcutAction::ToggleSettings.default_binding()
}

fn default_toggle_save_panel() -> ShortcutBinding {
    ShortcutAction::ToggleSavePanel.default_binding()
}

fn default_toggle_city_list() -> ShortcutBinding {
    ShortcutAction::ToggleCityList.default_binding()
}

fn default_toggle_officer_browser() -> ShortcutBinding {
    ShortcutAction::ToggleOfficerBrowser.default_binding()
}

fn default_toggle_retainers() -> ShortcutBinding {
    ShortcutAction::ToggleRetainers.default_binding()
}

fn default_toggle_shrine() -> ShortcutBinding {
    ShortcutAction::ToggleShrine.default_binding()
}

fn default_toggle_technology() -> ShortcutBinding {
    ShortcutAction::ToggleTechnology.default_binding()
}

fn default_toggle_events() -> ShortcutBinding {
    ShortcutAction::ToggleEvents.default_binding()
}

fn default_toggle_reports() -> ShortcutBinding {
    ShortcutAction::ToggleReports.default_binding()
}

fn default_open_council_hall() -> ShortcutBinding {
    ShortcutAction::OpenCouncilHall.default_binding()
}

fn default_zoom_map_in() -> ShortcutBinding {
    ShortcutAction::ZoomMapIn.default_binding()
}

fn default_zoom_map_out() -> ShortcutBinding {
    ShortcutAction::ZoomMapOut.default_binding()
}

fn default_reset_map_view() -> ShortcutBinding {
    ShortcutAction::ResetMapView.default_binding()
}

fn default_toggle_map_boundaries() -> ShortcutBinding {
    ShortcutAction::ToggleMapBoundaries.default_binding()
}

fn default_end_month() -> ShortcutBinding {
    ShortcutAction::EndMonth.default_binding()
}

fn default_clear_commands() -> ShortcutBinding {
    ShortcutAction::ClearCommands.default_binding()
}

fn default_return_main_menu() -> ShortcutBinding {
    ShortcutAction::ReturnMainMenu.default_binding()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(default)]
pub(super) struct ShortcutBinding {
    pub(super) key: Option<String>,
    pub(super) ctrl: bool,
    pub(super) shift: bool,
    pub(super) alt: bool,
    #[serde(rename = "super")]
    pub(super) super_key: bool,
}

impl ShortcutBinding {
    pub(super) fn new(key: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            ctrl: false,
            shift: false,
            alt: false,
            super_key: false,
        }
    }

    pub(super) fn unbound() -> Self {
        Self::default()
    }

    pub(super) fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub(super) fn is_bound(&self) -> bool {
        self.key
            .as_deref()
            .is_some_and(|key| !key.trim().is_empty())
    }

    pub(super) fn display_name(&self) -> Option<String> {
        let key = self.key_code()?;
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.shift {
            parts.push("Shift".to_string());
        }
        if self.alt {
            parts.push("Alt".to_string());
        }
        if self.super_key {
            parts.push("Super".to_string());
        }
        parts.push(key_display_label(key).to_string());
        Some(parts.join("+"))
    }

    fn normalized(mut self) -> Result<Self, String> {
        let Some(key) = self
            .key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
        else {
            return Ok(Self::unbound());
        };
        let Some(key_code) = key_code_from_name(key) else {
            return Err(format!("不支持的快捷键: {key}"));
        };
        if is_modifier_key(key_code) {
            return Err("快捷键不能只使用修饰键".to_string());
        }
        self.key = key_code_name(key_code).map(str::to_string);
        Ok(self)
    }

    fn is_triggered(&self, input: &ButtonInput<KeyCode>) -> bool {
        let Some(key) = self.key_code() else {
            return false;
        };
        input.just_pressed(key) && self.modifiers_match(input)
    }

    fn key_code(&self) -> Option<KeyCode> {
        self.key.as_deref().and_then(key_code_from_name)
    }

    fn modifiers_match(&self, input: &ButtonInput<KeyCode>) -> bool {
        self.ctrl == input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            && self.shift == input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
            && self.alt == input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
            && self.super_key == input.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
    }
}

pub(super) fn handle_shortcut_input(
    ctx: &egui::Context,
    input: &ButtonInput<KeyCode>,
    ui_state: &mut GameUiState,
) {
    if handle_shortcut_capture(input, ui_state) {
        return;
    }
    if ctx.wants_keyboard_input() {
        return;
    }
    if let Some(action) = ui_state.applied_settings.shortcuts.action_for_input(input) {
        if ui_state.settings_open
            && !matches!(
                action,
                ShortcutAction::ClosePanel | ShortcutAction::ToggleSettings
            )
        {
            return;
        }
        dispatch_shortcut_action(action, ui_state);
    }
}

fn handle_shortcut_capture(input: &ButtonInput<KeyCode>, ui_state: &mut GameUiState) -> bool {
    let Some(action) = ui_state.shortcut_capture_action else {
        return false;
    };
    let t = Translator::new(ui_state.pending_settings.general.ui_language);
    match binding_from_just_pressed_input(input) {
        Ok(Some(binding)) => {
            let display = binding
                .display_name()
                .unwrap_or_else(|| t.text("settings-shortcut-unbound"));
            *ui_state.pending_settings.shortcuts.binding_mut(action) = binding;
            ui_state.shortcut_capture_action = None;
            ui_state.message = t.text_args(
                "message-shortcut-captured",
                &args([("action", t.text(action.label_key())), ("binding", display)]),
            );
        }
        Ok(None) => {}
        Err(CaptureError::ModifierOnly) => {
            ui_state.message = t.text("message-shortcut-modifier-only");
        }
        Err(CaptureError::UnsupportedKey(key)) => {
            ui_state.message =
                t.text_args("message-shortcut-unsupported-key", &args([("key", key)]));
        }
    }
    true
}

fn binding_from_just_pressed_input(
    input: &ButtonInput<KeyCode>,
) -> Result<Option<ShortcutBinding>, CaptureError> {
    let mut just_pressed = input.get_just_pressed().copied().collect::<Vec<_>>();
    if just_pressed.is_empty() {
        return Ok(None);
    }
    just_pressed.sort();
    let Some(key) = just_pressed
        .iter()
        .copied()
        .find(|key| !is_modifier_key(*key))
    else {
        return Err(CaptureError::ModifierOnly);
    };
    let Some(name) = key_code_name(key) else {
        return Err(CaptureError::UnsupportedKey(format!("{key:?}")));
    };
    Ok(Some(ShortcutBinding {
        key: Some(name.to_string()),
        ctrl: input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        shift: input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        alt: input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]),
        super_key: input.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]),
    }))
}

enum CaptureError {
    ModifierOnly,
    UnsupportedKey(String),
}

pub(super) fn close_top_panel(ui_state: &mut GameUiState) -> bool {
    if ui_state.shortcut_capture_action.take().is_some() {
        return true;
    }
    if ui_state.settings_open {
        ui_state.settings_open = false;
        return true;
    }
    if ui_state.officer_edit_open {
        ui_state.officer_edit_open = false;
        ui_state.officer_edit_draft = None;
        ui_state.officer_edit_error = None;
        return true;
    }
    if ui_state.officer_detail_id.take().is_some() {
        return true;
    }
    if ui_state.main_menu_new_game_open {
        ui_state.main_menu_new_game_open = false;
        return true;
    }
    if ui_state.main_menu_load_game_open {
        ui_state.main_menu_load_game_open = false;
        return true;
    }
    if ui_state.officer_settings_open {
        ui_state.officer_settings_open = false;
        return true;
    }
    if ui_state.screen != Screen::InGame {
        return false;
    }
    if ui_state.events_open {
        ui_state.events_open = false;
        return true;
    }
    if ui_state.technology_open {
        ui_state.technology_open = false;
        return true;
    }
    if ui_state.shrine_open {
        ui_state.shrine_open = false;
        return true;
    }
    if ui_state.retainers_open {
        ui_state.retainers_open = false;
        return true;
    }
    if ui_state.officer_browser_open {
        ui_state.officer_browser_open = false;
        return true;
    }
    if ui_state.city_drawer_open {
        ui_state.city_drawer_open = false;
        return true;
    }
    if ui_state.city_list_open {
        ui_state.city_list_open = false;
        return true;
    }
    if ui_state.save_panel_open {
        ui_state.save_panel_open = false;
        return true;
    }
    if ui_state.reports_open {
        ui_state.reports_open = false;
        return true;
    }
    false
}

fn dispatch_shortcut_action(action: ShortcutAction, ui_state: &mut GameUiState) {
    match action {
        ShortcutAction::ClosePanel => {
            close_top_panel(ui_state);
        }
        ShortcutAction::ToggleSettings => {
            ui_state.settings_open = !ui_state.settings_open;
            if !ui_state.settings_open {
                ui_state.shortcut_capture_action = None;
            }
        }
        ShortcutAction::ToggleSavePanel => {
            if ui_state.screen == Screen::InGame {
                ui_state.save_panel_open = !ui_state.save_panel_open;
            }
        }
        ShortcutAction::ToggleCityList => {
            if ui_state.screen == Screen::InGame {
                ui_state.city_list_open = !ui_state.city_list_open;
            }
        }
        ShortcutAction::ToggleOfficerBrowser => {
            if ui_state.screen == Screen::InGame {
                ui_state.officer_browser_open = !ui_state.officer_browser_open;
            }
        }
        ShortcutAction::ToggleRetainers => {
            if ui_state.screen == Screen::InGame {
                ui_state.retainers_open = !ui_state.retainers_open;
            }
        }
        ShortcutAction::ToggleShrine => {
            if ui_state.screen == Screen::InGame {
                ui_state.shrine_open = !ui_state.shrine_open;
            }
        }
        ShortcutAction::ToggleTechnology => {
            if ui_state.screen == Screen::InGame {
                ui_state.technology_open = !ui_state.technology_open;
            }
        }
        ShortcutAction::ToggleEvents => {
            if ui_state.screen == Screen::InGame {
                ui_state.events_open = !ui_state.events_open;
            }
        }
        ShortcutAction::ToggleReports => {
            if ui_state.screen == Screen::InGame {
                ui_state.reports_open = !ui_state.reports_open;
            }
        }
        ShortcutAction::OpenCouncilHall => {
            if ui_state.screen == Screen::InGame
                && let Some(city_id) = ui_state.selected_city_id.clone()
            {
                open_city(ui_state, city_id);
            }
        }
        ShortcutAction::ZoomMapIn => {
            if ui_state.screen == Screen::InGame {
                zoom_map(ui_state, MAP_ZOOM_STEP, None, None);
            }
        }
        ShortcutAction::ZoomMapOut => {
            if ui_state.screen == Screen::InGame {
                zoom_map(ui_state, 1.0 / MAP_ZOOM_STEP, None, None);
            }
        }
        ShortcutAction::ResetMapView => {
            if ui_state.screen == Screen::InGame {
                reset_map_view(ui_state);
            }
        }
        ShortcutAction::ToggleMapBoundaries => {
            if ui_state.screen == Screen::InGame && ui_state.map_boundaries.is_some() {
                ui_state.map_boundaries_enabled = !ui_state.map_boundaries_enabled;
            }
        }
        ShortcutAction::EndMonth => {
            if ui_state.screen == Screen::InGame {
                finish_current_turn(ui_state);
            }
        }
        ShortcutAction::ClearCommands => {
            if ui_state.screen == Screen::InGame {
                clear_pending_commands(ui_state);
            }
        }
        ShortcutAction::ReturnMainMenu => {
            if ui_state.screen == Screen::InGame {
                close_in_game_panels(ui_state);
                ui_state.screen = Screen::MainMenu;
            }
        }
    }
}

fn close_in_game_panels(ui_state: &mut GameUiState) {
    ui_state.shortcut_capture_action = None;
    ui_state.city_drawer_open = false;
    ui_state.city_list_open = false;
    ui_state.officer_browser_open = false;
    ui_state.officer_detail_id = None;
    ui_state.retainers_open = false;
    ui_state.shrine_open = false;
    ui_state.technology_open = false;
    ui_state.events_open = false;
    ui_state.save_panel_open = false;
}

fn key_code_from_name(name: &str) -> Option<KeyCode> {
    let trimmed = name.trim();
    SUPPORTED_KEYS
        .iter()
        .find(|key| key.name == trimmed)
        .map(|key| key.code)
}

fn key_code_name(code: KeyCode) -> Option<&'static str> {
    SUPPORTED_KEYS
        .iter()
        .find(|key| key.code == code)
        .map(|key| key.name)
}

fn key_display_label(code: KeyCode) -> &'static str {
    SUPPORTED_KEYS
        .iter()
        .find(|key| key.code == code)
        .map(|key| key.label)
        .unwrap_or("Unknown")
}

fn is_modifier_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::ControlLeft
            | KeyCode::ControlRight
            | KeyCode::ShiftLeft
            | KeyCode::ShiftRight
            | KeyCode::AltLeft
            | KeyCode::AltRight
            | KeyCode::SuperLeft
            | KeyCode::SuperRight
    )
}

#[derive(Clone, Copy)]
struct SupportedKey {
    code: KeyCode,
    name: &'static str,
    label: &'static str,
}

const SUPPORTED_KEYS: &[SupportedKey] = &[
    SupportedKey {
        code: KeyCode::Escape,
        name: "Escape",
        label: "Esc",
    },
    SupportedKey {
        code: KeyCode::F1,
        name: "F1",
        label: "F1",
    },
    SupportedKey {
        code: KeyCode::F2,
        name: "F2",
        label: "F2",
    },
    SupportedKey {
        code: KeyCode::F3,
        name: "F3",
        label: "F3",
    },
    SupportedKey {
        code: KeyCode::F4,
        name: "F4",
        label: "F4",
    },
    SupportedKey {
        code: KeyCode::F5,
        name: "F5",
        label: "F5",
    },
    SupportedKey {
        code: KeyCode::F6,
        name: "F6",
        label: "F6",
    },
    SupportedKey {
        code: KeyCode::F7,
        name: "F7",
        label: "F7",
    },
    SupportedKey {
        code: KeyCode::F8,
        name: "F8",
        label: "F8",
    },
    SupportedKey {
        code: KeyCode::F9,
        name: "F9",
        label: "F9",
    },
    SupportedKey {
        code: KeyCode::F10,
        name: "F10",
        label: "F10",
    },
    SupportedKey {
        code: KeyCode::F11,
        name: "F11",
        label: "F11",
    },
    SupportedKey {
        code: KeyCode::F12,
        name: "F12",
        label: "F12",
    },
    SupportedKey {
        code: KeyCode::Backquote,
        name: "Backquote",
        label: "`",
    },
    SupportedKey {
        code: KeyCode::Digit0,
        name: "Digit0",
        label: "0",
    },
    SupportedKey {
        code: KeyCode::Digit1,
        name: "Digit1",
        label: "1",
    },
    SupportedKey {
        code: KeyCode::Digit2,
        name: "Digit2",
        label: "2",
    },
    SupportedKey {
        code: KeyCode::Digit3,
        name: "Digit3",
        label: "3",
    },
    SupportedKey {
        code: KeyCode::Digit4,
        name: "Digit4",
        label: "4",
    },
    SupportedKey {
        code: KeyCode::Digit5,
        name: "Digit5",
        label: "5",
    },
    SupportedKey {
        code: KeyCode::Digit6,
        name: "Digit6",
        label: "6",
    },
    SupportedKey {
        code: KeyCode::Digit7,
        name: "Digit7",
        label: "7",
    },
    SupportedKey {
        code: KeyCode::Digit8,
        name: "Digit8",
        label: "8",
    },
    SupportedKey {
        code: KeyCode::Digit9,
        name: "Digit9",
        label: "9",
    },
    SupportedKey {
        code: KeyCode::Minus,
        name: "Minus",
        label: "-",
    },
    SupportedKey {
        code: KeyCode::Equal,
        name: "Equal",
        label: "=",
    },
    SupportedKey {
        code: KeyCode::KeyA,
        name: "KeyA",
        label: "A",
    },
    SupportedKey {
        code: KeyCode::KeyB,
        name: "KeyB",
        label: "B",
    },
    SupportedKey {
        code: KeyCode::KeyC,
        name: "KeyC",
        label: "C",
    },
    SupportedKey {
        code: KeyCode::KeyD,
        name: "KeyD",
        label: "D",
    },
    SupportedKey {
        code: KeyCode::KeyE,
        name: "KeyE",
        label: "E",
    },
    SupportedKey {
        code: KeyCode::KeyF,
        name: "KeyF",
        label: "F",
    },
    SupportedKey {
        code: KeyCode::KeyG,
        name: "KeyG",
        label: "G",
    },
    SupportedKey {
        code: KeyCode::KeyH,
        name: "KeyH",
        label: "H",
    },
    SupportedKey {
        code: KeyCode::KeyI,
        name: "KeyI",
        label: "I",
    },
    SupportedKey {
        code: KeyCode::KeyJ,
        name: "KeyJ",
        label: "J",
    },
    SupportedKey {
        code: KeyCode::KeyK,
        name: "KeyK",
        label: "K",
    },
    SupportedKey {
        code: KeyCode::KeyL,
        name: "KeyL",
        label: "L",
    },
    SupportedKey {
        code: KeyCode::KeyM,
        name: "KeyM",
        label: "M",
    },
    SupportedKey {
        code: KeyCode::KeyN,
        name: "KeyN",
        label: "N",
    },
    SupportedKey {
        code: KeyCode::KeyO,
        name: "KeyO",
        label: "O",
    },
    SupportedKey {
        code: KeyCode::KeyP,
        name: "KeyP",
        label: "P",
    },
    SupportedKey {
        code: KeyCode::KeyQ,
        name: "KeyQ",
        label: "Q",
    },
    SupportedKey {
        code: KeyCode::KeyR,
        name: "KeyR",
        label: "R",
    },
    SupportedKey {
        code: KeyCode::KeyS,
        name: "KeyS",
        label: "S",
    },
    SupportedKey {
        code: KeyCode::KeyT,
        name: "KeyT",
        label: "T",
    },
    SupportedKey {
        code: KeyCode::KeyU,
        name: "KeyU",
        label: "U",
    },
    SupportedKey {
        code: KeyCode::KeyV,
        name: "KeyV",
        label: "V",
    },
    SupportedKey {
        code: KeyCode::KeyW,
        name: "KeyW",
        label: "W",
    },
    SupportedKey {
        code: KeyCode::KeyX,
        name: "KeyX",
        label: "X",
    },
    SupportedKey {
        code: KeyCode::KeyY,
        name: "KeyY",
        label: "Y",
    },
    SupportedKey {
        code: KeyCode::KeyZ,
        name: "KeyZ",
        label: "Z",
    },
    SupportedKey {
        code: KeyCode::BracketLeft,
        name: "BracketLeft",
        label: "[",
    },
    SupportedKey {
        code: KeyCode::BracketRight,
        name: "BracketRight",
        label: "]",
    },
    SupportedKey {
        code: KeyCode::Backslash,
        name: "Backslash",
        label: "\\",
    },
    SupportedKey {
        code: KeyCode::Semicolon,
        name: "Semicolon",
        label: ";",
    },
    SupportedKey {
        code: KeyCode::Quote,
        name: "Quote",
        label: "'",
    },
    SupportedKey {
        code: KeyCode::Comma,
        name: "Comma",
        label: ",",
    },
    SupportedKey {
        code: KeyCode::Period,
        name: "Period",
        label: ".",
    },
    SupportedKey {
        code: KeyCode::Slash,
        name: "Slash",
        label: "/",
    },
    SupportedKey {
        code: KeyCode::Space,
        name: "Space",
        label: "Space",
    },
    SupportedKey {
        code: KeyCode::Tab,
        name: "Tab",
        label: "Tab",
    },
    SupportedKey {
        code: KeyCode::Enter,
        name: "Enter",
        label: "Enter",
    },
    SupportedKey {
        code: KeyCode::Backspace,
        name: "Backspace",
        label: "Backspace",
    },
    SupportedKey {
        code: KeyCode::Delete,
        name: "Delete",
        label: "Delete",
    },
    SupportedKey {
        code: KeyCode::Insert,
        name: "Insert",
        label: "Insert",
    },
    SupportedKey {
        code: KeyCode::Home,
        name: "Home",
        label: "Home",
    },
    SupportedKey {
        code: KeyCode::End,
        name: "End",
        label: "End",
    },
    SupportedKey {
        code: KeyCode::PageUp,
        name: "PageUp",
        label: "PageUp",
    },
    SupportedKey {
        code: KeyCode::PageDown,
        name: "PageDown",
        label: "PageDown",
    },
    SupportedKey {
        code: KeyCode::ArrowUp,
        name: "ArrowUp",
        label: "Up",
    },
    SupportedKey {
        code: KeyCode::ArrowDown,
        name: "ArrowDown",
        label: "Down",
    },
    SupportedKey {
        code: KeyCode::ArrowLeft,
        name: "ArrowLeft",
        label: "Left",
    },
    SupportedKey {
        code: KeyCode::ArrowRight,
        name: "ArrowRight",
        label: "Right",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_have_no_conflicts() {
        assert!(ShortcutSettings::default().validated().is_ok());
    }

    #[test]
    fn modifier_matching_is_exact() {
        let settings = ShortcutSettings::default();
        let mut input = ButtonInput::default();
        input.press(KeyCode::KeyS);

        assert_eq!(settings.action_for_input(&input), None);

        let mut input = ButtonInput::default();
        input.press(KeyCode::ControlLeft);
        input.clear();
        input.press(KeyCode::KeyS);

        assert_eq!(
            settings.action_for_input(&input),
            Some(ShortcutAction::ToggleSavePanel)
        );

        let mut input = ButtonInput::default();
        input.press(KeyCode::ControlLeft);
        input.press(KeyCode::ShiftLeft);
        input.clear();
        input.press(KeyCode::KeyS);

        assert_eq!(settings.action_for_input(&input), None);
    }

    #[test]
    fn unbound_actions_do_not_trigger() {
        let settings = ShortcutSettings::default();
        let mut input = ButtonInput::default();
        input.press(KeyCode::KeyM);

        assert_eq!(settings.action_for_input(&input), None);
    }

    #[test]
    fn close_top_panel_prefers_foreground_layers() {
        let mut ui_state = GameUiState {
            settings_open: true,
            officer_edit_open: true,
            officer_detail_id: Some("liu_bei".to_string()),
            ..GameUiState::default()
        };

        assert!(close_top_panel(&mut ui_state));
        assert!(!ui_state.settings_open);
        assert!(ui_state.officer_edit_open);
        assert!(ui_state.officer_detail_id.is_some());

        assert!(close_top_panel(&mut ui_state));
        assert!(!ui_state.officer_edit_open);
        assert!(ui_state.officer_detail_id.is_some());

        assert!(close_top_panel(&mut ui_state));
        assert!(ui_state.officer_detail_id.is_none());
    }
}
