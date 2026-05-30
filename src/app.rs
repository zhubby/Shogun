use crate::app_settings::{
    AppSettings, AppSettingsStore, DisplayMode, DisplayResolution, LoadedAppSettings,
};
use crate::game::{Command as GameCommand, *};
use bevy::prelude::*;
use bevy::window::{EnabledButtons, PrimaryWindow};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const MAP_MIN_ZOOM: f32 = 0.65;
const MAP_MAX_ZOOM: f32 = 5.0;
const MAP_ZOOM_STEP: f32 = 1.2;
const HUD_MARGIN: f32 = 16.0;
const HUD_TOP_OFFSET: f32 = 14.0;
const HUD_TOP_HEIGHT: f32 = 68.0;
const CITY_DRAWER_WIDTH: f32 = 390.0;

pub fn run() {
    let settings_store = AppSettingsStore::with_default_path();
    let loaded_settings = settings_store.load();
    let initial_settings = loaded_settings.settings;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "大将军 Shogun".to_string(),
                resolution: initial_settings.window_resolution(),
                mode: initial_settings.window_mode(),
                present_mode: initial_settings.present_mode(),
                resizable: false,
                enabled_buttons: EnabledButtons {
                    maximize: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(GameUiState::new(settings_store, loaded_settings))
        .add_systems(Startup, setup_camera)
        .add_systems(EguiPrimaryContextPass, game_ui_system)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Resource)]
struct GameUiState {
    json_scenario: ScenarioData,
    history_scenarios: Vec<HistoricalScenario>,
    selected_scenario_id: ScenarioId,
    history_factions: Vec<Faction>,
    screen: Screen,
    city_tab: CityTab,
    map_zoom: f32,
    map_pan: egui::Vec2,
    map_boundaries_enabled: bool,
    map_boundaries: Option<MapBoundaryCatalog>,
    city_drawer_open: bool,
    city_list_open: bool,
    reports_open: bool,
    save_panel_open: bool,
    settings_open: bool,
    game: Option<GameState>,
    selected_faction_id: FactionId,
    selected_city_id: Option<CityId>,
    selected_officers: BTreeMap<CityId, OfficerId>,
    selected_focus: DevelopmentFocus,
    recruit_amount: u32,
    transfer_troops: u32,
    expedition_troops: u32,
    selected_transfer_target: Option<CityId>,
    selected_expedition_target: Option<CityId>,
    selected_diplomacy_target: Option<FactionId>,
    selected_diplomacy_proposal: DiplomacyProposal,
    save_manager: SaveManager,
    save_slots: Vec<SaveSlotMeta>,
    save_slot_id: String,
    save_display_name: String,
    settings_store: AppSettingsStore,
    applied_settings: AppSettings,
    pending_settings: AppSettings,
    message: String,
    egui_font_configured: bool,
}

impl GameUiState {
    fn new(settings_store: AppSettingsStore, loaded_settings: LoadedAppSettings) -> Self {
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
            city_tab: CityTab::Construction,
            map_zoom: 1.0,
            map_pan: egui::Vec2::ZERO,
            map_boundaries_enabled: true,
            map_boundaries,
            city_drawer_open: false,
            city_list_open: false,
            reports_open: true,
            save_panel_open: false,
            settings_open: false,
            game: None,
            selected_faction_id,
            selected_city_id: None,
            selected_officers: BTreeMap::new(),
            selected_focus: DevelopmentFocus::Agriculture,
            recruit_amount: 800,
            transfer_troops: 500,
            expedition_troops: 1200,
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
        }
    }
}

impl Default for GameUiState {
    fn default() -> Self {
        let settings_store = AppSettingsStore::with_default_path();
        let loaded_settings = settings_store.load();
        Self::new(settings_store, loaded_settings)
    }
}

fn combined_menu_message(settings_message: Option<&str>, history_message: &str) -> String {
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

fn load_map_boundary_catalog() -> (Option<MapBoundaryCatalog>, Option<String>) {
    match MapBoundaryCatalog::from_path(MAP_BOUNDARY_ASSET_PATH) {
        Ok(catalog) => (Some(catalog), None),
        Err(error) => (
            None,
            Some(format!("州郡边界不可用，已退回点线地图: {error}")),
        ),
    }
}

struct HistoryMenuState {
    scenarios: Vec<HistoricalScenario>,
    selected_scenario_id: ScenarioId,
    factions: Vec<Faction>,
    message: String,
}

fn load_history_menu(preferred_scenario_id: Option<&str>) -> HistoryMenuState {
    match SqliteHistoricalCatalog::open_asset().and_then(|catalog| {
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

fn refresh_history_menu(ui_state: &mut GameUiState) {
    let menu = load_history_menu(Some(&ui_state.selected_scenario_id));
    ui_state.history_scenarios = menu.scenarios;
    ui_state.selected_scenario_id = menu.selected_scenario_id;
    ui_state.history_factions = menu.factions;
    if !menu.message.is_empty() {
        ui_state.message = menu.message;
    }
    ensure_selected_faction(ui_state);
}

fn refresh_history_factions(ui_state: &mut GameUiState) {
    match SqliteHistoricalCatalog::open_asset()
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

fn ensure_selected_faction(ui_state: &mut GameUiState) {
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
enum Screen {
    MainMenu,
    InGame,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CityTab {
    Construction,
    Governance,
}

fn game_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<GameUiState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    configure_egui_fonts(ctx, &mut ui_state);
    configure_egui_theme(ctx);

    match ui_state.screen {
        Screen::MainMenu => {
            if main_menu(ctx, &mut ui_state) {
                match windows.single_mut() {
                    Ok(mut window) => apply_pending_display_settings(&mut ui_state, &mut window),
                    Err(_) => ui_state.message = "找不到主窗口，无法应用显示设置".to_string(),
                }
            }
        }
        Screen::InGame => in_game(ctx, &mut ui_state),
    }
}

fn main_menu(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
    let mut apply_display_settings = false;
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter_at(rect);
            draw_menu_background(&painter, rect);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(34.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("大将军")
                                .size(42.0)
                                .color(war_gold())
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("Shogun")
                                .size(18.0)
                                .color(war_text_muted()),
                        );
                    });
                    ui.add_space(24.0);

                    let total_width = (ui.available_width() - HUD_MARGIN * 2.0).min(1060.0);
                    let stacked_menu = total_width < 900.0;
                    let panel_width = if stacked_menu {
                        total_width
                    } else {
                        (total_width - 18.0) * 0.5
                    };
                    let left_pad = ((ui.available_width() - total_width) * 0.5).max(HUD_MARGIN);

                    if stacked_menu {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                new_game_menu(ui, ui_state);
                            });
                        });
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                load_game_menu(ui, ui_state);
                            });
                        });
                    } else {
                        ui.horizontal_top(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                new_game_menu(ui, ui_state);
                            });
                            ui.add_space(18.0);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(panel_width);
                                load_game_menu(ui, ui_state);
                            });
                        });
                    }

                    if !ui_state.message.is_empty() {
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            ui.add_space(left_pad);
                            war_panel_frame().show(ui, |ui| {
                                ui.set_width(total_width);
                                ui.colored_label(war_gold(), &ui_state.message);
                            });
                        });
                    }
                });
        });
    main_menu_settings_button(ctx, ui_state);
    if ui_state.settings_open {
        apply_display_settings |= settings_modal(ctx, ui_state);
    }
    apply_display_settings
}

