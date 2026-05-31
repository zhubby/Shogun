use crate::game::*;
use bevy::prelude::Resource;
use bevy_egui::egui;
use std::collections::BTreeMap;

use super::display_settings::{DisplaySettings, DisplaySettingsStore, LoadedDisplaySettings};
use super::map::MapBoundaryViewCache;

#[derive(Resource)]
pub(super) struct GameUiState {
    pub(super) json_scenario: ScenarioData,
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
    pub(super) retainers_open: bool,
    pub(super) technology_open: bool,
    pub(super) selected_technology_branch: TechnologyBranch,
    pub(super) selected_technology_id: TechnologyId,
    pub(super) retainer_filters: OfficerBrowserFilters,
    pub(super) reports_open: bool,
    pub(super) save_panel_open: bool,
    pub(super) main_menu_new_game_open: bool,
    pub(super) main_menu_load_game_open: bool,
    pub(super) main_menu_bgm_enabled: bool,
    pub(super) settings_open: bool,
    pub(super) officer_settings_open: bool,
    pub(super) officer_settings_game: Option<GameState>,
    pub(super) officer_settings_filters: OfficerBrowserFilters,
    pub(super) officer_settings_editable: bool,
    pub(super) officer_settings_selected_id: Option<OfficerId>,
    pub(super) officer_edit_open: bool,
    pub(super) officer_edit_draft: Option<OfficerEditDraft>,
    pub(super) officer_edit_error: Option<String>,
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
    pub(super) expedition_deputy_one: Option<OfficerId>,
    pub(super) expedition_deputy_two: Option<OfficerId>,
    pub(super) selected_transfer_target: Option<CityId>,
    pub(super) selected_expedition_target: Option<CityId>,
    pub(super) selected_diplomacy_target: Option<FactionId>,
    pub(super) selected_diplomacy_proposal: DiplomacyProposal,
    pub(super) save_manager: SaveManager,
    pub(super) save_slots: Vec<SaveSlotMeta>,
    pub(super) save_slot_id: String,
    pub(super) save_display_name: String,
    pub(super) settings_store: DisplaySettingsStore,
    pub(super) applied_settings: DisplaySettings,
    pub(super) pending_settings: DisplaySettings,
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
    pub(super) crop_uv: egui::Rect,
    pub(super) crop_size: egui::Vec2,
}

pub(super) struct MenuIllustration {
    pub(super) texture: egui::TextureHandle,
    pub(super) crop_uv: egui::Rect,
    pub(super) crop_size: egui::Vec2,
}

pub(super) struct MenuCloudPattern {
    pub(super) texture: egui::TextureHandle,
    pub(super) size: egui::Vec2,
}

impl GameUiState {
    pub(super) fn new(
        settings_store: DisplaySettingsStore,
        loaded_settings: LoadedDisplaySettings,
    ) -> Self {
        let json_scenario = ScenarioData::from_path("assets/scenarios/early_three_kingdoms.json")
            .or_else(|_| ScenarioData::default_scenario())
            .expect("默认剧本必须可加载");
        let history_menu = load_history_menu(None);
        let selected_faction_id = history_menu
            .factions
            .iter()
            .find(|faction| faction.selectable)
            .map(|faction| faction.id.clone())
            .or_else(|| json_scenario.player_selectable_factions.first().cloned())
            .unwrap_or_default();
        let save_manager = SaveManager::with_default_dir();
        let save_slots = save_manager.list_slots().unwrap_or_default();
        let (map_boundaries, map_boundary_message) = load_map_boundary_catalog();
        let mut message =
            combined_menu_message(loaded_settings.message.as_deref(), &history_menu.message);
        if let Some(boundary_message) = &map_boundary_message {
            if !message.is_empty() {
                message.push('\n');
            }
            message.push_str(boundary_message);
        }
        Self {
            json_scenario,
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
            retainers_open: false,
            technology_open: false,
            selected_technology_branch: TechnologyBranch::Military,
            selected_technology_id: TechnologyId::MilitiaDrill,
            retainer_filters: OfficerBrowserFilters::default(),
            reports_open: true,
            save_panel_open: false,
            main_menu_new_game_open: false,
            main_menu_load_game_open: false,
            main_menu_bgm_enabled: true,
            settings_open: false,
            officer_settings_open: false,
            officer_settings_game: None,
            officer_settings_filters: OfficerBrowserFilters::default(),
            officer_settings_editable: false,
            officer_settings_selected_id: None,
            officer_edit_open: false,
            officer_edit_draft: None,
            officer_edit_error: None,
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
            expedition_deputy_one: None,
            expedition_deputy_two: None,
            selected_transfer_target: None,
            selected_expedition_target: None,
            selected_diplomacy_target: None,
            selected_diplomacy_proposal: DiplomacyProposal::ImproveRelations,
            save_manager,
            save_slots,
            save_slot_id: "slot1".to_string(),
            save_display_name: "新存档".to_string(),
            settings_store,
            applied_settings: loaded_settings.settings,
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
        let settings_store = DisplaySettingsStore::with_default_path();
        let loaded_settings = settings_store.load();
        Self::new(settings_store, loaded_settings)
    }
}

pub(super) fn combined_menu_message(
    settings_message: Option<&str>,
    history_message: &str,
) -> String {
    match (
        settings_message.filter(|message| !message.is_empty()),
        history_message.is_empty(),
    ) {
        (Some(message), false) => format!("{message}\n{history_message}"),
        (Some(message), true) => message.to_string(),
        (None, false) => history_message.to_string(),
        (None, true) => String::new(),
    }
}

pub(super) fn load_map_boundary_catalog() -> (Option<MapBoundaryCatalog>, Option<String>) {
    match MapBoundaryCatalog::from_path(MAP_BOUNDARY_ASSET_PATH) {
        Ok(catalog) => (Some(catalog), None),
        Err(error) => (
            None,
            Some(format!("州郡边界不可用，已退回点线地图: {error}")),
        ),
    }
}

pub(super) struct HistoryMenuState {
    scenarios: Vec<HistoricalScenario>,
    selected_scenario_id: ScenarioId,
    factions: Vec<Faction>,
    message: String,
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
            message: String::new(),
        })
    }) {
        Ok(menu) => menu,
        Err(error) => HistoryMenuState {
            scenarios: Vec::new(),
            selected_scenario_id: String::new(),
            factions: Vec::new(),
            message: format!("历史资料库不可用，已启用兼容小剧本: {error}"),
        },
    }
}

pub(super) fn refresh_history_menu(ui_state: &mut GameUiState) {
    let menu = load_history_menu(Some(&ui_state.selected_scenario_id));
    ui_state.history_scenarios = menu.scenarios;
    ui_state.selected_scenario_id = menu.selected_scenario_id;
    ui_state.history_factions = menu.factions;
    if !menu.message.is_empty() {
        ui_state.message = menu.message;
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
            ui_state.message = format!("读取势力列表失败: {error}");
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
            return;
        }
        if let Some(faction_id) = ui_state
            .json_scenario
            .player_selectable_factions
            .first()
            .cloned()
        {
            ui_state.selected_faction_id = faction_id;
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
    Wild,
    Unavailable,
    Dead,
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
}
