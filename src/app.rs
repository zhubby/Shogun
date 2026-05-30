use crate::game::{Command as GameCommand, *};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const MAP_MIN_ZOOM: f32 = 0.65;
const MAP_MAX_ZOOM: f32 = 5.0;
const MAP_ZOOM_STEP: f32 = 1.2;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shogun - 三国志风格策略原型".to_string(),
                resolution: WindowResolution::new(1280, 820),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .init_resource::<GameUiState>()
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
    in_game_view: InGameView,
    city_tab: CityTab,
    map_zoom: f32,
    map_pan: egui::Vec2,
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
    message: String,
    egui_font_configured: bool,
}

impl Default for GameUiState {
    fn default() -> Self {
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
        Self {
            json_scenario,
            history_scenarios: history_menu.scenarios,
            selected_scenario_id: history_menu.selected_scenario_id,
            history_factions: history_menu.factions,
            screen: Screen::MainMenu,
            in_game_view: InGameView::Map,
            city_tab: CityTab::Construction,
            map_zoom: 1.0,
            map_pan: egui::Vec2::ZERO,
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
            message: history_menu.message,
            egui_font_configured: false,
        }
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
enum InGameView {
    Map,
    City,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CityTab {
    Construction,
    Governance,
}

fn game_ui_system(mut contexts: EguiContexts, mut ui_state: ResMut<GameUiState>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    configure_egui_fonts(ctx, &mut ui_state);

    egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
        top_bar(ui, &mut ui_state);
    });

    match ui_state.screen {
        Screen::MainMenu => main_menu(ctx, &mut ui_state),
        Screen::InGame => in_game(ctx, &mut ui_state),
    }
}

fn top_bar(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    ui.horizontal(|ui| {
        ui.heading("Shogun");
        ui.separator();
        if let Some(game) = &ui_state.game {
            ui.label(format!(
                "{} {}年{}月 第{}回合",
                game.scenario_name, game.year, game.month, game.turn
            ));
            if let Some(faction) = game.factions.get(&game.player_faction_id) {
                ui.label(format!("玩家: {}", faction.name));
            }
            match &game.status {
                GameStatus::Running => {}
                GameStatus::Victory { reason } => {
                    ui.colored_label(egui::Color32::LIGHT_GREEN, format!("胜利: {reason}"));
                }
                GameStatus::Defeat { reason } => {
                    ui.colored_label(egui::Color32::LIGHT_RED, format!("失败: {reason}"));
                }
            }
        } else {
            ui.label("桌面策略游戏原型");
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("主菜单").clicked() {
                ui_state.screen = Screen::MainMenu;
            }
        });
    });
}

fn main_menu(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.heading("新开局");
                ui.add_space(8.0);
                if !ui_state.history_scenarios.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("剧本");
                        if ui.button("刷新资料库").clicked() {
                            refresh_history_menu(ui_state);
                        }
                    });
                    let mut scenario_changed = false;
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
                    if scenario_changed {
                        refresh_history_factions(ui_state);
                    }

                    ui.add_space(8.0);
                    ui.label("势力");
                    for faction in ui_state
                        .history_factions
                        .iter()
                        .filter(|faction| faction.selectable)
                        .cloned()
                        .collect::<Vec<_>>()
                    {
                        ui.radio_value(&mut ui_state.selected_faction_id, faction.id, faction.name);
                    }
                    ui.add_space(8.0);
                    if ui.button("开始游戏").clicked() {
                        start_history_game(ui_state);
                    }
                } else {
                    ui.label("选择势力");
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
                    ui.add_space(8.0);
                    if ui.button("开始兼容小剧本").clicked() {
                        start_json_game(ui_state);
                    }
                }
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.heading("读取存档");
                ui.label(format!(
                    "目录: {}",
                    ui_state.save_manager.base_dir().display()
                ));
                if ui.button("刷新存档列表").clicked() {
                    refresh_saves(ui_state);
                }
                ui.add_space(8.0);
                let slots = ui_state.save_slots.clone();
                for slot in slots {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{} - {}年{}月 第{}回合",
                                slot.display_name, slot.year, slot.month, slot.turn
                            ));
                            if ui.button("读取").clicked() {
                                match ui_state.save_manager.load_slot(&slot.slot_id) {
                                    Ok(game) => {
                                        enter_game(
                                            ui_state,
                                            game,
                                            format!("读取存档 {}", slot.display_name),
                                        );
                                    }
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
                }
            });
        });

        if !ui_state.message.is_empty() {
            ui.separator();
            ui.label(&ui_state.message);
        }
    });
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
    ui_state.in_game_view = InGameView::Map;
    ui_state.city_tab = CityTab::Construction;
    reset_map_view(ui_state);
    ui_state.game = Some(game);
    ui_state.screen = Screen::InGame;
    ui_state.message = message;
}

