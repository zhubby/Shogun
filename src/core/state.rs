use crate::game::*;
use bevy::prelude::Resource;
use bevy_egui::egui;
use std::collections::BTreeMap;

use super::asset_path;
use super::i18n::{Translator, args};
use super::map::MapBoundaryViewCache;
use super::portraits::OfficerPortraitStore;
use super::settings::{GameSettings, GameSettingsStore, LoadedGameSettings};
use super::shortcuts::ShortcutAction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SettingsTab {
    Display,
    Audio,
    Language,
    Shortcuts,
    Ai,
}

#[derive(Resource)]
pub(super) struct GameUiState {
    pub(super) history_scenarios: Vec<HistoricalScenario>,
    pub(super) selected_scenario_id: ScenarioId,
    pub(super) history_factions: Vec<Faction>,
    pub(super) screen: Screen,
    pub(super) selected_command_category: CommandCategory,
    pub(super) selected_command_action: CommandAction,
    pub(super) map_zoom: f32,
    pub(super) map_pan: egui::Vec2,
    pub(super) map_boundaries_enabled: bool,
    pub(super) map_boundaries: Option<MapBoundaryCatalog>,
    pub(super) map_boundary_view_cache: MapBoundaryViewCache,
    pub(super) city_drawer_open: bool,
    pub(super) city_list_open: bool,
    pub(super) officer_browser_open: bool,
    pub(super) officer_browser_filters: OfficerBrowserFilters,
    pub(super) officer_detail_id: Option<OfficerId>,
    pub(super) retainers_open: bool,
    pub(super) shrine_open: bool,
    pub(super) shrine_tab: ShrineTab,
    pub(super) shrine_marriage_first: Option<OfficerId>,
    pub(super) shrine_marriage_second: Option<OfficerId>,
    pub(super) technology_open: bool,
    pub(super) events_open: bool,
    pub(super) selected_event_id: Option<String>,
    pub(super) event_message: String,
    pub(super) selected_technology_branch: TechnologyBranch,
    pub(super) selected_technology_id: TechnologyId,
    pub(super) retainer_filters: OfficerBrowserFilters,
    pub(super) reports_open: bool,
    pub(super) save_panel_open: bool,
    pub(super) main_menu_new_game_open: bool,
    pub(super) main_menu_load_game_open: bool,
    pub(super) main_menu_bgm_enabled: bool,
    pub(super) settings_open: bool,
    pub(super) settings_tab: SettingsTab,
    pub(super) shortcut_capture_action: Option<ShortcutAction>,
    pub(super) audio_output_devices: Vec<String>,
    pub(super) audio_output_devices_refresh_attempted: bool,
    pub(super) audio_output_devices_error: Option<String>,
    pub(super) officer_settings_open: bool,
    pub(super) officer_settings_game: Option<GameState>,
    pub(super) officer_settings_filters: OfficerBrowserFilters,
    pub(super) officer_settings_editable: bool,
    pub(super) officer_settings_selected_id: Option<OfficerId>,
    pub(super) officer_edit_open: bool,
    pub(super) officer_edit_draft: Option<OfficerEditDraft>,
    pub(super) officer_edit_error: Option<String>,
    pub(super) officer_portraits: OfficerPortraitStore,
    pub(super) game: Option<GameState>,
    pub(super) selected_faction_id: FactionId,
    pub(super) selected_city_id: Option<CityId>,
    pub(super) selected_city_tab: CityPanelTab,
    pub(super) selected_officers: BTreeMap<CityId, OfficerId>,
    pub(super) selected_focus: DevelopmentFocus,
    pub(super) selected_facility_kind: FacilityKind,
    pub(super) selected_recruit_kind: TroopKind,
    pub(super) recruit_amount: u32,
    pub(super) transfer_troops: TroopPool,
    pub(super) expedition_main_kind: TroopKind,
    pub(super) expedition_deputy_one_kind: TroopKind,
    pub(super) expedition_deputy_two_kind: TroopKind,
    pub(super) expedition_main_troops: u32,
    pub(super) expedition_deputy_one_troops: u32,
    pub(super) expedition_deputy_two_troops: u32,
    pub(super) expedition_food_supply: u32,
    pub(super) expedition_deputy_one: Option<OfficerId>,
    pub(super) expedition_deputy_two: Option<OfficerId>,
    pub(super) selected_recruitment_target: Option<OfficerId>,
    pub(super) selected_transfer_target: Option<CityId>,
    pub(super) selected_expedition_target: Option<CityId>,
    pub(super) selected_diplomacy_target: Option<FactionId>,
    pub(super) selected_diplomacy_proposal: DiplomacyProposal,
    pub(super) save_manager: SaveManager,
    pub(super) save_slots: Vec<SaveSlotMeta>,
    pub(super) save_slot_id: String,
    pub(super) save_display_name: String,
    pub(super) settings_store: GameSettingsStore,
    pub(super) applied_settings: GameSettings,
    pub(super) pending_settings: GameSettings,
    pub(super) message: String,
    pub(super) egui_font_configured: bool,
    pub(super) banner_logo: Option<MenuBannerLogo>,
    pub(super) banner_logo_error: Option<String>,
    pub(super) main_menu_illustrations: Vec<Option<MenuIllustration>>,
    pub(super) main_menu_illustration_errors: Vec<Option<String>>,
    pub(super) main_menu_hovered_illustration_index: Option<usize>,
    pub(super) main_menu_cloud_pattern: Option<MenuCloudPattern>,
    pub(super) main_menu_cloud_pattern_error: Option<String>,
}

