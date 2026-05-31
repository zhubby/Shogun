use crate::game::*;
use bevy_egui::egui;

use super::actions::{
    clear_pending_commands, enter_game, finish_current_turn, open_city, refresh_saves,
};
use super::city_panel::selected_city_panel;
use super::labels::officer_gender_label;
use super::map::{map_panel, reset_map_view, zoom_map};
use super::state::{
    GameUiState, OfficerBrowserFilters, OfficerGenderFilter, OfficerStatusFilter, Screen,
};
use super::style::{war_bar_frame, war_gold, war_panel_frame, war_text_muted};
use super::{CITY_DRAWER_WIDTH, HUD_MARGIN, HUD_TOP_HEIGHT, HUD_TOP_OFFSET, MAP_ZOOM_STEP};

pub(super) fn in_game(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            map_panel(ui, ui_state);
        });

    in_game_hud(ctx, ui_state);
}

pub(super) fn in_game_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    let screen = ctx.content_rect();
    top_status_hud(ctx, ui_state, screen);
    left_map_hud(ctx, ui_state);
    city_list_hud(ctx, ui_state, screen);
    save_hud(ctx, ui_state, screen);
    city_drawer_hud(ctx, ui_state, screen);
    report_hud(ctx, ui_state, screen);
    bottom_map_actions_hud(ctx, ui_state);
    officer_browser_hud(ctx, ui_state, screen);
}

pub(super) fn top_status_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
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

pub(super) fn left_map_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
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
            });
        });
}

pub(super) fn bottom_map_actions_hud(ctx: &egui::Context, ui_state: &mut GameUiState) {
    egui::Area::new(egui::Id::new("hud_bottom_map_actions"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            egui::vec2(-HUD_MARGIN, -HUD_MARGIN),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let city_label = if ui_state.city_list_open {
                        "收起城池"
                    } else {
                        "城池"
                    };
                    if ui.button(city_label).clicked() {
                        ui_state.city_list_open = !ui_state.city_list_open;
                    }

                    let officer_label = if ui_state.officer_browser_open {
                        "收起武将"
                    } else {
                        "武将"
                    };
                    if ui.button(officer_label).clicked() {
                        ui_state.officer_browser_open = !ui_state.officer_browser_open;
                    }
                });
            });
        });
}

pub(super) fn city_list_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
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

pub(super) fn save_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
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

pub(super) fn city_drawer_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
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

pub(super) fn report_hud(ctx: &egui::Context, ui_state: &mut GameUiState, screen: egui::Rect) {
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

pub(super) fn officer_browser_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    screen: egui::Rect,
) {
    if !ui_state.officer_browser_open {
        return;
    }

    let width = (screen.width() * 0.82).clamp(720.0, 1080.0);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_officer_browser"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("武将").color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("关闭").clicked() {
                            ui_state.officer_browser_open = false;
                        }
                    });
                });
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.officer_browser_filters,
                        "hud_officer_browser_filters",
                    );
                } else {
                    ui.label("当前没有剧本局面。");
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_table(
                        ui,
                        game,
                        &ui_state.officer_browser_filters,
                        height - 118.0,
                        "hud_officer_browser_table",
                    );
                }
            });
        });
}