fn in_game(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::SidePanel::left("city_panel")
        .resizable(true)
        .default_width(360.0)
        .show(ctx, |ui| match ui_state.in_game_view {
            InGameView::Map => map_side_panel(ui, ui_state),
            InGameView::City => city_navigation_panel(ui, ui_state),
        });

    egui::TopBottomPanel::bottom("report_panel")
        .resizable(true)
        .default_height(160.0)
        .show(ctx, |ui| report_panel(ui, ui_state));

    egui::CentralPanel::default().show(ctx, |ui| match ui_state.in_game_view {
        InGameView::Map => map_panel(ui, ui_state),
        InGameView::City => selected_city_panel(ui, ui_state),
    });
}

fn map_side_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    save_controls(ui, ui_state);
    ui.separator();
    turn_controls(ui, ui_state);
    ui.separator();
    selected_city_summary(ui, ui_state);
    ui.separator();
    map_controls(ui, ui_state);
    ui.separator();
    city_list(ui, ui_state);
}

fn city_navigation_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    save_controls(ui, ui_state);
    ui.separator();
    turn_controls(ui, ui_state);
    ui.separator();
    if ui.button("返回地图").clicked() {
        ui_state.in_game_view = InGameView::Map;
    }
    ui.separator();
    city_list(ui, ui_state);
}

fn turn_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(game) = &mut ui_state.game else {
        ui.label("尚未开始游戏");
        return;
    };
    ui.horizontal(|ui| {
        if ui.button("结束本月").clicked() && game.status == GameStatus::Running {
            let provider = RuleBasedAiProvider;
            let report = finish_turn(game, &provider);
            ui_state.message = format!("完成 {} 条结算记录", report.entries.len());
            ui_state.selected_city_id = first_player_city(game);
        }
        if ui.button("清空待命令").clicked() {
            game.pending_commands.clear();
            ui_state.message = "已清空玩家待命令".to_string();
        }
    });
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
    if ui.button("进入城市").clicked() {
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
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (city_id, city_name, faction_name) in rows {
                let selected = ui_state.selected_city_id.as_deref() == Some(city_id.as_str());
                let response =
                    ui.selectable_label(selected, format!("{} ({})", city_name, faction_name));
                if response.clicked() {
                    ui_state.selected_city_id = Some(city_id.clone());
                }
                if response.double_clicked() {
                    open_city(ui_state, city_id.clone());
                }
                response.context_menu(|ui| {
                    if ui.button("进入城市").clicked() {
                        open_city(ui_state, city_id.clone());
                        ui.close();
                    }
                });
            }
        });
}

fn open_city(ui_state: &mut GameUiState, city_id: CityId) {
    ui_state.selected_city_id = Some(city_id);
    ui_state.in_game_view = InGameView::City;
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
    ui.heading("存档");
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
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(24, 29, 34));

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

    for road in &game.roads {
        let Some(from) = game.cities.get(&road.from) else {
            continue;
        };
        let Some(to) = game.cities.get(&road.to) else {
            continue;
        };
        let a = map_to_screen(from.position, bounds, rect, ui_state);
        let b = map_to_screen(to.position, bounds, rect, ui_state);
        painter.line_segment([a, b], egui::Stroke::new(3.0, egui::Color32::from_gray(96)));
    }

    for city in game.cities.values() {
        let pos = map_to_screen(city.position, bounds, rect, ui_state);
        let faction = &game.factions[&city.faction_id];
        let color = faction_color(faction);
        let selected = ui_state.selected_city_id.as_deref() == Some(city.id.as_str());
        painter.circle_filled(pos, 20.0, color);
        painter.circle_stroke(
            pos,
            if selected { 26.0 } else { 22.0 },
            egui::Stroke::new(
                if selected { 4.0 } else { 2.0 },
                if selected {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_gray(35)
                },
            ),
        );
        painter.text(
            pos + egui::vec2(0.0, 31.0),
            egui::Align2::CENTER_CENTER,
            &city.name,
            egui::FontId::proportional(16.0),
            egui::Color32::WHITE,
        );
        painter.text(
            pos,
            egui::Align2::CENTER_CENTER,
            city.troops.to_string(),
            egui::FontId::proportional(12.0),
            egui::Color32::BLACK,
        );
    }

    let picked_city = response
        .interact_pointer_pos()
        .and_then(|pointer_pos| city_at_position(game, bounds, rect, pointer_pos, ui_state));

    if response.clicked() || response.secondary_clicked() {
        if let Some(city_id) = picked_city.clone() {
            ui_state.selected_city_id = Some(city_id);
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
            if ui.button("进入城市").clicked() {
                open_city(ui_state, city_id);
                ui.close();
            }
        }
    });
}

fn report_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    let Some(game) = &ui_state.game else {
        return;
    };
    ui.heading("回合报告");
    egui::ScrollArea::vertical().show(ui, |ui| {
        for report in game.reports.iter().rev().take(8) {
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
    let padding = 70.0;
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