pub(super) struct MenuBannerLogo {
    pub(super) texture: egui::TextureHandle,
    pub(super) crop_size: egui::Vec2,
}

pub(super) struct MenuIllustration {
    pub(super) texture_id: egui::TextureId,
    pub(super) crop_uv: egui::Rect,
    pub(super) crop_size: egui::Vec2,
}

pub(super) struct MenuCloudPattern {
    pub(super) texture_id: egui::TextureId,
    pub(super) size: egui::Vec2,
}

impl GameUiState {
    pub(super) fn new(
        settings_store: GameSettingsStore,
        loaded_settings: LoadedGameSettings,
    ) -> Self {
        let history_menu = load_history_menu(None);
        let selected_faction_id = history_menu
            .factions
            .iter()
            .find(|faction| faction.selectable)
            .map(|faction| faction.id.clone())
            .unwrap_or_default();
        let save_manager = SaveManager::with_default_dir();
        let save_slots = save_manager.list_slots().unwrap_or_default();
        let translator = Translator::new(loaded_settings.settings.general.ui_language);
        let (map_boundaries, map_boundary_message) = load_map_boundary_catalog(&translator);
        let settings_message = loaded_settings
            .message
            .as_ref()
            .map(|message| message.localized(&translator));
        let history_message = history_menu
            .message
            .as_ref()
            .map(|message| message.localized(&translator));
        let mut message =
            combined_menu_message(settings_message.as_deref(), history_message.as_deref());
        if let Some(boundary_message) = &map_boundary_message {
            if !message.is_empty() {
                message.push('\n');
            }
            message.push_str(boundary_message);
        }
        Self {
            history_scenarios: history_menu.scenarios,
            selected_scenario_id: history_menu.selected_scenario_id,
            history_factions: history_menu.factions,
            screen: Screen::MainMenu,
            selected_command_category: CommandCategory::Domestic,
            selected_command_action: CommandAction::Develop,
            map_zoom: 1.0,
            map_pan: egui::Vec2::ZERO,
            map_boundaries_enabled: true,
            map_boundaries,
            map_boundary_view_cache: MapBoundaryViewCache::default(),
            city_drawer_open: false,
            city_list_open: false,
            officer_browser_open: false,
            officer_browser_filters: OfficerBrowserFilters::default(),
            officer_detail_id: None,
            retainers_open: false,
            shrine_open: false,
            shrine_tab: ShrineTab::Succession,
            shrine_marriage_first: None,
            shrine_marriage_second: None,
            technology_open: false,
            events_open: false,
            selected_event_id: None,
            event_message: String::new(),
            selected_technology_branch: TechnologyBranch::Military,
            selected_technology_id: TechnologyId::MilitiaDrill,
            retainer_filters: OfficerBrowserFilters::default(),
            reports_open: true,
            save_panel_open: false,
            main_menu_new_game_open: false,
            main_menu_load_game_open: false,
            main_menu_bgm_enabled: true,
            settings_open: false,
            settings_tab: SettingsTab::Display,
            shortcut_capture_action: None,
            audio_output_devices: Vec::new(),
            audio_output_devices_refresh_attempted: false,
            audio_output_devices_error: None,
            officer_settings_open: false,
            officer_settings_game: None,
            officer_settings_filters: OfficerBrowserFilters::default(),
            officer_settings_editable: false,
            officer_settings_selected_id: None,
            officer_edit_open: false,
            officer_edit_draft: None,
            officer_edit_error: None,
            officer_portraits: OfficerPortraitStore::default(),
            game: None,
            selected_faction_id,
            selected_city_id: None,
            selected_city_tab: CityPanelTab::Overview,
            selected_officers: BTreeMap::new(),
            selected_focus: DevelopmentFocus::Agriculture,
            selected_facility_kind: FacilityKind::Farmland,
            selected_recruit_kind: TroopKind::Infantry,
            recruit_amount: 800,
            transfer_troops: TroopPool::from_total(500),
            expedition_main_kind: TroopKind::Infantry,
            expedition_deputy_one_kind: TroopKind::Archers,
            expedition_deputy_two_kind: TroopKind::Cavalry,
            expedition_main_troops: 1200,
            expedition_deputy_one_troops: 0,
            expedition_deputy_two_troops: 0,
            expedition_food_supply: 0,
            expedition_deputy_one: None,
            expedition_deputy_two: None,
            selected_recruitment_target: None,
            selected_transfer_target: None,
            selected_expedition_target: None,
            selected_diplomacy_target: None,
            selected_diplomacy_proposal: DiplomacyProposal::ImproveRelations,
            save_manager,
            save_slots,
            save_slot_id: "slot1".to_string(),
            save_display_name: translator.text("save-default-name"),
            settings_store,
            applied_settings: loaded_settings.settings.clone(),
            pending_settings: loaded_settings.settings,
            message,
            egui_font_configured: false,
            banner_logo: None,
            banner_logo_error: None,
            main_menu_illustrations: Vec::new(),
            main_menu_illustration_errors: Vec::new(),
            main_menu_hovered_illustration_index: None,
            main_menu_cloud_pattern: None,
            main_menu_cloud_pattern_error: None,
        }
    }
}