fn new_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.heading(egui::RichText::new("新开局").color(war_gold()));
        ui.add_space(8.0);
        if !ui_state.history_scenarios.is_empty() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("剧本").color(war_text_muted()));
                if ui.button("刷新资料库").clicked() {
                    refresh_history_menu(ui_state);
                }
            });
            let mut scenario_changed = false;
            egui::ScrollArea::vertical()
                .id_salt("main_menu_scenarios")
                .max_height(190.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for scenario in ui_state.history_scenarios.clone() {
                        let label = format!(
                            "{} ({}年{}月)",
                            scenario.name, scenario.year, scenario.month
                        );
                        if ui
                            .radio_value(&mut ui_state.selected_scenario_id, scenario.id, label)
                            .changed()
                        {
                            scenario_changed = true;
                        }
                    }
                });
            if scenario_changed {
                refresh_history_factions(ui_state);
            }

            ui.add_space(10.0);
            ui.label(egui::RichText::new("势力").color(war_text_muted()));
            egui::ScrollArea::vertical()
                .id_salt("main_menu_factions")
                .max_height(160.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for faction in ui_state
                        .history_factions
                        .iter()
                        .filter(|faction| faction.selectable)
                        .cloned()
                        .collect::<Vec<_>>()
                    {
                        ui.radio_value(&mut ui_state.selected_faction_id, faction.id, faction.name);
                    }
                });
            ui.add_space(12.0);
            if ui
                .add_sized([ui.available_width(), 34.0], egui::Button::new("开始游戏"))
                .clicked()
            {
                start_history_game(ui_state);
            }
        } else {
            ui.label(egui::RichText::new("选择势力").color(war_text_muted()));
            for faction_id in &ui_state.json_scenario.player_selectable_factions {
                let faction_name = ui_state
                    .json_scenario
                    .factions
                    .iter()
                    .find(|faction| &faction.id == faction_id)
                    .map(|faction| faction.name.as_str())
                    .unwrap_or(faction_id);
                ui.radio_value(
                    &mut ui_state.selected_faction_id,
                    faction_id.clone(),
                    faction_name,
                );
            }
            ui.add_space(12.0);
            if ui
                .add_sized(
                    [ui.available_width(), 34.0],
                    egui::Button::new("开始兼容小剧本"),
                )
                .clicked()
            {
                start_json_game(ui_state);
            }
        }
    });
}

fn load_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.heading(egui::RichText::new("读取存档").color(war_gold()));
        ui.label(
            egui::RichText::new(format!(
                "目录: {}",
                ui_state.save_manager.base_dir().display()
            ))
            .color(war_text_muted()),
        );
        if ui.button("刷新存档列表").clicked() {
            refresh_saves(ui_state);
        }
        ui.add_space(8.0);
        let slots = ui_state.save_slots.clone();
        if slots.is_empty() {
            ui.label("暂无存档");
        }
        egui::ScrollArea::vertical()
            .id_salt("main_menu_saves")
            .max_height(430.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                for slot in slots {
                    war_sub_panel_frame().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(format!(
                            "{} - {}年{}月 第{}回合",
                            slot.display_name, slot.year, slot.month, slot.turn
                        ));
                        ui.horizontal(|ui| {
                            if ui.button("读取").clicked() {
                                match ui_state.save_manager.load_slot(&slot.slot_id) {
                                    Ok(game) => enter_game(
                                        ui_state,
                                        game,
                                        format!("读取存档 {}", slot.display_name),
                                    ),
                                    Err(error) => ui_state.message = error.to_string(),
                                }
                            }
                            if ui.button("删除").clicked() {
                                match ui_state.save_manager.delete_slot(&slot.slot_id) {
                                    Ok(()) => {
                                        refresh_saves(ui_state);
                                        ui_state.message =
                                            format!("删除存档 {}", slot.display_name);
                                    }
                                    Err(error) => ui_state.message = error.to_string(),
                                }
                            }
                        });
                    });
                    ui.add_space(6.0);
                }
            });
    });
}

fn main_menu_settings_button(ctx: &egui::Context, ui_state: &mut GameUiState) {
    if ui_state.settings_open {
        return;
    }

    egui::Area::new(egui::Id::new("main_menu_settings_button"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET),
        )
        .show(ctx, |ui| {
            war_bar_frame().show(ui, |ui| {
                if ui
                    .add_sized([86.0, 32.0], egui::Button::new("设置"))
                    .clicked()
                {
                    ui_state.settings_open = true;
                }
            });
        });
}

fn settings_modal(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
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
    let modal_width = (screen.width() - HUD_MARGIN * 2.0).clamp(320.0, 560.0);
    egui::Area::new(egui::Id::new("settings_modal"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("设置").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("关闭").clicked() {
                            ui_state.settings_open = false;
                        }
                    });
                });
                ui.separator();
                apply_settings |= settings_controls(ui, ui_state);
            });
        });
    apply_settings
}

fn settings_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) -> bool {
    let mut apply_settings = false;
    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(format!(
                "配置: {}",
                ui_state.settings_store.path().display()
            ))
            .color(war_text_muted()),
        );
        ui.add_space(8.0);

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("分辨率").color(war_text_muted()));
            egui::ComboBox::from_id_salt("display_resolution")
                .selected_text(ui_state.pending_settings.resolution.to_string())
                .show_ui(ui, |ui| {
                    for resolution in DisplayResolution::presets() {
                        ui.selectable_value(
                            &mut ui_state.pending_settings.resolution,
                            *resolution,
                            resolution.to_string(),
                        );
                    }
                });
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("显示模式").color(war_text_muted()));
            for mode in DisplayMode::variants() {
                ui.radio_value(
                    &mut ui_state.pending_settings.display_mode,
                    *mode,
                    mode.label(),
                );
            }
        });

        ui.checkbox(&mut ui_state.pending_settings.vsync, "垂直同步");

        if ui_state.pending_settings != ui_state.applied_settings {
            ui.colored_label(war_gold(), "有未应用更改");
        }

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_sized([132.0, 34.0], egui::Button::new("应用并保存"))
                .clicked()
            {
                apply_settings = true;
                ui_state.settings_open = false;
            }
            if ui.button("恢复默认").clicked() {
                ui_state.pending_settings = AppSettings::default();
                ui_state.message = "显示设置已恢复默认，点击应用并保存生效".to_string();
            }
        });
    });
    apply_settings
}

fn apply_pending_display_settings(ui_state: &mut GameUiState, window: &mut Window) {
    let settings = ui_state.pending_settings;
    settings.apply_to_window(window);
    ui_state.applied_settings = settings;
    match ui_state.settings_store.save(settings) {
        Ok(()) => {
            ui_state.message = format!(
                "显示设置已保存到 {}",
                ui_state.settings_store.path().display()
            );
        }
        Err(error) => {
            ui_state.message = format!("显示设置已应用，但保存失败: {error}");
        }
    }
}

fn start_history_game(ui_state: &mut GameUiState) {
    if ui_state.selected_scenario_id.is_empty() {
        ui_state.message = "没有可用的 SQLite 历史剧本".to_string();
        return;
    }
    match SqliteHistoricalCatalog::open_asset().and_then(|catalog| {
        catalog.build_game(
            &ui_state.selected_scenario_id,
            &ui_state.selected_faction_id,
        )
    }) {
        Ok(game) => enter_game(ui_state, game, "新游戏开始".to_string()),
        Err(error) => ui_state.message = error.to_string(),
    }
}

fn start_json_game(ui_state: &mut GameUiState) {
    match ui_state
        .json_scenario
        .build_game(&ui_state.selected_faction_id)
    {
        Ok(game) => enter_game(ui_state, game, "兼容小剧本开始".to_string()),
        Err(error) => ui_state.message = error.to_string(),
    }
}

fn enter_game(ui_state: &mut GameUiState, game: GameState, message: String) {
    ui_state.selected_city_id = first_player_city(&game);
    ui_state.selected_officers.clear();
    ui_state.selected_transfer_target = None;
    ui_state.selected_expedition_target = None;
    ui_state.selected_diplomacy_target = None;
    ui_state.city_tab = CityTab::Construction;
    ui_state.city_drawer_open = ui_state.selected_city_id.is_some();
    ui_state.city_list_open = false;
    ui_state.reports_open = true;
    ui_state.save_panel_open = false;
    reset_map_view(ui_state);
    ui_state.game = Some(game);
    ui_state.screen = Screen::InGame;
    ui_state.message = message;
}

