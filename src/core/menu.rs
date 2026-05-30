use bevy_egui::egui;

use crate::build_info::menu_build_label;

use super::actions::{enter_game, refresh_saves, start_history_game, start_json_game};
use super::settings::settings_modal;
use super::state::{refresh_history_factions, refresh_history_menu, GameUiState};
use super::style::{
    draw_menu_background, war_bar_frame, war_gold, war_panel_frame, war_sub_panel_frame,
    war_text_muted,
};
use super::{HUD_MARGIN, HUD_TOP_OFFSET};

pub(super) fn main_menu(ctx: &egui::Context, ui_state: &mut GameUiState) -> bool {
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
                        ui.label(
                            egui::RichText::new(menu_build_label())
                                .size(12.0)
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

pub(super) fn new_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
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

pub(super) fn load_game_menu(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
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

pub(super) fn main_menu_settings_button(ctx: &egui::Context, ui_state: &mut GameUiState) {
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