impl Default for GameUiState {
    fn default() -> Self {
        let settings_store = GameSettingsStore::with_default_path();
        let loaded_settings = settings_store.load();
        Self::new(settings_store, loaded_settings)
    }
}

pub(super) fn combined_menu_message(
    settings_message: Option<&str>,
    history_message: Option<&str>,
) -> String {
    match (
        settings_message.filter(|message| !message.is_empty()),
        history_message.filter(|message| !message.is_empty()),
    ) {
        (Some(settings), Some(history)) => format!("{settings}\n{history}"),
        (Some(settings), None) => settings.to_string(),
        (None, Some(history)) => history.to_string(),
        (None, None) => String::new(),
    }
}

pub(super) fn load_map_boundary_catalog(
    t: &Translator,
) -> (Option<MapBoundaryCatalog>, Option<String>) {
    match MapBoundaryCatalog::from_path(asset_path("data/map_boundaries.json")) {
        Ok(catalog) => (Some(catalog), None),
        Err(error) => (
            None,
            Some(t.text_args(
                "message-map-boundary-unavailable",
                &super::i18n::args([("error", error.to_string())]),
            )),
        ),
    }
}

pub(super) struct HistoryMenuState {
    scenarios: Vec<HistoricalScenario>,
    selected_scenario_id: ScenarioId,
    factions: Vec<Faction>,
    message: Option<HistoryMenuLoadMessage>,
}

pub(super) enum HistoryMenuLoadMessage {
    CatalogUnavailable { error: String },
}

impl HistoryMenuLoadMessage {
    fn localized(&self, t: &Translator) -> String {
        match self {
            Self::CatalogUnavailable { error } => t.text_args(
                "message-history-catalog-unavailable",
                &args([("error", error.clone())]),
            ),
        }
    }
}