fn in_game(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            map_panel(ui, ui_state);
        });

    in_game_hud(ctx, ui_state);
}

fn in_game_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    let screen = ctx.content_rect();
    top_status_hud(ctx, ui_state, screen);
    left_map_hud(ctx, ui_state);
    city_list_hud(ctx, ui_state, screen);
    save_hud(ctx, ui_state, screen);
    city_drawer_hud(ctx, ui_state, screen);
    report_hud(ctx, ui_state, screen);
}

fn top_status_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    let width = (screen.width() - HUD_MARGIN * 2.0).max(320.0);
    let summary = ui_state.game.as_ref().map(|game| {
        let faction_name = game
            .factions
            .get(&game.player_faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| "未知势力".to_string());
        let status = match &game.status {
            GameStatus::Running => None,
            GameStatus::Victory { reason } => Some(format!("胜利: {reason}")),
            GameStatus::Defeat { reason } => Some(format!("失败: {reason}")),
        };
        (
            game.scenario_name.clone(),
            game.year,
            game.month,
            game.turn,
            faction_name,
            status,
        )
    });

    egui::Area::new(egui::Id::new("hud_top_status"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET),
        )
        .show(ctx, |ui| {
            war_bar_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("大将军")
                            .size(24.0)
                            .color(war_gold())
                            .strong(),
                    );
                    ui.separator();
                    if let Some((scenario, year, month, turn, faction_name, status)) = summary {
                        ui.label(format!("{scenario}  {year}年{month}月  第{turn}回合"));
                        ui.label(format!("玩家: {faction_name}"));
                        if let Some(status) = status {
                            ui.colored_label(egui::Color32::from_rgb(200, 72, 52), status);
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("主菜单").clicked() {
                            ui_state.screen = Screen::MainMenu;
                        }
                        if ui.button("存档").clicked() {
                            ui_state.save_panel_open = !ui_state.save_panel_open;
                        }
                        if ui.button("清空命令").clicked() {
                            clear_pending_commands(ui_state);
                        }
                        if ui.button("结束本月").clicked() {
                            finish_current_turn(ui_state);
                        }
                    });
                });
            });
        });
}

fn left_map_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::Area::new(egui::Id::new("hud_left_map_tools"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(285.0);
                map_controls(ui, ui_state);
                ui.separator();
                selected_city_summary(ui, ui_state);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui
                        .button(if ui_state.city_list_open {
                            "收起城池"
                        } else {
                            "城池一览"
                        })
                        .clicked()
                    {
                        ui_state.city_list_open = !ui_state.city_list_open;
                    }
                    if ui.button("战报").clicked() {
                        ui_state.reports_open = !ui_state.reports_open;
                    }
                });
            });
        });
}

fn city_list_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    if !ui_state.city_list_open {
        return;
    }
    let max_height = (screen.height() - HUD_TOP_HEIGHT - 170.0).clamp(240.0, 520.0);
    egui::Area::new(egui::Id::new("hud_city_list"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 210.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(285.0);
                ui.set_max_height(max_height);
                city_list(ui, ui_state);
            });
        });
}

fn save_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    if !ui_state.save_panel_open {
        return;
    }
    let x_offset = if ui_state.city_drawer_open && screen.width() > 860.0 {
        -(CITY_DRAWER_WIDTH + HUD_MARGIN + 18.0)
    } else {
        -HUD_MARGIN
    };
    egui::Area::new(egui::Id::new("hud_save_panel"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(x_offset, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(330.0);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("存档").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("收起").clicked() {
                            ui_state.save_panel_open = false;
                        }
                    });
                });
                save_controls(ui, ui_state);
            });
        });
}

fn city_drawer_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    if !ui_state.city_drawer_open {
        return;
    }
    let max_height = (screen.height() - HUD_TOP_HEIGHT - 48.0).max(360.0);
    egui::Area::new(egui::Id::new("hud_city_drawer"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                let drawer_width = CITY_DRAWER_WIDTH
                    .min(screen.width() - HUD_MARGIN * 2.0)
                    .max(300.0);
                ui.set_width(drawer_width);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("军令").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("收起").clicked() {
                            ui_state.city_drawer_open = false;
                        }
                    });
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .max_height(max_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        selected_city_panel(ui, ui_state);
                    });
            });
        });
}

fn report_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
    let width = (screen.width() * 0.62).clamp(420.0, 880.0);
    egui::Area::new(egui::Id::new("hud_report_panel"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_BOTTOM,
            egui::vec2(HUD_MARGIN, -HUD_MARGIN),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("回合报告").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(if ui_state.reports_open {
                                "收起"
                            } else {
                                "展开"
                            })
                            .clicked()
                        {
                            ui_state.reports_open = !ui_state.reports_open;
                        }
                    });
                });
                if ui_state.reports_open {
                    ui.separator();
                    report_panel(ui, ui_state, screen);
                } else if !ui_state.message.is_empty() {
                    ui.label(&ui_state.message);
                }
            });
        });
}

fn finish_current_turn(ui_state: &mut GameUiState) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = "尚未开始游戏".to_string();
        return;
    };
    if game.status != GameStatus::Running {
        return;
    }
    let provider = RuleBasedAiProvider;
    let report = finish_turn(game, &provider);
    ui_state.message = format!("完成 {} 条结算记录", report.entries.len());
    ui_state.selected_city_id = first_player_city(game);
    ui_state.city_drawer_open = ui_state.selected_city_id.is_some();
}

fn clear_pending_commands(ui_state: &mut GameUiState) {
    let Some(game) = &mut ui_state.game else {
        ui_state.message = "尚未开始游戏".to_string();
        return;
    };
    game.pending_commands.clear();
    ui_state.message = "已清空玩家待命令".to_string();
}

fn selected_city_summary(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let summary = ui_state.game.as_ref().and_then(|game| {
        let city = game.cities.get(ui_state.selected_city_id.as_deref()?)?;
        let faction_name = game
            .factions
            .get(&city.faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| "未知".to_string());
        Some((
            city.id.clone(),
            city.name.clone(),
            faction_name,
            city.population,
            city.troops,
            city.gold,
            city.food,
        ))
    });

    let Some((city_id, city_name, faction_name, population, troops, gold, food)) = summary else {
        ui.label("未选择城池");
        return;
    };

    ui.heading(city_name);
    ui.label(format!("归属: {faction_name}"));
    ui.label(format!(
        "人口 {population} | 兵 {troops} | 金 {gold} | 粮 {food}"
    ));
    if ui.button("打开军令").clicked() {
        open_city(ui_state, city_id);
    }
}

fn map_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.heading("地图");
    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            zoom_map(ui_state, 1.0 / MAP_ZOOM_STEP, None, None);
        }
        ui.label(format!("{:.0}%", ui_state.map_zoom * 100.0));
        if ui.button("+").clicked() {
            zoom_map(ui_state, MAP_ZOOM_STEP, None, None);
        }
        if ui.button("重置").clicked() {
            reset_map_view(ui_state);
        }
    });
    ui.add_enabled(
        ui_state.map_boundaries.is_some(),
        egui::Checkbox::new(&mut ui_state.map_boundaries_enabled, "州郡边界"),
    );
    if ui_state.map_boundaries.is_none() {
        ui.colored_label(war_text_muted(), "边界资产未加载");
    }
}

fn city_list(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(game) = &ui_state.game else {
        return;
    };
    let mut rows: Vec<_> = game
        .cities
        .values()
        .map(|city| {
            let faction_name = game
                .factions
                .get(&city.faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| "未知".to_string());
            (city.id.clone(), city.name.clone(), faction_name)
        })
        .collect();
    rows.sort_by(|a, b| a.1.cmp(&b.1));

    ui.heading("城池");
    egui::ScrollArea::vertical()
        .id_salt("city_list")
        .max_height(460.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (city_id, city_name, faction_name) in rows {
                let selected = ui_state.selected_city_id.as_deref() == Some(city_id.as_str());
                let response =
                    ui.selectable_label(selected, format!("{} ({})", city_name, faction_name));
                if response.clicked() {
                    ui_state.selected_city_id = Some(city_id.clone());
                    ui_state.city_drawer_open = true;
                }
                if response.double_clicked() {
                    open_city(ui_state, city_id.clone());
                }
                response.context_menu(|ui| {
                    if ui.button("打开军令").clicked() {
                        open_city(ui_state, city_id.clone());
                        ui.close();
                    }
                });
            }
        });
}