pub(super) fn selected_city_summary(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
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

pub(super) fn map_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
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

pub(super) fn city_list(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
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

pub(super) fn officer_browser_filters(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &mut OfficerBrowserFilters,
    id_salt: &'static str,
) {
    const FILTER_HEIGHT: f32 = 40.0;
    ui.horizontal(|ui| {
        ui.set_min_height(FILTER_HEIGHT);
        ui.spacing_mut().interact_size.y = FILTER_HEIGHT;
        ui.spacing_mut().item_spacing.x = 16.0;
        ui.label("搜索");
        ui.add_sized(
            [260.0, FILTER_HEIGHT],
            egui::TextEdit::singleline(&mut filters.search).hint_text("姓名 / ID / 字 / 籍贯"),
        );

        egui::ComboBox::from_id_salt((id_salt, "gender"))
            .width(160.0)
            .height(FILTER_HEIGHT)
            .selected_text(officer_gender_filter_label(filters.gender))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerGenderFilter::All,
                    OfficerGenderFilter::Male,
                    OfficerGenderFilter::Female,
                ] {
                    ui.selectable_value(
                        &mut filters.gender,
                        filter,
                        officer_gender_filter_label(filter),
                    );
                }
            });

        egui::ComboBox::from_id_salt((id_salt, "faction"))
            .width(170.0)
            .height(FILTER_HEIGHT)
            .selected_text(
                filters
                    .faction_id
                    .as_deref()
                    .and_then(|id| game.factions.get(id))
                    .map(|faction| faction.name.as_str())
                    .unwrap_or("全部势力"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filters.faction_id, None, "全部势力");
                for faction in game.factions.values() {
                    ui.selectable_value(
                        &mut filters.faction_id,
                        Some(faction.id.clone()),
                        &faction.name,
                    );
                }
            });

        egui::ComboBox::from_id_salt((id_salt, "status"))
            .width(170.0)
            .height(FILTER_HEIGHT)
            .selected_text(officer_status_filter_label(filters.status))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerStatusFilter::All,
                    OfficerStatusFilter::Active,
                    OfficerStatusFilter::Wild,
                    OfficerStatusFilter::Unavailable,
                    OfficerStatusFilter::Dead,
                ] {
                    ui.selectable_value(
                        &mut filters.status,
                        filter,
                        officer_status_filter_label(filter),
                    );
                }
            });

        egui::ComboBox::from_id_salt((id_salt, "city"))
            .width(170.0)
            .height(FILTER_HEIGHT)
            .selected_text(
                filters
                    .city_id
                    .as_deref()
                    .and_then(|id| game.cities.get(id))
                    .map(|city| city.name.as_str())
                    .unwrap_or("全部城池"),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filters.city_id, None, "全部城池");
                let mut cities: Vec<_> = game.cities.values().collect();
                cities.sort_by(|a, b| a.name.cmp(&b.name));
                for city in cities {
                    ui.selectable_value(&mut filters.city_id, Some(city.id.clone()), &city.name);
                }
            });

        if ui
            .add_sized([86.0, FILTER_HEIGHT], egui::Button::new("重置"))
            .clicked()
        {
            filters.reset();
        }
    });
}

pub(super) fn officer_browser_table(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &OfficerBrowserFilters,
    max_height: f32,
    id_salt: &'static str,
) {
    let rows = filtered_officer_rows(filters, game);
    ui.label(format!("共 {} 名武将", rows.len()));
    egui::ScrollArea::vertical()
        .id_salt(id_salt)
        .max_height(max_height.max(260.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Grid::new((id_salt, "grid"))
                .striped(true)
                .num_columns(11)
                .min_col_width(42.0)
                .spacing(egui::vec2(12.0, 6.0))
                .show(ui, |ui| {
                    ui.strong("姓名");
                    ui.strong("性别");
                    ui.strong("势力");
                    ui.strong("城池");
                    ui.strong("状态");
                    ui.strong("忠诚");
                    ui.strong("统");
                    ui.strong("武");
                    ui.strong("智");
                    ui.strong("政");
                    ui.strong("魅");
                    ui.end_row();

                    for row in rows {
                        ui.label(row.name);
                        ui.label(row.gender);
                        ui.label(row.faction_name);
                        ui.label(row.city_name);
                        ui.label(row.status);
                        ui.label(row.loyalty.to_string());
                        ui.label(row.leadership.to_string());
                        ui.label(row.strength.to_string());
                        ui.label(row.intelligence.to_string());
                        ui.label(row.politics.to_string());
                        ui.label(row.charm.to_string());
                        ui.end_row();
                    }
                });
        });
}

#[derive(Clone, Debug)]
pub(super) struct OfficerBrowserRow {
    name: String,
    gender: &'static str,
    faction_name: String,
    city_name: String,
    status: &'static str,
    loyalty: u8,
    leadership: u8,
    strength: u8,
    intelligence: u8,
    politics: u8,
    charm: u8,
}