pub(super) fn load_history_menu(preferred_scenario_id: Option<&str>) -> HistoryMenuState {
    match SqliteHistoricalCatalog::open_default().and_then(|catalog| {
        let scenarios = catalog.scenarios()?;
        let selected_scenario_id = preferred_scenario_id
            .filter(|id| scenarios.iter().any(|scenario| scenario.id == *id))
            .map(str::to_string)
            .or_else(|| scenarios.first().map(|scenario| scenario.id.clone()))
            .unwrap_or_default();
        let factions = if selected_scenario_id.is_empty() {
            Vec::new()
        } else {
            catalog.selectable_factions(&selected_scenario_id)?
        };
        Ok(HistoryMenuState {
            scenarios,
            selected_scenario_id,
            factions,
            message: None,
        })
    }) {
        Ok(menu) => menu,
        Err(error) => HistoryMenuState {
            scenarios: Vec::new(),
            selected_scenario_id: String::new(),
            factions: Vec::new(),
            message: Some(HistoryMenuLoadMessage::CatalogUnavailable {
                error: error.to_string(),
            }),
        },
    }
}

pub(super) fn refresh_history_menu(ui_state: &mut GameUiState) {
    let menu = load_history_menu(Some(&ui_state.selected_scenario_id));
    ui_state.history_scenarios = menu.scenarios;
    ui_state.selected_scenario_id = menu.selected_scenario_id;
    ui_state.history_factions = menu.factions;
    if let Some(message) = menu.message {
        let t = Translator::new(ui_state.applied_settings.general.ui_language);
        ui_state.message = message.localized(&t);
    }
    ensure_selected_faction(ui_state);
}

pub(super) fn refresh_history_factions(ui_state: &mut GameUiState) {
    match SqliteHistoricalCatalog::open_default()
        .and_then(|catalog| catalog.selectable_factions(&ui_state.selected_scenario_id))
    {
        Ok(factions) => {
            ui_state.history_factions = factions;
            ensure_selected_faction(ui_state);
        }
        Err(error) => {
            ui_state.history_factions.clear();
            let t = Translator::new(ui_state.applied_settings.general.ui_language);
            ui_state.message = t.text_args(
                "message-history-factions-load-failed",
                &args([("error", error.to_string())]),
            );
        }
    }
}