fn open_city(ui_state: &mut GameUiState, city_id: CityId) {
    ui_state.selected_city_id = Some(city_id);
    ui_state.city_drawer_open = true;
}

fn finish_turn(game: &mut GameState, provider: &RuleBasedAiProvider) -> TurnReport {
    if is_history_scenario(&game.scenario_id) {
        if let Ok(catalog) = SqliteHistoricalCatalog::open_asset() {
            return finish_turn_with_ai_with_history(game, provider, &catalog);
        }
    }
    finish_turn_with_ai(game, provider)
}

fn is_history_scenario(scenario_id: &str) -> bool {
    matches!(scenario_id, "ad190" | "ad200" | "ad208" | "ad220")
}

fn save_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.horizontal(|ui| {
        ui.label("槽位");
        egui::ComboBox::from_id_salt("save_slot_combo")
            .selected_text(&ui_state.save_slot_id)
            .show_ui(ui, |ui| {
                for slot_id in ["slot1", "slot2", "slot3", "slot4", "slot5"] {
                    ui.selectable_value(&mut ui_state.save_slot_id, slot_id.to_string(), slot_id);
                }
            });
    });
    ui.horizontal(|ui| {
        ui.label("名称");
        ui.text_edit_singleline(&mut ui_state.save_display_name);
    });
    ui.horizontal(|ui| {
        if ui.button("保存").clicked() {
            if let Some(game) = &ui_state.game {
                match ui_state.save_manager.save_slot(
                    &ui_state.save_slot_id,
                    &ui_state.save_display_name,
                    game,
                ) {
                    Ok(meta) => {
                        refresh_saves(ui_state);
                        ui_state.message = format!("保存到 {}", meta.display_name);
                    }
                    Err(error) => ui_state.message = error.to_string(),
                }
            }
        }
        if ui.button("读取当前槽").clicked() {
            match ui_state.save_manager.load_slot(&ui_state.save_slot_id) {
                Ok(game) => {
                    enter_game(ui_state, game, "读取当前槽位".to_string());
                }
                Err(error) => ui_state.message = error.to_string(),
            }
        }
    });
    if !ui_state.message.is_empty() {
        ui.label(&ui_state.message);
    }
}

fn selected_city_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(game) = &mut ui_state.game else {
        return;
    };
    let Some(city_id) = ui_state.selected_city_id.clone() else {
        ui.label("请选择城池");
        return;
    };
    let Some(city) = game.cities.get(&city_id).cloned() else {
        ui.label("城池不存在");
        return;
    };

    let faction_name = game
        .factions
        .get(&city.faction_id)
        .map(|faction| faction.name.as_str())
        .unwrap_or("未知");
    ui.heading(&city.name);
    ui.label(format!("归属: {faction_name}"));
    ui.label(format!(
        "人口 {} | 金 {} | 粮 {} | 兵 {}",
        city.population, city.gold, city.food, city.troops
    ));
    ui.label(format!(
        "农业 {} | 商业 {} | 城防 {} | 训练 {} | 治安 {}",
        city.agriculture, city.commerce, city.defense, city.training, city.order
    ));
    if let Some(profile) = &city.profile {
        ui.label(format!(
            "{}{} | 规模 {} | 战略 {} | 可信度 {}",
            profile.province,
            profile.commandery,
            city_scale_label(&profile.scale),
            profile.strategic_rank,
            confidence_label(&profile.confidence)
        ));
        ui.label(format!(
            "人口区间 {}-{} | 农 {} 商 {} 防 {}",
            profile.population_min,
            profile.population_max,
            profile.agriculture_base,
            profile.commerce_base,
            profile.defense_base
        ));
        if !profile.notes.is_empty() {
            ui.label(&profile.notes);
        }
    }

    if let Some(governor_id) = &city.governor_id {
        if let Some(governor) = game.officers.get(governor_id) {
            ui.label(format!("太守: {}", governor.name));
        }
    }

    ui.separator();
    ui.horizontal(|ui| {
        ui.selectable_value(&mut ui_state.city_tab, CityTab::Construction, "建设");
        ui.selectable_value(&mut ui_state.city_tab, CityTab::Governance, "政务");
    });

    if city.faction_id != game.player_faction_id {
        ui.separator();
        officer_roster(ui, game, &city);
        ui.label("非己方城池，只能查看。");
        return;
    }

    let pending_city_ids = game.pending_city_ids();
    if pending_city_ids.contains(city.id.as_str()) {
        ui.separator();
        officer_roster(ui, game, &city);
        ui.label("本城本月已有待执行命令。");
        return;
    }

    let pending_officers = game.pending_officer_ids();
    let available_officers: Vec<_> = game
        .officers_in_city(&city.id)
        .into_iter()
        .filter(|officer| !pending_officers.contains(officer.id.as_str()))
        .cloned()
        .collect();
    if available_officers.is_empty() {
        ui.separator();
        ui.label("本城没有可行动武将。");
        return;
    }

    let selected_officer = ui_state
        .selected_officers
        .entry(city.id.clone())
        .or_insert_with(|| available_officers[0].id.clone());
    if !available_officers
        .iter()
        .any(|officer| officer.id == *selected_officer)
    {
        *selected_officer = available_officers[0].id.clone();
    }

    ui.separator();
    egui::ComboBox::from_id_salt(format!("officer_{}", city.id))
        .selected_text(
            game.officers
                .get(selected_officer)
                .map(|officer| officer.name.as_str())
                .unwrap_or("选择武将"),
        )
        .show_ui(ui, |ui| {
            for officer in &available_officers {
                ui.selectable_value(selected_officer, officer.id.clone(), &officer.name);
            }
        });

    match ui_state.city_tab {
        CityTab::Construction => {
            ui.heading("建设");
            develop_controls(ui, ui_state, &city);
            recruit_controls(ui, ui_state, &city);
            train_controls(ui, ui_state, &city);
        }
        CityTab::Governance => {
            officer_roster(ui, game, &city);
            ui.heading("政务");
            appoint_controls(ui, ui_state, &city, &available_officers);
            transfer_controls(ui, ui_state, &city);
            expedition_controls(ui, ui_state, &city);
            diplomacy_controls(ui, ui_state, &city);
        }
    }
}

fn officer_roster(ui: &mut egui::Ui, game: &GameState, city: &City) {
    ui.heading("武将");
    let officers = game.officers_in_city(&city.id);
    if officers.is_empty() {
        ui.label("无武将");
        return;
    }
    for officer in officers {
        officer_row(ui, officer);
    }
}

fn develop_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("开发", |ui| {
        egui::ComboBox::from_id_salt("develop_focus")
            .selected_text(development_focus_label(&ui_state.selected_focus))
            .show_ui(ui, |ui| {
                for focus in [
                    DevelopmentFocus::Agriculture,
                    DevelopmentFocus::Commerce,
                    DevelopmentFocus::Defense,
                    DevelopmentFocus::Order,
                ] {
                    ui.selectable_value(
                        &mut ui_state.selected_focus,
                        focus.clone(),
                        development_focus_label(&focus),
                    );
                }
            });
        if ui.button("提交开发").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Develop {
                    focus: ui_state.selected_focus.clone(),
                },
            );
        }
    });
}

fn recruit_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("征兵", |ui| {
        ui.add(egui::Slider::new(&mut ui_state.recruit_amount, 100..=5000).text("兵力"));
        if ui.button("提交征兵").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Recruit {
                    amount: ui_state.recruit_amount,
                },
            );
        }
    });
}