pub(super) fn filtered_officer_rows(
    filters: &OfficerBrowserFilters,
    game: &GameState,
) -> Vec<OfficerBrowserRow> {
    let search = filters.search.trim().to_lowercase();
    let mut officers: Vec<_> = game.officers.values().collect();
    officers.sort_by(|a, b| {
        let a_faction = game
            .factions
            .get(&a.faction_id)
            .map(|faction| faction.name.as_str())
            .unwrap_or("未知");
        let b_faction = game
            .factions
            .get(&b.faction_id)
            .map(|faction| faction.name.as_str())
            .unwrap_or("未知");
        let a_city = a
            .city_id
            .as_deref()
            .and_then(|city_id| game.cities.get(city_id))
            .map(|city| city.name.as_str())
            .unwrap_or("未配置");
        let b_city = b
            .city_id
            .as_deref()
            .and_then(|city_id| game.cities.get(city_id))
            .map(|city| city.name.as_str())
            .unwrap_or("未配置");
        (a_faction, a_city, a.name.as_str(), a.id.as_str()).cmp(&(
            b_faction,
            b_city,
            b.name.as_str(),
            b.id.as_str(),
        ))
    });

    officers
        .into_iter()
        .filter(|officer| officer_matches_filters(officer, game, filters, &search))
        .map(|officer| {
            let faction_name = game
                .factions
                .get(&officer.faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| "未知".to_string());
            let city_name = officer
                .city_id
                .as_deref()
                .and_then(|city_id| game.cities.get(city_id))
                .map(|city| city.name.clone())
                .unwrap_or_else(|| "未配置".to_string());
            OfficerBrowserRow {
                name: officer.name.clone(),
                gender: officer_gender_label(&officer.gender),
                faction_name,
                city_name,
                status: officer_status_label(&officer.status),
                loyalty: officer.loyalty,
                leadership: officer.stats.leadership,
                strength: officer.stats.strength,
                intelligence: officer.stats.intelligence,
                politics: officer.stats.politics,
                charm: officer.stats.charm,
            }
        })
        .collect()
}

fn officer_matches_filters(
    officer: &Officer,
    game: &GameState,
    filters: &OfficerBrowserFilters,
    search: &str,
) -> bool {
    if !officer_gender_matches(&officer.gender, filters.gender) {
        return false;
    }
    if filters
        .faction_id
        .as_ref()
        .is_some_and(|faction_id| officer.faction_id != *faction_id)
    {
        return false;
    }
    if !officer_status_matches(&officer.status, filters.status) {
        return false;
    }
    if filters
        .city_id
        .as_ref()
        .is_some_and(|city_id| officer.city_id.as_deref() != Some(city_id.as_str()))
    {
        return false;
    }
    search.is_empty() || officer_search_text(officer, game).contains(search)
}

fn officer_search_text(officer: &Officer, game: &GameState) -> String {
    let faction_name = game
        .factions
        .get(&officer.faction_id)
        .map(|faction| faction.name.as_str())
        .unwrap_or("未知");
    let city_name = officer
        .city_id
        .as_deref()
        .and_then(|city_id| game.cities.get(city_id))
        .map(|city| city.name.as_str())
        .unwrap_or("未配置");
    let mut text = format!(
        "{} {} {} {}",
        officer.name, officer.id, faction_name, city_name
    );
    if let Some(profile) = &officer.profile {
        if let Some(courtesy) = &profile.courtesy_name {
            text.push(' ');
            text.push_str(courtesy);
        }
        if let Some(native_place) = &profile.native_place {
            text.push(' ');
            text.push_str(native_place);
        }
    }
    text.to_lowercase()
}

fn officer_gender_matches(gender: &OfficerGender, filter: OfficerGenderFilter) -> bool {
    match filter {
        OfficerGenderFilter::All => true,
        OfficerGenderFilter::Male => *gender == OfficerGender::Male,
        OfficerGenderFilter::Female => *gender == OfficerGender::Female,
    }
}