pub(super) fn ensure_selected_faction(ui_state: &mut GameUiState) {
    let current_valid = ui_state
        .history_factions
        .iter()
        .any(|faction| faction.selectable && faction.id == ui_state.selected_faction_id);
    if !current_valid {
        if let Some(faction) = ui_state
            .history_factions
            .iter()
            .find(|faction| faction.selectable)
        {
            ui_state.selected_faction_id = faction.id.clone();
        } else {
            ui_state.selected_faction_id.clear();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Screen {
    MainMenu,
    InGame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommandCategory {
    Domestic,
    Military,
    Diplomacy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum CityPanelTab {
    #[default]
    Overview,
    Domestic,
    Military,
    Diplomacy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CommandAction {
    Develop,
    UpgradeCityCore,
    BuildFacility,
    Recruit,
    RecruitOfficer,
    Train,
    AppointGovernor,
    Transfer,
    Expedition,
    Diplomacy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum OfficerGenderFilter {
    #[default]
    All,
    Male,
    Female,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum OfficerStatusFilter {
    #[default]
    All,
    Active,
    Minor,
    Wild,
    Unavailable,
    Dead,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ShrineTab {
    #[default]
    Succession,
    Marriage,
    Children,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OfficerBrowserFilters {
    pub(super) search: String,
    pub(super) gender: OfficerGenderFilter,
    pub(super) faction_id: Option<FactionId>,
    pub(super) status: OfficerStatusFilter,
    pub(super) city_id: Option<CityId>,
}

impl OfficerBrowserFilters {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OfficerEditDraft {
    pub(super) id: OfficerId,
    pub(super) name: String,
    pub(super) courtesy_name: String,
    pub(super) native_place: String,
    pub(super) birth_year: String,
    pub(super) death_year: String,
    pub(super) gender: OfficerGender,
    pub(super) leadership: u8,
    pub(super) strength: u8,
    pub(super) intelligence: u8,
    pub(super) politics: u8,
    pub(super) charm: u8,
    pub(super) tags: String,
    pub(super) confidence: SourceConfidence,
    pub(super) biography: String,
    pub(super) notes: String,
}

impl OfficerEditDraft {
    pub(super) fn from_profile(profile: &OfficerProfile) -> Self {
        Self {
            id: profile.id.clone(),
            name: profile.name.clone(),
            courtesy_name: profile.courtesy_name.clone().unwrap_or_default(),
            native_place: profile.native_place.clone().unwrap_or_default(),
            birth_year: profile
                .birth_year
                .map(|year| year.to_string())
                .unwrap_or_default(),
            death_year: profile
                .death_year
                .map(|year| year.to_string())
                .unwrap_or_default(),
            gender: profile.gender.clone(),
            leadership: profile.stats.leadership,
            strength: profile.stats.strength,
            intelligence: profile.stats.intelligence,
            politics: profile.stats.politics,
            charm: profile.stats.charm,
            tags: profile.tags.join(","),
            confidence: profile.confidence.clone(),
            biography: profile.biography.clone(),
            notes: profile.notes.clone(),
        }
    }

    pub(super) fn from_officer(officer: &Officer) -> Self {
        if let Some(profile) = officer.profile.as_ref() {
            Self::from_profile(profile)
        } else {
            Self {
                id: officer.id.clone(),
                name: officer.name.clone(),
                courtesy_name: String::new(),
                native_place: String::new(),
                birth_year: if officer.birth_year != 0 {
                    officer.birth_year.to_string()
                } else {
                    String::new()
                },
                death_year: String::new(),
                gender: officer.gender.clone(),
                leadership: officer.stats.leadership,
                strength: officer.stats.strength,
                intelligence: officer.stats.intelligence,
                politics: officer.stats.politics,
                charm: officer.stats.charm,
                tags: String::new(),
                confidence: SourceConfidence::Medium,
                biography: String::new(),
                notes: String::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_officer() -> Officer {
        Officer {
            id: "test_officer".to_string(),
            name: "测试武将".to_string(),
            faction_id: "test_faction".to_string(),
            city_id: None,
            office_id: None,
            stats: OfficerStats {
                leadership: 71,
                strength: 82,
                intelligence: 63,
                politics: 54,
                charm: 75,
            },
            loyalty: 80,
            birth_year: 180,
            gender: OfficerGender::Female,
            status: OfficerStatus::Active,
            profile: None,
        }
    }

    #[test]
    fn officer_edit_draft_can_be_built_from_current_officer_fields() {
        let officer = test_officer();

        let draft = OfficerEditDraft::from_officer(&officer);

        assert_eq!(draft.id, "test_officer");
        assert_eq!(draft.name, "测试武将");
        assert_eq!(draft.birth_year, "180");
        assert_eq!(draft.gender, OfficerGender::Female);
        assert_eq!(draft.leadership, 71);
        assert_eq!(draft.strength, 82);
        assert_eq!(draft.confidence, SourceConfidence::Medium);
        assert!(draft.biography.is_empty());
    }

    #[test]
    fn officer_edit_draft_prefers_embedded_profile_for_portrait_context() {
        let mut officer = test_officer();
        officer.profile = Some(OfficerProfile {
            id: "test_officer".to_string(),
            name: "史料名".to_string(),
            courtesy_name: Some("文远".to_string()),
            native_place: Some("雁门".to_string()),
            birth_year: Some(169),
            death_year: Some(222),
            gender: OfficerGender::Male,
            stats: OfficerStats {
                leadership: 90,
                strength: 91,
                intelligence: 78,
                politics: 62,
                charm: 76,
            },
            tags: vec!["famous_general".to_string()],
            confidence: SourceConfidence::High,
            biography: "以勇略闻名。".to_string(),
            relationships: Vec::new(),
            notes: "profile note".to_string(),
        });

        let draft = OfficerEditDraft::from_officer(&officer);

        assert_eq!(draft.name, "史料名");
        assert_eq!(draft.courtesy_name, "文远");
        assert_eq!(draft.birth_year, "169");
        assert_eq!(draft.death_year, "222");
        assert_eq!(draft.gender, OfficerGender::Male);
        assert_eq!(draft.leadership, 90);
        assert_eq!(draft.tags, "famous_general");
        assert_eq!(draft.confidence, SourceConfidence::High);
        assert_eq!(draft.biography, "以勇略闻名。");
    }
}