fn train_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("训练", |ui| {
        if ui.button("提交训练").clicked() {
            queue_selected_city_command(ui_state, city, CommandKind::Train);
        }
    });
}

fn appoint_controls(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    city: &City,
    available_officers: &[Officer],
) {
    ui.collapsing("任命太守", |ui| {
        let target = ui_state
            .selected_officers
            .get(&city.id)
            .cloned()
            .unwrap_or_else(|| available_officers[0].id.clone());
        if ui.button("任命当前武将为太守").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::AppointGovernor {
                    target_officer_id: target,
                },
            );
        }
    });
}

fn transfer_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("调动", |ui| {
        let Some(game) = &ui_state.game else {
            return;
        };
        let targets: Vec<_> = game
            .cities
            .values()
            .filter(|target| {
                target.faction_id == game.player_faction_id
                    && game.are_adjacent(&city.id, &target.id)
            })
            .cloned()
            .collect();
        if targets.is_empty() {
            ui.label("无邻接己方城池");
            return;
        }
        let selected = ui_state
            .selected_transfer_target
            .get_or_insert_with(|| targets[0].id.clone());
        if !targets.iter().any(|target| target.id == *selected) {
            *selected = targets[0].id.clone();
        }
        egui::ComboBox::from_id_salt("transfer_target")
            .selected_text(
                game.cities
                    .get(selected)
                    .map(|city| city.name.as_str())
                    .unwrap_or("目标"),
            )
            .show_ui(ui, |ui| {
                for target in &targets {
                    ui.selectable_value(selected, target.id.clone(), &target.name);
                }
            });
        ui.add(egui::Slider::new(&mut ui_state.transfer_troops, 0..=city.troops).text("兵力"));
        let selected_target_id = selected.clone();
        if ui.button("提交调动").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Transfer {
                    target_city_id: selected_target_id,
                    troops: ui_state.transfer_troops,
                    officer_ids: Vec::new(),
                },
            );
        }
    });
}

fn expedition_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("出征", |ui| {
        let Some(game) = &ui_state.game else {
            return;
        };
        let targets: Vec<_> = game
            .cities
            .values()
            .filter(|target| {
                target.faction_id != game.player_faction_id
                    && game.are_adjacent(&city.id, &target.id)
            })
            .cloned()
            .collect();
        if targets.is_empty() {
            ui.label("无邻接敌方城池");
            return;
        }
        let selected = ui_state
            .selected_expedition_target
            .get_or_insert_with(|| targets[0].id.clone());
        if !targets.iter().any(|target| target.id == *selected) {
            *selected = targets[0].id.clone();
        }
        egui::ComboBox::from_id_salt("expedition_target")
            .selected_text(
                game.cities
                    .get(selected)
                    .map(|city| city.name.as_str())
                    .unwrap_or("目标"),
            )
            .show_ui(ui, |ui| {
                for target in &targets {
                    ui.selectable_value(selected, target.id.clone(), &target.name);
                }
            });
        ui.add(
            egui::Slider::new(&mut ui_state.expedition_troops, 100..=city.troops.max(100))
                .text("兵力"),
        );
        let selected_target_id = selected.clone();
        if ui.button("提交出征").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Expedition {
                    target_city_id: selected_target_id,
                    troops: ui_state.expedition_troops.min(city.troops),
                },
            );
        }
    });
}

fn diplomacy_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, city: &City) {
    ui.collapsing("外交", |ui| {
        let Some(game) = &ui_state.game else {
            return;
        };
        let targets: Vec<_> = game
            .factions
            .values()
            .filter(|faction| {
                faction.id != game.player_faction_id && game.faction_alive(&faction.id)
            })
            .cloned()
            .collect();
        if targets.is_empty() {
            ui.label("无外交目标");
            return;
        }
        let selected = ui_state
            .selected_diplomacy_target
            .get_or_insert_with(|| targets[0].id.clone());
        if !targets.iter().any(|target| target.id == *selected) {
            *selected = targets[0].id.clone();
        }
        egui::ComboBox::from_id_salt("diplomacy_target")
            .selected_text(
                game.factions
                    .get(selected)
                    .map(|faction| faction.name.as_str())
                    .unwrap_or("目标"),
            )
            .show_ui(ui, |ui| {
                for target in &targets {
                    ui.selectable_value(selected, target.id.clone(), &target.name);
                }
            });
        egui::ComboBox::from_id_salt("diplomacy_proposal")
            .selected_text(diplomacy_label(&ui_state.selected_diplomacy_proposal))
            .show_ui(ui, |ui| {
                for proposal in [
                    DiplomacyProposal::ImproveRelations,
                    DiplomacyProposal::Truce,
                    DiplomacyProposal::DeclareWar,
                ] {
                    ui.selectable_value(
                        &mut ui_state.selected_diplomacy_proposal,
                        proposal.clone(),
                        diplomacy_label(&proposal),
                    );
                }
            });
        let selected_target_id = selected.clone();
        if ui.button("提交外交").clicked() {
            queue_selected_city_command(
                ui_state,
                city,
                CommandKind::Diplomacy {
                    target_faction_id: selected_target_id,
                    proposal: ui_state.selected_diplomacy_proposal.clone(),
                },
            );
        }
    });
}

fn queue_selected_city_command(ui_state: &mut GameUiState, city: &City, kind: CommandKind) {
    let Some(game) = &mut ui_state.game else {
        return;
    };
    let Some(officer_id) = ui_state.selected_officers.get(&city.id).cloned() else {
        ui_state.message = "请选择执行武将".to_string();
        return;
    };
    let command = GameCommand {
        issuer_faction_id: game.player_faction_id.clone(),
        city_id: city.id.clone(),
        officer_id: Some(officer_id),
        kind,
    };
    match queue_player_command(game, command) {
        Ok(()) => ui_state.message = format!("已提交 {} 的命令", city.name),
        Err(error) => ui_state.message = error.to_string(),
    }
}

fn map_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    if ui_state.game.is_none() {
        ui.centered_and_justified(|ui| {
            ui.label("尚未开始游戏");
        });
        return;
    }
    let desired = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    draw_strategy_map_background(&painter, rect);

    let scroll_delta = ui.input(|input| input.raw_scroll_delta.y);
    if response.hovered() && scroll_delta.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + scroll_delta * 0.0015).clamp(0.8, 1.25);
        zoom_map(
            ui_state,
            zoom_factor,
            response.hover_pos(),
            Some(rect.center()),
        );
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        ui_state.map_pan += response.drag_delta();
        clamp_map_pan(ui_state, rect);
    }

    let Some(game) = &ui_state.game else {
        return;
    };
    let Some(bounds) = map_bounds(game) else {
        return;
    };

    if ui_state.map_boundaries_enabled {
        if let Some(catalog) = &ui_state.map_boundaries {
            draw_map_boundaries(&painter, game, catalog, bounds, rect, ui_state);
        }
    }

    for road in &game.roads {
        let Some(from) = game.cities.get(&road.from) else {
            continue;
        };
        let Some(to) = game.cities.get(&road.to) else {
            continue;
        };
        let a = map_to_screen(from.position, bounds, rect, ui_state);
        let b = map_to_screen(to.position, bounds, rect, ui_state);
        painter.line_segment(
            [a, b],
            egui::Stroke::new(7.0, egui::Color32::from_rgba_unmultiplied(10, 12, 10, 110)),
        );
        painter.line_segment(
            [a, b],
            egui::Stroke::new(
                3.0,
                egui::Color32::from_rgba_unmultiplied(160, 128, 77, 185),
            ),
        );
    }

    for city in game.cities.values() {
        let pos = map_to_screen(city.position, bounds, rect, ui_state);
        let faction = &game.factions[&city.faction_id];
        let color = faction_color(faction);
        let selected = ui_state.selected_city_id.as_deref() == Some(city.id.as_str());
        let player_owned = city.faction_id == game.player_faction_id;
        draw_city_marker(&painter, pos, city, color, selected, player_owned, ui_state);
    }

    let picked_city = response
        .interact_pointer_pos()
        .and_then(|pointer_pos| city_at_position(game, bounds, rect, pointer_pos, ui_state));

    if response.clicked() || response.secondary_clicked() {
        if let Some(city_id) = picked_city.clone() {
            ui_state.selected_city_id = Some(city_id);
            ui_state.city_drawer_open = true;
        }
    }
    if response.double_clicked() {
        if let Some(city_id) = picked_city.clone() {
            open_city(ui_state, city_id);
        }
    }

    let context_city_id = ui_state.selected_city_id.clone();
    response.context_menu(|ui| {
        if let Some(city_id) = context_city_id.clone() {
            if ui.button("打开军令").clicked() {
                open_city(ui_state, city_id);
                ui.close();
            }
        }
    });
}

