use crate::game::*;
use bevy_egui::egui;

use super::super::actions::{
    clear_pending_commands, confirm_return_main_menu, finish_current_turn, request_return_main_menu,
};
use super::super::i18n::{Translator, args};
use super::super::state::GameUiState;
use super::super::style::{
    modal_content_width, modal_title_bar, war_bar_frame, war_gold, war_panel_frame, war_success,
    war_text, war_text_muted,
};
use super::super::{HUD_MARGIN, HUD_TOP_OFFSET};
use super::events::{ensure_selected_event, event_button_label};
pub(super) fn top_status_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    let width = (screen.width() - HUD_MARGIN * 2.0).max(320.0);
    let summary = ui_state.game.as_ref().map(|game| {
        let faction_name = game
            .factions
            .get(&game.player_faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| t.text("unknown-faction"));
        let resources = FactionResourceSummary::from_game(game, &game.player_faction_id);
        let status = match &game.status {
            GameStatus::Running => None,
            GameStatus::Victory { reason } => {
                Some(t.text_args("game-status-victory", &args([("reason", reason.clone())])))
            }
            GameStatus::Defeat { reason } => {
                Some(t.text_args("game-status-defeat", &args([("reason", reason.clone())])))
            }
        };
        (
            game.scenario_name.clone(),
            game.year,
            game.month,
            game.turn,
            faction_name,
            resources,
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
                        egui::RichText::new(t.text("app-title"))
                            .size(24.0)
                            .color(war_gold())
                            .strong(),
                    );
                    ui.separator();
                    if let Some((scenario, year, month, turn, faction_name, resources, status)) =
                        summary
                    {
                        ui.label(t.text_args(
                            "hud-date-turn",
                            &args([
                                ("scenario", scenario),
                                ("year", year.to_string()),
                                ("month", month.to_string()),
                                ("turn", turn.to_string()),
                            ]),
                        ));
                        ui.label(t.text_args("hud-player", &args([("faction", faction_name)])));
                        resources.ui(ui, t);
                        if let Some(status) = status {
                            ui.colored_label(egui::Color32::from_rgb(200, 72, 52), status);
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t.text("hud-main-menu")).clicked() {
                            request_return_main_menu(ui_state);
                        }
                        if ui.button(t.text("hud-save")).clicked() {
                            ui_state.save_panel_open = !ui_state.save_panel_open;
                        }
                        if ui.button(t.text("hud-clear-commands")).clicked() {
                            clear_pending_commands(ui_state);
                        }
                        if ui.button(t.text("hud-end-month")).clicked() {
                            finish_current_turn(ui_state);
                        }
                    });
                });
            });
        });
}

pub(super) fn bottom_map_actions_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
) {
    egui::Area::new(egui::Id::new("hud_bottom_map_actions"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_BOTTOM,
            egui::vec2(-HUD_MARGIN, -HUD_MARGIN),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(t.text("hud-cities")).clicked() {
                        ui_state.city_list_open = !ui_state.city_list_open;
                    }

                    if ui.button(t.text("hud-factions")).clicked() {
                        ui_state.faction_overview_open = !ui_state.faction_overview_open;
                    }

                    if ui.button(t.text("hud-officers")).clicked() {
                        ui_state.officer_browser_open = !ui_state.officer_browser_open;
                    }

                    if ui.button(t.text("hud-retainers")).clicked() {
                        ui_state.retainers_open = !ui_state.retainers_open;
                    }

                    if ui.button(t.text("hud-shrine")).clicked() {
                        ui_state.shrine_open = !ui_state.shrine_open;
                    }

                    if ui.button(t.text("hud-technology")).clicked() {
                        ui_state.technology_open = !ui_state.technology_open;
                    }

                    let event_label = event_button_label(ui_state, t);
                    if ui.button(event_label).clicked() {
                        ui_state.events_open = !ui_state.events_open;
                        if ui_state.events_open {
                            ensure_selected_event(ui_state);
                        }
                    }
                });
            });
        });
}

pub(super) fn return_main_menu_confirm_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.return_main_menu_confirm_open {
        return;
    }

    egui::Area::new(egui::Id::new("hud_return_main_menu_confirm_scrim"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150),
            );
            if response.clicked() {
                ui_state.return_main_menu_confirm_open = false;
            }
        });

    let width = modal_content_width(screen, 420.0);
    let button_size = egui::vec2(112.0, 34.0);
    egui::Area::new(egui::Id::new("hud_return_main_menu_confirm"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                if modal_title_bar(ui, t, &t.text("hud-main-menu-confirm-title")) {
                    ui_state.return_main_menu_confirm_open = false;
                }
                ui.separator();
                ui.set_max_width(width);
                ui.label(egui::RichText::new(t.text("hud-main-menu-confirm-heading")).strong());
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(t.text("hud-main-menu-confirm-message"))
                            .color(war_text_muted()),
                    )
                    .wrap(),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(button_size, egui::Button::new(t.text("common-confirm")))
                        .clicked()
                    {
                        confirm_return_main_menu(ui_state);
                    }
                    if ui
                        .add_sized(button_size, egui::Button::new(t.text("common-cancel")))
                        .clicked()
                    {
                        ui_state.return_main_menu_confirm_open = false;
                    }
                });
            });
        });
}

#[derive(Clone, Debug, Default)]
struct FactionResourceSummary {
    city_count: usize,
    gold: i64,
    food: i64,
    materials: i64,
    troops: u64,
    population: u64,
    wounded: u64,
}

impl FactionResourceSummary {
    fn from_game(game: &GameState, faction_id: &str) -> Self {
        let mut summary = Self::default();
        for city in game
            .cities
            .values()
            .filter(|city| city.faction_id == faction_id)
        {
            summary.city_count += 1;
            summary.gold += i64::from(city.gold);
            summary.food += i64::from(city.food);
            summary.materials += i64::from(city.materials);
            summary.troops += u64::from(city.troops.total());
            summary.population += u64::from(city.population);
            summary.wounded += u64::from(city.wounded_troops.total());
        }
        summary
    }

    fn ui(&self, ui: &mut egui::Ui, t: &Translator) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            self.resource_badge(
                ui,
                egui_phosphor::regular::COINS,
                self.gold.to_string(),
                war_gold(),
                &t.text("resource-gold"),
            );
            self.resource_badge(
                ui,
                egui_phosphor::regular::GRAINS,
                self.food.to_string(),
                war_success(),
                &t.text("resource-food"),
            );
            self.resource_badge(
                ui,
                egui_phosphor::regular::STACK,
                self.materials.to_string(),
                egui::Color32::from_rgb(176, 153, 116),
                &t.text("resource-materials"),
            );
            self.resource_badge(
                ui,
                egui_phosphor::regular::SWORD,
                self.troops.to_string(),
                egui::Color32::from_rgb(185, 128, 96),
                &t.text("resource-troops"),
            );
        })
        .response
        .on_hover_text(self.tooltip(t));
    }

    fn resource_badge(
        &self,
        ui: &mut egui::Ui,
        icon: &str,
        value: String,
        color: egui::Color32,
        tooltip: &str,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;
            ui.label(egui::RichText::new(icon).size(15.0).color(color));
            ui.label(egui::RichText::new(value).color(war_text()));
        })
        .response
        .on_hover_text(tooltip);
    }

    fn tooltip(&self, t: &Translator) -> String {
        t.text_args(
            "hud-faction-resources-tooltip",
            &args([
                ("cities", self.city_count.to_string()),
                ("population", self.population.to_string()),
                ("wounded", self.wounded.to_string()),
            ]),
        )
    }
}
