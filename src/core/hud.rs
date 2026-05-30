use crate::game::*;
use bevy_egui::egui;

use super::actions::{
    clear_pending_commands, enter_game, finish_current_turn, open_city, refresh_saves,
};
use super::city_panel::selected_city_panel;
use super::map::{map_panel, reset_map_view, zoom_map};
use super::state::{GameUiState, Screen};
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