fn report_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState, screen: egui::Rect) {
    let Some(game) = &ui_state.game else {
        return;
    };
    let report_count = game.reports.len();
    let visible_start = report_count.saturating_sub(12);
    let report_height = (screen.height() * 0.32).clamp(220.0, 340.0);
    ui.set_min_height(report_height);
    egui::ScrollArea::vertical()
        .id_salt("turn_report_scroll")
        .max_height(report_height)
        .min_scrolled_height(report_height)
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if !ui_state.message.is_empty() {
                ui.colored_label(war_gold(), &ui_state.message);
                ui.separator();
            }
            if game.reports.is_empty() {
                ui.label("暂无报告");
            }
            for report in game.reports.iter().skip(visible_start) {
                ui.label(format!(
                    "{}年{}月 第{}回合",
                    report.year, report.month, report.turn
                ));
                for entry in &report.entries {
                    match entry.severity {
                        ReportSeverity::Info => {
                            ui.label(format!("· {}", entry.message));
                        }
                        ReportSeverity::Warning => {
                            ui.colored_label(egui::Color32::YELLOW, format!("! {}", entry.message));
                        }
                    }
                }
                ui.separator();
            }
        });
}

fn refresh_saves(ui_state: &mut GameUiState) {
    ui_state.save_slots = ui_state.save_manager.list_slots().unwrap_or_default();
}

fn first_player_city(game: &GameState) -> Option<CityId> {
    game.cities
        .values()
        .find(|city| city.faction_id == game.player_faction_id)
        .map(|city| city.id.clone())
}

fn configure_egui_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(war_text());
    style.visuals.weak_text_color = Some(war_text_muted());
    style.visuals.window_fill = war_panel_fill();
    style.visuals.panel_fill = egui::Color32::TRANSPARENT;
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(15, 14, 12);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(43, 38, 29);
    style.visuals.hyperlink_color = war_gold();
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(122, 59, 39);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, war_gold());
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 36, 27);
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(57, 45, 32);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, war_border());
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(83, 61, 37);
    style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(94, 69, 42);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, war_gold());
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(110, 53, 37);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, war_gold());
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

fn war_panel_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 12))
        .fill(war_panel_fill())
        .stroke(egui::Stroke::new(1.0, war_border()))
        .corner_radius(6)
}

fn war_bar_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 9))
        .fill(egui::Color32::from_rgba_unmultiplied(18, 17, 14, 238))
        .stroke(egui::Stroke::new(1.0, war_border()))
        .corner_radius(4)
}

fn war_sub_panel_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .fill(egui::Color32::from_rgba_unmultiplied(34, 29, 22, 210))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(138, 101, 58, 120),
        ))
        .corner_radius(4)
}

fn war_panel_fill() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(24, 21, 16, 232)
}

fn war_text() -> egui::Color32 {
    egui::Color32::from_rgb(226, 213, 184)
}

fn war_text_muted() -> egui::Color32 {
    egui::Color32::from_rgb(168, 154, 124)
}

fn war_gold() -> egui::Color32 {
    egui::Color32::from_rgb(215, 162, 72)
}

fn war_border() -> egui::Color32 {
    egui::Color32::from_rgb(118, 85, 48)
}

fn draw_menu_background(painter: &egui::Painter, rect: egui::Rect) {
    draw_strategy_map_background(painter, rect);
    painter.rect_filled(
        rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(8, 8, 7, 86),
    );
    let top = egui::Rect::from_min_max(rect.min, egui::pos2(rect.right(), rect.top() + 150.0));
    painter.rect_filled(
        top,
        0.0,
        egui::Color32::from_rgba_unmultiplied(15, 13, 10, 120),
    );
}

fn draw_strategy_map_background(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(35, 39, 29));

    let mut x = rect.left() - rect.left().rem_euclid(72.0);
    while x < rect.right() {
        painter.line_segment(
            [
                egui::pos2(x, rect.top()),
                egui::pos2(x + 44.0, rect.bottom()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 104, 65, 24)),
        );
        x += 72.0;
    }

    let mut y = rect.top() - rect.top().rem_euclid(58.0);
    while y < rect.bottom() {
        painter.line_segment(
            [
                egui::pos2(rect.left(), y),
                egui::pos2(rect.right(), y + 18.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(94, 120, 93, 18)),
        );
        y += 58.0;
    }

    let river_y = rect.center().y + rect.height() * 0.12;
    let river = [
        egui::pos2(rect.left() - 80.0, river_y - 28.0),
        egui::pos2(rect.left() + rect.width() * 0.22, river_y + 22.0),
        egui::pos2(rect.left() + rect.width() * 0.44, river_y - 10.0),
        egui::pos2(rect.left() + rect.width() * 0.68, river_y + 30.0),
        egui::pos2(rect.right() + 80.0, river_y - 18.0),
    ];
    for segment in river.windows(2) {
        painter.line_segment(
            [segment[0], segment[1]],
            egui::Stroke::new(15.0, egui::Color32::from_rgba_unmultiplied(40, 83, 84, 48)),
        );
        painter.line_segment(
            [segment[0], segment[1]],
            egui::Stroke::new(
                3.0,
                egui::Color32::from_rgba_unmultiplied(111, 143, 126, 74),
            ),
        );
    }

    painter.rect_stroke(
        rect.shrink(8.0),
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(180, 132, 74, 76)),
        egui::StrokeKind::Inside,
    );
}

fn draw_map_boundaries(
    painter: &egui::Painter,
    game: &GameState,
    catalog: &MapBoundaryCatalog,
    bounds: MapBounds,
    rect: egui::Rect,
    ui_state: &GameUiState,
) {
    let cells = catalog.territory_cells_for_year(game.year);
    if cells.is_empty() {
        return;
    }

    for cell in &cells {
        let fill = territory_cell_fill_color(cell, game, ui_state);
        let points = territory_cell_screen_points(cell, bounds, rect, ui_state);
        paint_boundary_polygon(painter, points, fill, egui::Stroke::NONE);
    }

    for cell in &cells {
        let selected = selected_city_in_cell(cell, game, ui_state);
        let points = territory_cell_screen_points(cell, bounds, rect, ui_state);
        let (stroke, dash, gap) = if selected {
            (
                egui::Stroke::new(
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(174, 221, 210, 230),
                ),
                8.0,
                4.0,
            )
        } else {
            (
                egui::Stroke::new(
                    0.9,
                    egui::Color32::from_rgba_unmultiplied(112, 136, 124, 98),
                ),
                6.0,
                8.0,
            )
        };
        draw_dashed_closed_polyline(painter, &points, stroke, dash, gap);
    }

    for (start, end) in province_border_segments(&cells) {
        let start = map_to_screen(start, bounds, rect, ui_state);
        let end = map_to_screen(end, bounds, rect, ui_state);
        draw_dashed_segment(
            painter,
            start,
            end,
            egui::Stroke::new(3.5, egui::Color32::from_rgba_unmultiplied(6, 14, 15, 150)),
            15.0,
            7.0,
        );
        draw_dashed_segment(
            painter,
            start,
            end,
            egui::Stroke::new(
                1.8,
                egui::Color32::from_rgba_unmultiplied(116, 171, 170, 190),
            ),
            15.0,
            7.0,
        );
    }
}