fn officer_status_matches(status: &OfficerStatus, filter: OfficerStatusFilter) -> bool {
    match filter {
        OfficerStatusFilter::All => true,
        OfficerStatusFilter::Active => *status == OfficerStatus::Active,
        OfficerStatusFilter::Wild => *status == OfficerStatus::Wild,
        OfficerStatusFilter::Unavailable => *status == OfficerStatus::Unavailable,
        OfficerStatusFilter::Dead => *status == OfficerStatus::Dead,
    }
}

fn officer_gender_filter_label(filter: OfficerGenderFilter) -> &'static str {
    match filter {
        OfficerGenderFilter::All => "全部性别",
        OfficerGenderFilter::Male => "男",
        OfficerGenderFilter::Female => "女",
    }
}

fn officer_status_filter_label(filter: OfficerStatusFilter) -> &'static str {
    match filter {
        OfficerStatusFilter::All => "全部状态",
        OfficerStatusFilter::Active => "在任",
        OfficerStatusFilter::Wild => "在野",
        OfficerStatusFilter::Unavailable => "不可用",
        OfficerStatusFilter::Dead => "死亡",
    }
}

fn officer_status_label(status: &OfficerStatus) -> &'static str {
    match status {
        OfficerStatus::Active => "在任",
        OfficerStatus::Wild => "在野",
        OfficerStatus::Unavailable => "不可用",
        OfficerStatus::Dead => "死亡",
    }
}

pub(super) fn save_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
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
        if ui.button("保存").clicked()
            && let Some(game) = &ui_state.game
        {
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

pub(super) fn report_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState, screen: egui::Rect) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::display_settings::{DisplaySettings, LoadedDisplaySettings};
    use crate::core::state::{GameUiState, OfficerGenderFilter, OfficerStatusFilter};
    use crate::game::{OfficerGender, ScenarioData};

    fn ui_state_with_game() -> GameUiState {
        let mut state = GameUiState::new(
            crate::core::display_settings::DisplaySettingsStore::with_default_path(),
            LoadedDisplaySettings {
                settings: DisplaySettings::default(),
                message: None,
            },
        );
        state.game = Some(
            ScenarioData::default_scenario()
                .unwrap()
                .build_game("liu_bei")
                .unwrap(),
        );
        state
    }

    #[test]
    fn officer_browser_search_matches_name_id_faction_and_city() {
        let mut state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        state.officer_browser_filters.search = "关羽".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game)
                .iter()
                .any(|row| row.name == "关羽")
        );

        state.officer_browser_filters.search = "guan_yu".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game)
                .iter()
                .any(|row| row.name == "关羽")
        );

        state.officer_browser_filters.search = "刘备军".to_string();
        assert!(!filtered_officer_rows(&state.officer_browser_filters, game).is_empty());

        state.officer_browser_filters.search = "平原".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game)
                .iter()
                .all(|row| row.city_name == "平原")
        );
    }

    #[test]
    fn officer_browser_filters_can_be_combined() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            let officer = game.officers.get_mut("guan_yu").unwrap();
            officer.gender = OfficerGender::Male;
        }
        state.officer_browser_filters.gender = OfficerGenderFilter::Male;
        state.officer_browser_filters.faction_id = Some("liu_bei".to_string());
        state.officer_browser_filters.status = OfficerStatusFilter::Active;
        state.officer_browser_filters.city_id = Some("pingyuan".to_string());

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game);

        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| {
            row.gender == "男"
                && row.faction_name == "刘备军"
                && row.status == "在任"
                && row.city_name == "平原"
        }));
    }

    #[test]
    fn officer_browser_empty_filters_return_all_officers_stably_sorted() {
        let state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        let rows = filtered_officer_rows(&state.officer_browser_filters, game);
        let sorted_names = rows.windows(2).all(|pair| {
            (&pair[0].faction_name, &pair[0].city_name, &pair[0].name)
                <= (&pair[1].faction_name, &pair[1].city_name, &pair[1].name)
        });

        assert_eq!(rows.len(), game.officers.len());
        assert!(sorted_names);
    }
}