fn territory_cell_screen_points(
    cell: &TerritoryCell,
    bounds: MapBounds,
    rect: egui::Rect,
    ui_state: &GameUiState,
) -> Vec<egui::Pos2> {
    cell.points
        .iter()
        .map(|point| map_to_screen(*point, bounds, rect, ui_state))
        .collect()
}

fn paint_boundary_polygon(
    painter: &egui::Painter,
    points: Vec<egui::Pos2>,
    fill: egui::Color32,
    stroke: egui::Stroke,
) {
    if points.len() < 3 {
        return;
    }
    painter.add(egui::Shape::Path(egui::epaint::PathShape {
        points,
        closed: true,
        fill,
        stroke: stroke.into(),
    }));
}

fn draw_dashed_closed_polyline(
    painter: &egui::Painter,
    points: &[egui::Pos2],
    stroke: egui::Stroke,
    dash: f32,
    gap: f32,
) {
    if points.len() < 2 {
        return;
    }

    let cycle = dash + gap;
    if cycle <= f32::EPSILON {
        return;
    }

    for index in 0..points.len() {
        draw_dashed_segment(
            painter,
            points[index],
            points[(index + 1) % points.len()],
            stroke,
            dash,
            gap,
        );
    }
}

fn draw_dashed_segment(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
    dash: f32,
    gap: f32,
) {
    let cycle = dash + gap;
    if cycle <= f32::EPSILON {
        return;
    }

    let delta = end - start;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }

    let direction = delta / length;
    let mut offset = 0.0;
    while offset < length {
        let dash_end = (offset + dash).min(length);
        painter.line_segment(
            [start + direction * offset, start + direction * dash_end],
            stroke,
        );
        offset += cycle;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TerritoryEdgeKey {
    start_x: i64,
    start_y: i64,
    end_x: i64,
    end_y: i64,
}

struct TerritoryEdge {
    start: MapPosition,
    end: MapPosition,
    parent_id: Option<String>,
}

fn province_border_segments(cells: &[TerritoryCell]) -> Vec<(MapPosition, MapPosition)> {
    let mut edges: BTreeMap<TerritoryEdgeKey, Vec<TerritoryEdge>> = BTreeMap::new();
    for cell in cells {
        for index in 0..cell.points.len() {
            let start = cell.points[index];
            let end = cell.points[(index + 1) % cell.points.len()];
            edges
                .entry(territory_edge_key(start, end))
                .or_default()
                .push(TerritoryEdge {
                    start,
                    end,
                    parent_id: cell.parent_id.clone(),
                });
        }
    }

    edges
        .into_values()
        .filter_map(|edge_group| {
            let first = edge_group.first()?;
            let crosses_parent = edge_group
                .iter()
                .any(|edge| edge.parent_id != first.parent_id);
            (edge_group.len() == 1 || crosses_parent).then_some((first.start, first.end))
        })
        .collect()
}

fn territory_edge_key(start: MapPosition, end: MapPosition) -> TerritoryEdgeKey {
    let start = quantized_map_position(start);
    let end = quantized_map_position(end);
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    TerritoryEdgeKey {
        start_x: start.0,
        start_y: start.1,
        end_x: end.0,
        end_y: end.1,
    }
}

fn quantized_map_position(position: MapPosition) -> (i64, i64) {
    (
        (position.x * 1_000.0).round() as i64,
        (position.y * 1_000.0).round() as i64,
    )
}

fn territory_cell_fill_color(
    cell: &TerritoryCell,
    game: &GameState,
    ui_state: &GameUiState,
) -> egui::Color32 {
    let selected = selected_city_in_cell(cell, game, ui_state);
    let alpha = if selected { 44 } else { 18 };

    dominant_cell_faction(cell, game)
        .map(faction_color)
        .map(|color| egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha))
        .unwrap_or_else(|| egui::Color32::from_rgba_unmultiplied(92, 96, 67, alpha))
}

fn dominant_cell_faction<'a>(cell: &TerritoryCell, game: &'a GameState) -> Option<&'a Faction> {
    let mut counts: BTreeMap<&str, (usize, u32)> = BTreeMap::new();
    for city in game
        .cities
        .values()
        .filter(|city| city_matches_cell(cell, city))
    {
        let entry = counts.entry(city.faction_id.as_str()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 = entry.1.saturating_add(city.troops);
    }
    let faction_id = counts
        .iter()
        .max_by_key(|(_, (city_count, troops))| (*city_count, *troops))
        .map(|(faction_id, _)| *faction_id)?;
    game.factions.get(faction_id)
}

fn selected_city_in_cell(cell: &TerritoryCell, game: &GameState, ui_state: &GameUiState) -> bool {
    let Some(selected_city_id) = ui_state.selected_city_id.as_deref() else {
        return false;
    };
    game.cities
        .get(selected_city_id)
        .is_some_and(|city| city_matches_cell(cell, city))
}

fn city_matches_cell(cell: &TerritoryCell, city: &City) -> bool {
    if cell.city_ids.iter().any(|city_id| city_id == &city.id) {
        return true;
    }

    let Some(profile) = &city.profile else {
        return false;
    };
    profile.commandery == cell.name
}

fn draw_city_marker(
    painter: &egui::Painter,
    pos: egui::Pos2,
    city: &City,
    color: egui::Color32,
    selected: bool,
    player_owned: bool,
    ui_state: &GameUiState,
) {
    let scale = ui_state.map_zoom.sqrt().clamp(0.85, 1.35);
    let pole_top = pos + egui::vec2(-13.0 * scale, -31.0 * scale);
    let pole_bottom = pos + egui::vec2(-13.0 * scale, 18.0 * scale);
    let flag_fill = if player_owned {
        color
    } else {
        color.gamma_multiply(0.82)
    };
    let shadow = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120);

    painter.line_segment(
        [
            pole_top + egui::vec2(2.0, 3.0),
            pole_bottom + egui::vec2(2.0, 3.0),
        ],
        egui::Stroke::new(5.0, shadow),
    );
    painter.line_segment(
        [pole_top, pole_bottom],
        egui::Stroke::new(3.0, egui::Color32::from_rgb(34, 26, 18)),
    );

    let banner = egui::Rect::from_min_size(
        pole_top + egui::vec2(3.0 * scale, 1.0 * scale),
        egui::vec2(43.0 * scale, 20.0 * scale),
    );
    painter.rect(
        banner.translate(egui::vec2(2.0, 2.0)),
        2.0,
        shadow,
        egui::Stroke::NONE,
        egui::StrokeKind::Outside,
    );
    painter.rect(
        banner,
        2.0,
        flag_fill,
        egui::Stroke::new(
            if selected { 2.5 } else { 1.5 },
            if selected {
                war_gold()
            } else {
                egui::Color32::from_rgb(35, 28, 20)
            },
        ),
        egui::StrokeKind::Outside,
    );

    let base = egui::Rect::from_center_size(
        pos + egui::vec2(0.0, 12.0 * scale),
        egui::vec2(42.0 * scale, 17.0 * scale),
    );
    painter.rect(
        base,
        4.0,
        egui::Color32::from_rgba_unmultiplied(20, 18, 14, 222),
        egui::Stroke::new(1.0, flag_fill),
        egui::StrokeKind::Outside,
    );
    painter.text(
        base.center(),
        egui::Align2::CENTER_CENTER,
        compact_troops(city.troops),
        egui::FontId::proportional(12.0 * scale),
        war_text(),
    );

    let label_center = pos + egui::vec2(0.0, 42.0 * scale);
    let label_width = (city.name.chars().count() as f32 * 17.0 + 28.0).max(68.0);
    let label_rect =
        egui::Rect::from_center_size(label_center, egui::vec2(label_width, 25.0 * scale));
    painter.rect(
        label_rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(17, 16, 13, if selected { 238 } else { 204 }),
        egui::Stroke::new(
            if selected { 1.5 } else { 1.0 },
            if selected { war_gold() } else { war_border() },
        ),
        egui::StrokeKind::Outside,
    );
    painter.text(
        label_center,
        egui::Align2::CENTER_CENTER,
        &city.name,
        egui::FontId::proportional(15.0 * scale),
        if selected { war_gold() } else { war_text() },
    );
}

fn compact_troops(troops: u32) -> String {
    if troops >= 10_000 {
        format!("{:.1}万", troops as f32 / 10_000.0)
    } else {
        troops.to_string()
    }
}

#[derive(Clone, Copy)]
struct MapBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

fn map_bounds(game: &GameState) -> Option<MapBounds> {
    let mut cities = game.cities.values();
    let first = cities.next()?;
    let mut bounds = MapBounds {
        min_x: first.position.x,
        max_x: first.position.x,
        min_y: first.position.y,
        max_y: first.position.y,
    };
    for city in cities {
        bounds.min_x = bounds.min_x.min(city.position.x);
        bounds.max_x = bounds.max_x.max(city.position.x);
        bounds.min_y = bounds.min_y.min(city.position.y);
        bounds.max_y = bounds.max_y.max(city.position.y);
    }
    Some(bounds)
}

fn map_to_screen(
    position: MapPosition,
    bounds: MapBounds,
    rect: egui::Rect,
    ui_state: &GameUiState,
) -> egui::Pos2 {
    let padding = (rect.width().min(rect.height()) * 0.09).clamp(72.0, 118.0);
    let width = (bounds.max_x - bounds.min_x).max(1.0);
    let height = (bounds.max_y - bounds.min_y).max(1.0);
    let x = (position.x - bounds.min_x) / width;
    let y = (position.y - bounds.min_y) / height;
    let base = egui::pos2(
        rect.left() + padding + x * (rect.width() - padding * 2.0).max(1.0),
        rect.bottom() - padding - y * (rect.height() - padding * 2.0).max(1.0),
    );
    rect.center() + (base - rect.center()) * ui_state.map_zoom + ui_state.map_pan
}

fn city_at_position(
    game: &GameState,
    bounds: MapBounds,
    rect: egui::Rect,
    pointer_pos: egui::Pos2,
    ui_state: &GameUiState,
) -> Option<CityId> {
    game.cities
        .values()
        .filter_map(|city| {
            let pos = map_to_screen(city.position, bounds, rect, ui_state);
            let distance = pos.distance(pointer_pos);
            (distance <= city_pick_radius(ui_state)).then_some((distance, city.id.clone()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, city_id)| city_id)
}

fn city_pick_radius(ui_state: &GameUiState) -> f32 {
    (24.0 * ui_state.map_zoom.sqrt()).clamp(22.0, 38.0)
}

fn zoom_map(
    ui_state: &mut GameUiState,
    factor: f32,
    anchor: Option<egui::Pos2>,
    center: Option<egui::Pos2>,
) {
    let old_zoom = ui_state.map_zoom;
    let new_zoom = (old_zoom * factor).clamp(MAP_MIN_ZOOM, MAP_MAX_ZOOM);
    if (new_zoom - old_zoom).abs() <= f32::EPSILON {
        return;
    }

    ui_state.map_zoom = new_zoom;
    if let (Some(anchor), Some(center)) = (anchor, center) {
        let content_from_center = (anchor - center - ui_state.map_pan) / old_zoom;
        ui_state.map_pan = anchor - center - content_from_center * new_zoom;
    } else if new_zoom <= 1.0 {
        ui_state.map_pan = egui::Vec2::ZERO;
    }
}

fn clamp_map_pan(ui_state: &mut GameUiState, rect: egui::Rect) {
    if ui_state.map_zoom <= 1.0 {
        ui_state.map_pan = egui::Vec2::ZERO;
        return;
    }
    let max_x = rect.width() * ui_state.map_zoom;
    let max_y = rect.height() * ui_state.map_zoom;
    ui_state.map_pan.x = ui_state.map_pan.x.clamp(-max_x, max_x);
    ui_state.map_pan.y = ui_state.map_pan.y.clamp(-max_y, max_y);
}

fn reset_map_view(ui_state: &mut GameUiState) {
    ui_state.map_zoom = 1.0;
    ui_state.map_pan = egui::Vec2::ZERO;
}

fn faction_color(faction: &Faction) -> egui::Color32 {
    egui::Color32::from_rgb(
        (faction.color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (faction.color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (faction.color[2].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn development_focus_label(focus: &DevelopmentFocus) -> &'static str {
    match focus {
        DevelopmentFocus::Agriculture => "农业",
        DevelopmentFocus::Commerce => "商业",
        DevelopmentFocus::Defense => "城防",
        DevelopmentFocus::Order => "治安",
    }
}

fn diplomacy_label(proposal: &DiplomacyProposal) -> &'static str {
    match proposal {
        DiplomacyProposal::ImproveRelations => "改善关系",
        DiplomacyProposal::Truce => "停战",
        DiplomacyProposal::DeclareWar => "宣战",
    }
}

fn officer_row(ui: &mut egui::Ui, officer: &Officer) {
    let title = format!(
        "{} 统{} 武{} 智{} 政{} 魅{}",
        officer.name,
        officer.stats.leadership,
        officer.stats.strength,
        officer.stats.intelligence,
        officer.stats.politics,
        officer.stats.charm
    );
    ui.collapsing(title, |ui| {
        ui.label(format!("忠诚 {}", officer.loyalty));
        if let Some(profile) = &officer.profile {
            let courtesy = profile.courtesy_name.as_deref().unwrap_or("无");
            let native_place = profile.native_place.as_deref().unwrap_or("未详");
            let birth = profile
                .birth_year
                .map(|year| year.to_string())
                .unwrap_or_else(|| "未详".to_string());
            let death = profile
                .death_year
                .map(|year| year.to_string())
                .unwrap_or_else(|| "未详".to_string());
            ui.label(format!("字 {courtesy} | 籍贯 {native_place}"));
            ui.label(format!(
                "生卒 {birth}-{death} | 可信度 {}",
                confidence_label(&profile.confidence)
            ));
            if !profile.tags.is_empty() {
                ui.label(format!("标签 {}", profile.tags.join(", ")));
            }
            if !profile.notes.is_empty() {
                ui.label(&profile.notes);
            }
        }
    });
}

fn city_scale_label(scale: &CityScale) -> &'static str {
    match scale {
        CityScale::County => "县城",
        CityScale::Commandery => "郡治",
        CityScale::RegionalCapital => "州郡重镇",
        CityScale::ImperialCapital => "都城",
    }
}

fn confidence_label(confidence: &SourceConfidence) -> &'static str {
    match confidence {
        SourceConfidence::High => "高",
        SourceConfidence::Medium => "中",
        SourceConfidence::Low => "低",
    }
}

fn configure_egui_fonts(ctx: &egui::Context, ui_state: &mut GameUiState) {
    if ui_state.egui_font_configured {
        return;
    }
    ui_state.egui_font_configured = true;

    let Some(bytes) = load_cjk_font_bytes() else {
        return;
    };
    ctx.add_font(egui::epaint::text::FontInsert::new(
        "shogun_cjk",
        egui::FontData::from_owned(bytes),
        vec![
            egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
            egui::epaint::text::InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    ));
}

fn load_cjk_font_bytes() -> Option<Vec<u8>> {
    let candidates = vec![
        PathBuf::from("assets/fonts/LXGWWenKai-Regular.ttf"),
        PathBuf::from("assets/fonts/ZCOOLXiaoWei-Regular.ttf"),
        PathBuf::from("assets/fonts/NotoSansCJKsc-Regular.otf"),
        PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
        PathBuf::from("/System/Library/Fonts/STHeiti Light.ttc"),
        PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("C:/Windows/Fonts/msyh.ttc"),
    ];

    candidates.into_iter().find_map(|path| fs::read(path).ok())
}
