use crate::game::*;
use bevy_egui::egui;
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use std::collections::{BTreeMap, BTreeSet};

use super::actions::{
    clear_pending_commands, enter_game, finish_current_turn, open_city, refresh_saves,
};
use super::city_intel::city_summary_intel;
use super::city_panel::selected_city_panel;
use super::i18n::{Translator, UiLanguage, args};
use super::labels::{
    confidence_label, officer_gender_label, officer_relationship_label, technology_branch_label,
};
use super::map::{draw_city_marker_icon, faction_color, map_panel, reset_map_view, zoom_map};
use super::officer_portrait_ui::{officer_portrait_status_line, paint_officer_portrait_preview};
use super::portraits::{
    OFFICER_PORTRAIT_ASPECT_HEIGHT, OFFICER_PORTRAIT_ASPECT_WIDTH, OfficerPortraitStore,
    OfficerPortraitTaskState, officer_portrait_path,
};
use super::runtime::CoreAsyncRuntime;
use super::state::{
    GameUiState, OfficerBrowserFilters, OfficerEditDraft, OfficerGenderFilter, OfficerStatusFilter,
    Screen, ShrineTab,
};
use super::style::{
    collapse_icon_button, modal_content_width, modal_title_bar, war_bar_frame, war_border,
    war_danger, war_gold, war_panel_frame, war_sub_panel_frame, war_success, war_text,
    war_text_muted, war_warning,
};
use super::{HUD_MARGIN, HUD_TOP_HEIGHT, HUD_TOP_OFFSET, MAP_ZOOM_STEP};

const OFFICER_DETAIL_PORTRAIT_WIDTH: f32 = 224.0;
pub(super) const OFFICER_BROWSER_MODAL_WIDTH: f32 = 1060.0;

pub(super) fn in_game(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    async_runtime: &CoreAsyncRuntime,
) {
    ui_state.officer_portraits.poll_task_events();
    let t = Translator::new(ui_state.applied_settings.general.ui_language);
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            map_panel(ui, ui_state, &t);
        });

    in_game_hud(ctx, ui_state, &t, async_runtime);
}

pub(super) fn in_game_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    async_runtime: &CoreAsyncRuntime,
) {
    let screen = ctx.content_rect();
    top_status_hud(ctx, ui_state, t, screen);
    map_controls_hud(ctx, ui_state, t);
    left_city_summary_hud(ctx, ui_state, t);
    city_list_hud(ctx, ui_state, t, screen);
    save_hud(ctx, ui_state, t, screen);
    city_drawer_hud(ctx, ui_state, t, screen);
    report_hud(ctx, ui_state, t, screen);
    bottom_map_actions_hud(ctx, ui_state, t);
    officer_browser_hud(ctx, ui_state, t, screen);
    retainer_hud(ctx, ui_state, t, screen);
    shrine_hud(ctx, ui_state, t, screen);
    officer_detail_modal(ctx, ui_state, t, screen, async_runtime);
    technology_hud(ctx, ui_state, t, screen);
    event_center_hud(ctx, ui_state, t, screen);
    event_popup_hud(ctx, ui_state, t, screen);
}

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
                            ui_state.screen = Screen::MainMenu;
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

pub(super) fn map_controls_hud(ctx: &egui::Context, ui_state: &mut GameUiState, t: &Translator) {
    egui::Area::new(egui::Id::new("hud_map_controls"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(270.0);
                map_controls(ui, ui_state, t);
            });
        });
}

pub(super) fn left_city_summary_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
) {
    egui::Area::new(egui::Id::new("hud_left_city_summary"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(285.0);
                selected_city_summary(ui, ui_state, t);
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

pub(super) fn technology_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.technology_open {
        return;
    }

    let width = (screen.width() * 0.84).clamp(780.0, 1120.0);
    let height = (screen.height() * 0.78).clamp(500.0, 720.0);
    egui::Area::new(egui::Id::new("hud_technology"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("technology-title")) {
                    ui_state.technology_open = false;
                }
                ui.separator();
                technology_panel(ui, ui_state, t, width, height);
            });
        });
}

pub(super) fn event_center_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.events_open {
        return;
    }
    ensure_selected_event(ui_state);

    let width = (screen.width() * 0.82).clamp(760.0, 1120.0);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_event_center"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("event-center-title")) {
                    ui_state.events_open = false;
                }
                ui.separator();
                event_center(ui, ui_state, t, width, height);
            });
        });
}

pub(super) fn event_popup_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    let Some(event_id) = ui_state
        .game
        .as_ref()
        .and_then(|game| popup_event_id(game).map(str::to_string))
    else {
        return;
    };
    let Some(event) = ui_state
        .game
        .as_ref()
        .and_then(|game| game.events.iter().find(|event| event.id == event_id))
        .cloned()
    else {
        return;
    };

    let width = (screen.width() * 0.42).clamp(420.0, 560.0);
    egui::Area::new(egui::Id::new("hud_event_popup"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-HUD_MARGIN, -86.0))
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                if modal_title_bar(ui, t, &event.title)
                    && let Some(game) = &mut ui_state.game
                {
                    let _ = dismiss_event_popup(game, &event_id);
                }
                ui.separator();
                ui.label(egui::RichText::new(&event.summary).strong());
                ui.colored_label(war_text_muted(), event_time_label(&event));
                ui.add_space(6.0);
                ui.label(&event.detail);
                if let EventResolution::PendingDecision {
                    deadline_turn,
                    choices,
                    ..
                } = &event.resolution
                {
                    ui.add_space(8.0);
                    ui.colored_label(
                        war_warning(),
                        t.text_args(
                            "event-deadline",
                            &args([("turn", deadline_turn.to_string())]),
                        ),
                    );
                    ui.horizontal_wrapped(|ui| {
                        for choice in choices {
                            if ui.button(&choice.label).clicked() {
                                resolve_event_from_ui(ui_state, &event_id, &choice.id);
                            }
                        }
                    });
                } else if ui.button(t.text("event-acknowledge")).clicked()
                    && let Some(game) = &mut ui_state.game
                {
                    let _ = dismiss_event_popup(game, &event_id);
                }
            });
        });
}

fn event_button_label(ui_state: &GameUiState, t: &Translator) -> String {
    let Some(game) = &ui_state.game else {
        return t.text("hud-events");
    };
    let pending = pending_event_count(game);
    if pending > 0 {
        return t.text_args(
            "hud-events-pending",
            &args([("count", pending.to_string())]),
        );
    }
    let unread = unread_event_count(game);
    if unread > 0 {
        return t.text_args("hud-events-unread", &args([("count", unread.to_string())]));
    }
    t.text("hud-events")
}

fn ensure_selected_event(ui_state: &mut GameUiState) {
    let Some(game) = &ui_state.game else {
        ui_state.selected_event_id = None;
        return;
    };
    let selected_exists = ui_state
        .selected_event_id
        .as_deref()
        .is_some_and(|id| game.events.iter().any(|event| event.id == id));
    if !selected_exists {
        ui_state.selected_event_id = game
            .events
            .iter()
            .max_by_key(|event| event.sequence)
            .map(|event| event.id.clone());
    }
}

fn event_center(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    width: f32,
    height: f32,
) {
    let Some(game) = ui_state.game.as_ref() else {
        ui.label(t.text("message-no-game-state"));
        return;
    };
    if game.events.is_empty() {
        ui.label(t.text("event-empty"));
        return;
    }

    ui.columns(2, |columns| {
        columns[0].set_width(width * 0.38);
        war_sub_panel_frame().show(&mut columns[0], |ui| {
            ui.label(
                egui::RichText::new(t.text("event-list-title"))
                    .color(war_gold())
                    .strong(),
            );
            ui.add_space(6.0);
            event_list(ui, ui_state, t, height - 112.0);
        });
        war_sub_panel_frame().show(&mut columns[1], |ui| {
            ui.label(
                egui::RichText::new(t.text("event-detail-title"))
                    .color(war_gold())
                    .strong(),
            );
            ui.add_space(6.0);
            event_detail(ui, ui_state, t, height - 112.0);
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

fn event_list(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator, max_height: f32) {
    let rows: Vec<GameEvent> = ui_state
        .game
        .as_ref()
        .map(|game| {
            game.events
                .iter()
                .rev()
                .cloned()
                .collect::<Vec<GameEvent>>()
        })
        .unwrap_or_default();
    egui::ScrollArea::vertical()
        .id_salt("event_center_list")
        .max_height(max_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for event in rows {
                let selected = ui_state.selected_event_id.as_deref() == Some(event.id.as_str());
                let response = ui.selectable_label(
                    selected,
                    format!(
                        "{}  {}  {}",
                        event_status_label(&event, t),
                        event_time_label(&event),
                        event.summary
                    ),
                );
                if response.clicked() {
                    ui_state.selected_event_id = Some(event.id.clone());
                    if let Some(game) = &mut ui_state.game {
                        let _ = mark_event_viewed(game, &event.id);
                    }
                }
            }
        });
}

fn event_detail(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator, max_height: f32) {
    let Some(event_id) = ui_state.selected_event_id.clone() else {
        ui.label(t.text("event-none-selected"));
        return;
    };
    if let Some(game) = &mut ui_state.game {
        let _ = mark_event_viewed(game, &event_id);
    }
    let Some(event) = ui_state
        .game
        .as_ref()
        .and_then(|game| game.events.iter().find(|event| event.id == event_id))
        .cloned()
    else {
        ui.label(t.text("event-none-selected"));
        return;
    };

    egui::ScrollArea::vertical()
        .id_salt("event_center_detail")
        .max_height(max_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.heading(egui::RichText::new(&event.title).color(war_gold()));
            ui.horizontal_wrapped(|ui| {
                ui.label(event_time_label(&event));
                ui.separator();
                ui.label(event_kind_label(&event.kind, t));
                ui.separator();
                ui.colored_label(
                    event_severity_color(&event.severity),
                    event_status_label(&event, t),
                );
            });
            ui.add_space(8.0);
            ui.label(egui::RichText::new(&event.summary).strong());
            ui.add_space(6.0);
            ui.label(&event.detail);
            ui.add_space(8.0);
            event_related_objects(ui, ui_state, &event, t);
            ui.add_space(10.0);
            event_resolution_detail(ui, ui_state, &event, t);
            if !ui_state.event_message.is_empty() {
                ui.add_space(8.0);
                ui.colored_label(war_warning(), &ui_state.event_message);
            }
        });
}

fn event_related_objects(
    ui: &mut egui::Ui,
    ui_state: &GameUiState,
    event: &GameEvent,
    t: &Translator,
) {
    let Some(game) = &ui_state.game else {
        return;
    };
    let city = event
        .city_id
        .as_deref()
        .and_then(|id| game.cities.get(id))
        .map(|city| city.name.clone());
    let faction = event
        .faction_id
        .as_deref()
        .and_then(|id| game.factions.get(id))
        .map(|faction| faction.name.clone());
    let officer = event
        .officer_id
        .as_deref()
        .and_then(|id| game.officers.get(id))
        .map(|officer| officer.name.clone());
    if city.is_none() && faction.is_none() && officer.is_none() {
        return;
    }
    ui.label(egui::RichText::new(t.text("event-related")).color(war_text_muted()));
    ui.horizontal_wrapped(|ui| {
        if let Some(city) = city {
            ui.label(t.text_args("event-related-city", &args([("city", city)])));
        }
        if let Some(faction) = faction {
            ui.label(t.text_args("event-related-faction", &args([("faction", faction)])));
        }
        if let Some(officer) = officer {
            ui.label(t.text_args("event-related-officer", &args([("officer", officer)])));
        }
    });
}

fn event_resolution_detail(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    event: &GameEvent,
    t: &Translator,
) {
    match &event.resolution {
        EventResolution::NoneRequired => {
            ui.colored_label(war_text_muted(), t.text("event-no-decision"));
        }
        EventResolution::PendingDecision {
            deadline_turn,
            choices,
            ..
        } => {
            ui.colored_label(
                war_warning(),
                t.text_args(
                    "event-deadline",
                    &args([("turn", deadline_turn.to_string())]),
                ),
            );
            ui.add_space(6.0);
            for choice in choices {
                war_sub_panel_frame().show(ui, |ui| {
                    ui.label(egui::RichText::new(&choice.label).strong());
                    ui.colored_label(war_text_muted(), &choice.description);
                    if ui.button(t.text("event-choose")).clicked() {
                        resolve_event_from_ui(ui_state, &event.id, &choice.id);
                    }
                });
                ui.add_space(4.0);
            }
        }
        EventResolution::Resolved { label, turn, .. } => {
            ui.colored_label(
                war_success(),
                t.text_args(
                    "event-resolved-detail",
                    &args([("label", label.clone()), ("turn", turn.to_string())]),
                ),
            );
        }
        EventResolution::Expired { label, turn, .. } => {
            ui.colored_label(
                war_warning(),
                t.text_args(
                    "event-expired-detail",
                    &args([("label", label.clone()), ("turn", turn.to_string())]),
                ),
            );
        }
        EventResolution::Cancelled { reason, turn } => {
            ui.colored_label(
                war_danger(),
                t.text_args(
                    "event-cancelled-detail",
                    &args([("reason", reason.clone()), ("turn", turn.to_string())]),
                ),
            );
        }
    }
}

fn resolve_event_from_ui(ui_state: &mut GameUiState, event_id: &str, choice_id: &str) {
    let Some(game) = &mut ui_state.game else {
        return;
    };
    match resolve_event_decision(game, event_id, choice_id) {
        Ok(()) => {
            ui_state.event_message.clear();
            ui_state.selected_event_id = Some(event_id.to_string());
        }
        Err(error) => {
            ui_state.event_message = error.to_string();
        }
    }
}

fn event_time_label(event: &GameEvent) -> String {
    format!("{}年{}月 第{}回合", event.year, event.month, event.turn)
}

fn event_status_label(event: &GameEvent, t: &Translator) -> String {
    match &event.resolution {
        EventResolution::PendingDecision { .. } => t.text("event-status-pending"),
        EventResolution::Resolved { .. } => t.text("event-status-resolved"),
        EventResolution::Expired { .. } => t.text("event-status-expired"),
        EventResolution::Cancelled { .. } => t.text("event-status-cancelled"),
        EventResolution::NoneRequired if !event.viewed => t.text("event-status-unread"),
        EventResolution::NoneRequired => t.text("event-status-recorded"),
    }
}

fn event_kind_label(kind: &GameEventKind, t: &Translator) -> String {
    match kind {
        GameEventKind::CityDevelopment => t.text("event-kind-city-development"),
        GameEventKind::Battle => t.text("event-kind-battle"),
        GameEventKind::CityCaptured => t.text("event-kind-city-captured"),
        GameEventKind::FactionDestroyed => t.text("event-kind-faction-destroyed"),
        GameEventKind::TechnologyCompleted => t.text("event-kind-technology-completed"),
        GameEventKind::HistoricalLife => t.text("event-kind-historical-life"),
        GameEventKind::OfficerLifecycle => t.text("event-kind-officer-lifecycle"),
        GameEventKind::Succession => t.text("event-kind-succession"),
        GameEventKind::Famine => t.text("event-kind-famine"),
        GameEventKind::GameStatus => t.text("event-kind-game-status"),
    }
}

fn event_severity_color(severity: &GameEventSeverity) -> egui::Color32 {
    match severity {
        GameEventSeverity::Info => war_text_muted(),
        GameEventSeverity::Important => war_gold(),
        GameEventSeverity::Warning => war_warning(),
        GameEventSeverity::Critical => war_danger(),
    }
}

fn technology_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    width: f32,
    height: f32,
) {
    let Some(game) = ui_state.game.as_ref().cloned() else {
        ui.label(t.text("message-no-game-state"));
        return;
    };
    let branch = ui_state.selected_technology_branch;
    if technology_spec(ui_state.selected_technology_id).branch != branch {
        ui_state.selected_technology_id = technology_specs_for_branch(branch)
            .next()
            .map(|spec| spec.id)
            .unwrap_or(TechnologyId::MilitiaDrill);
    }

    ui.horizontal(|ui| {
        for branch in [TechnologyBranch::Military, TechnologyBranch::Domestic] {
            if ui
                .selectable_label(
                    ui_state.selected_technology_branch == branch,
                    technology_branch_label(t, branch),
                )
                .clicked()
            {
                ui_state.selected_technology_branch = branch;
                ui_state.selected_technology_id = technology_specs_for_branch(branch)
                    .next()
                    .map(|spec| spec.id)
                    .unwrap_or(ui_state.selected_technology_id);
            }
        }
    });
    ui.add_space(8.0);

    ui.columns(2, |columns| {
        columns[0].set_width(width * 0.46);
        war_sub_panel_frame().show(&mut columns[0], |ui| {
            ui.label(
                egui::RichText::new(t.text("technology-tree"))
                    .color(war_gold())
                    .strong(),
            );
            ui.add_space(6.0);
            technology_tree(ui, ui_state, &game, t, height - 118.0);
        });

        war_sub_panel_frame().show(&mut columns[1], |ui| {
            ui.label(
                egui::RichText::new(t.text("technology-detail"))
                    .color(war_gold())
                    .strong(),
            );
            ui.add_space(6.0);
            technology_detail(ui, ui_state, &game, t);
        });
    });
}

fn technology_tree(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
    max_height: f32,
) {
    let faction_state = faction_technology_state(game, &game.player_faction_id);
    egui::ScrollArea::vertical()
        .id_salt("technology_tree")
        .max_height(max_height.max(320.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for spec in technology_specs_for_branch(ui_state.selected_technology_branch) {
                let selected = ui_state.selected_technology_id == spec.id;
                let status = technology_node_status(game, faction_state, spec);
                let response = technology_tree_node(ui, spec, selected, status, faction_state, t);
                if response.clicked() {
                    ui_state.selected_technology_id = spec.id;
                }
                response.on_hover_text(spec.effect);
                ui.add_space(7.0);
            }
        });
}

fn technology_tree_node(
    ui: &mut egui::Ui,
    spec: &TechnologySpec,
    selected: bool,
    status: TechnologyNodeStatus,
    faction_state: Option<&FactionTechnologyState>,
    t: &Translator,
) -> egui::Response {
    const ROW_HEIGHT: f32 = 48.0;
    let depth = technology_depth(spec);
    let indent = 18.0 + depth as f32 * 34.0;
    let available = ui.available_width().max(360.0);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(available, ROW_HEIGHT), egui::Sense::click());
    let painter = ui.painter_at(rect);
    let visuals = technology_status_visuals(status, t);
    let node_center = egui::pos2(rect.left() + indent + 12.0, rect.center().y);
    let has_parent = !spec.prerequisites.is_empty();

    if selected || response.hovered() {
        let fill = if selected {
            egui::Color32::from_rgba_unmultiplied(122, 59, 39, 170)
        } else {
            egui::Color32::from_rgba_unmultiplied(74, 56, 35, 135)
        };
        painter.rect(
            rect.shrink2(egui::vec2(1.0, 3.0)),
            5.0,
            fill,
            egui::Stroke::new(1.0, if selected { war_gold() } else { war_border() }),
            egui::StrokeKind::Inside,
        );
    }

    if depth > 0 {
        let trunk_x = rect.left() + indent - 18.0;
        let top = egui::pos2(trunk_x, rect.top() - 7.0);
        let bottom = egui::pos2(trunk_x, node_center.y);
        painter.line_segment(
            [top, bottom],
            egui::Stroke::new(1.0, tree_line_color(status)),
        );
        painter.line_segment(
            [bottom, egui::pos2(node_center.x - 14.0, node_center.y)],
            egui::Stroke::new(1.0, tree_line_color(status)),
        );
    }
    if has_parent {
        painter.circle_filled(
            egui::pos2(rect.left() + indent - 18.0, node_center.y),
            2.0,
            tree_line_color(status),
        );
    }

    painter.circle_filled(node_center, 14.0, visuals.fill);
    painter.circle_stroke(node_center, 14.0, egui::Stroke::new(1.2, visuals.stroke));
    painter.text(
        node_center,
        egui::Align2::CENTER_CENTER,
        technology_icon(spec),
        egui::FontId::proportional(18.0),
        visuals.icon_color,
    );

    let text_left = node_center.x + 23.0;
    painter.text(
        egui::pos2(text_left, rect.top() + 6.0),
        egui::Align2::LEFT_TOP,
        spec.name,
        egui::FontId::proportional(18.0),
        if status == TechnologyNodeStatus::Locked {
            war_text_muted()
        } else {
            egui::Color32::from_rgb(238, 225, 193)
        },
    );

    let progress = faction_state
        .map(|state| technology_progress(state, spec.id))
        .unwrap_or_default();
    let meta = if status == TechnologyNodeStatus::Active {
        t.text_args(
            "technology-node-active-meta",
            &args([
                ("progress", progress.to_string()),
                ("turns", spec.turns.to_string()),
                ("gold", spec.gold_cost.to_string()),
            ]),
        )
    } else {
        t.text_args(
            "technology-node-meta",
            &args([
                ("turns", spec.turns.to_string()),
                ("gold", spec.gold_cost.to_string()),
            ]),
        )
    };
    painter.text(
        egui::pos2(text_left, rect.top() + 29.0),
        egui::Align2::LEFT_TOP,
        meta,
        egui::FontId::proportional(13.0),
        war_text_muted(),
    );

    let badge_text = visuals.label;
    let badge_width = (badge_text.chars().count() as f32 * 13.0 + 18.0).clamp(54.0, 90.0);
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(rect.right() - badge_width - 8.0, rect.center().y - 13.0),
        egui::vec2(badge_width, 26.0),
    );
    painter.rect(
        badge_rect,
        4.0,
        visuals.badge_fill,
        egui::Stroke::new(1.0, visuals.stroke),
        egui::StrokeKind::Inside,
    );
    painter.text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        badge_text,
        egui::FontId::proportional(13.0),
        visuals.badge_text,
    );

    response
}

fn technology_detail(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    game: &GameState,
    t: &Translator,
) {
    let faction_id = game.player_faction_id.clone();
    let spec = technology_spec(ui_state.selected_technology_id);
    let faction_state = faction_technology_state(game, &faction_id);
    let progress = faction_state
        .map(|state| technology_progress(state, spec.id))
        .unwrap_or_default();
    let total_gold = faction_total_gold(game, &faction_id);
    let cost = effective_technology_cost(game, &faction_id, spec.id);
    let missing = missing_prerequisite_names(faction_state, spec.id);
    let is_completed = faction_state.is_some_and(|state| state.completed.contains(&spec.id));
    let is_funded = faction_state.is_some_and(|state| state.funded.contains(&spec.id));
    let is_active = faction_state.is_some_and(|state| state.active == Some(spec.id));

    ui.heading(egui::RichText::new(spec.name).color(war_gold()));
    ui.label(t.text_args(
        "technology-detail-meta",
        &args([
            ("branch", technology_branch_label(t, spec.branch)),
            ("turns", spec.turns.to_string()),
            ("gold", cost.to_string()),
        ]),
    ));
    if cost != spec.gold_cost {
        ui.colored_label(
            war_success(),
            t.text_args(
                "technology-original-cost",
                &args([("gold", spec.gold_cost.to_string())]),
            ),
        );
    }
    ui.label(t.text_args(
        "technology-current-gold",
        &args([("gold", total_gold.to_string())]),
    ));
    ui.label(t.text_args(
        "technology-progress",
        &args([
            ("progress", progress.to_string()),
            ("turns", spec.turns.to_string()),
        ]),
    ));
    ui.separator();

    if spec.prerequisites.is_empty() {
        ui.label(t.text("technology-prerequisite-none"));
    } else if missing.is_empty() {
        ui.colored_label(
            war_success(),
            t.text_args(
                "technology-prerequisites",
                &args([("names", prerequisite_names(spec).join("、"))]),
            ),
        );
    } else {
        ui.colored_label(
            war_warning(),
            t.text_args(
                "technology-missing-prerequisites",
                &args([("names", missing.join("、"))]),
            ),
        );
    }
    ui.add_space(6.0);
    ui.label(t.text("technology-effect"));
    ui.colored_label(war_text_muted(), spec.effect);
    ui.separator();

    if is_completed {
        ui.add_enabled(false, egui::Button::new(t.text("technology-completed")));
        return;
    }
    if is_active {
        ui.add_enabled(
            false,
            egui::Button::new(t.text_args(
                "technology-active-progress",
                &args([
                    ("progress", progress.to_string()),
                    ("turns", spec.turns.to_string()),
                ]),
            )),
        );
        return;
    }
    if !missing.is_empty() {
        ui.add_enabled(
            false,
            egui::Button::new(t.text("technology-prerequisite-locked")),
        );
        return;
    }
    if is_funded {
        if ui.button(t.text("technology-continue-research")).clicked() {
            start_player_research(ui_state, spec.id, t);
        }
        return;
    }
    if total_gold < cost {
        ui.colored_label(
            war_danger(),
            t.text_args(
                "technology-gold-shortfall",
                &args([("gold", (cost - total_gold).to_string())]),
            ),
        );
        ui.add_enabled(
            false,
            egui::Button::new(t.text("technology-insufficient-gold")),
        );
        return;
    }
    if ui.button(t.text("technology-start-research")).clicked() {
        start_player_research(ui_state, spec.id, t);
    }
}

fn start_player_research(ui_state: &mut GameUiState, technology_id: TechnologyId, t: &Translator) {
    let Some(game) = ui_state.game.as_mut() else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    let faction_id = game.player_faction_id.clone();
    let spec = technology_spec(technology_id);
    match start_research(game, &faction_id, technology_id) {
        Ok(outcome) if outcome.resumed => {
            ui_state.message = t.text_args(
                "message-research-resumed",
                &args([("name", spec.name.to_string())]),
            );
        }
        Ok(outcome) => {
            ui_state.message = t.text_args(
                "message-research-started",
                &args([
                    ("name", spec.name.to_string()),
                    ("gold", outcome.cost_paid.to_string()),
                ]),
            );
        }
        Err(error) => ui_state.message = error.to_string(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TechnologyNodeStatus {
    Completed,
    Active,
    Funded,
    Available,
    Unaffordable,
    Locked,
}

#[derive(Clone, Debug)]
struct TechnologyNodeVisuals {
    label: String,
    fill: egui::Color32,
    stroke: egui::Color32,
    icon_color: egui::Color32,
    badge_fill: egui::Color32,
    badge_text: egui::Color32,
}

fn technology_node_status(
    game: &GameState,
    faction_state: Option<&FactionTechnologyState>,
    spec: &TechnologySpec,
) -> TechnologyNodeStatus {
    if faction_state.is_some_and(|state| state.completed.contains(&spec.id)) {
        return TechnologyNodeStatus::Completed;
    }
    if faction_state.is_some_and(|state| state.active == Some(spec.id)) {
        return TechnologyNodeStatus::Active;
    }
    if faction_state.is_some_and(|state| state.funded.contains(&spec.id)) {
        return TechnologyNodeStatus::Funded;
    }
    if !missing_prerequisite_names(faction_state, spec.id).is_empty() {
        return TechnologyNodeStatus::Locked;
    }
    if faction_total_gold(game, &game.player_faction_id)
        >= effective_technology_cost(game, &game.player_faction_id, spec.id)
    {
        TechnologyNodeStatus::Available
    } else {
        TechnologyNodeStatus::Unaffordable
    }
}

fn technology_status_visuals(
    status: TechnologyNodeStatus,
    t: &Translator,
) -> TechnologyNodeVisuals {
    match status {
        TechnologyNodeStatus::Completed => TechnologyNodeVisuals {
            label: t.text("technology-completed"),
            fill: egui::Color32::from_rgba_unmultiplied(39, 86, 51, 220),
            stroke: war_success(),
            icon_color: egui::Color32::from_rgb(226, 244, 218),
            badge_fill: egui::Color32::from_rgba_unmultiplied(45, 92, 55, 180),
            badge_text: egui::Color32::from_rgb(226, 244, 218),
        },
        TechnologyNodeStatus::Active => TechnologyNodeVisuals {
            label: t.text("technology-active"),
            fill: egui::Color32::from_rgba_unmultiplied(122, 59, 39, 230),
            stroke: war_gold(),
            icon_color: egui::Color32::from_rgb(255, 235, 180),
            badge_fill: egui::Color32::from_rgba_unmultiplied(117, 65, 35, 200),
            badge_text: egui::Color32::from_rgb(255, 236, 190),
        },
        TechnologyNodeStatus::Funded => TechnologyNodeVisuals {
            label: t.text("technology-funded"),
            fill: egui::Color32::from_rgba_unmultiplied(72, 65, 39, 220),
            stroke: war_warning(),
            icon_color: egui::Color32::from_rgb(245, 216, 145),
            badge_fill: egui::Color32::from_rgba_unmultiplied(87, 67, 34, 180),
            badge_text: egui::Color32::from_rgb(246, 222, 160),
        },
        TechnologyNodeStatus::Available => TechnologyNodeVisuals {
            label: t.text("technology-available"),
            fill: egui::Color32::from_rgba_unmultiplied(54, 48, 34, 220),
            stroke: war_gold(),
            icon_color: war_gold(),
            badge_fill: egui::Color32::from_rgba_unmultiplied(66, 50, 28, 170),
            badge_text: egui::Color32::from_rgb(247, 224, 173),
        },
        TechnologyNodeStatus::Unaffordable => TechnologyNodeVisuals {
            label: t.text("technology-unaffordable"),
            fill: egui::Color32::from_rgba_unmultiplied(44, 39, 32, 190),
            stroke: war_warning(),
            icon_color: war_warning(),
            badge_fill: egui::Color32::from_rgba_unmultiplied(61, 45, 28, 150),
            badge_text: egui::Color32::from_rgb(236, 196, 122),
        },
        TechnologyNodeStatus::Locked => TechnologyNodeVisuals {
            label: t.text("technology-locked"),
            fill: egui::Color32::from_rgba_unmultiplied(34, 31, 27, 170),
            stroke: egui::Color32::from_rgba_unmultiplied(118, 105, 81, 150),
            icon_color: war_text_muted(),
            badge_fill: egui::Color32::from_rgba_unmultiplied(42, 37, 31, 140),
            badge_text: war_text_muted(),
        },
    }
}

fn tree_line_color(status: TechnologyNodeStatus) -> egui::Color32 {
    match status {
        TechnologyNodeStatus::Completed => {
            egui::Color32::from_rgba_unmultiplied(118, 186, 122, 150)
        }
        TechnologyNodeStatus::Active | TechnologyNodeStatus::Available => {
            egui::Color32::from_rgba_unmultiplied(215, 162, 72, 155)
        }
        TechnologyNodeStatus::Funded | TechnologyNodeStatus::Unaffordable => {
            egui::Color32::from_rgba_unmultiplied(218, 174, 88, 130)
        }
        TechnologyNodeStatus::Locked => egui::Color32::from_rgba_unmultiplied(118, 105, 81, 92),
    }
}

fn technology_depth(spec: &TechnologySpec) -> usize {
    spec.prerequisites
        .iter()
        .map(|id| technology_depth(technology_spec(*id)) + 1)
        .max()
        .unwrap_or_default()
}

fn technology_icon(spec: &TechnologySpec) -> &'static str {
    match spec.id {
        TechnologyId::MilitiaDrill => egui_phosphor::regular::USERS,
        TechnologyId::ArsenalLogistics => egui_phosphor::regular::STACK,
        TechnologyId::ScoutRoads => egui_phosphor::regular::MAP_TRIFOLD,
        TechnologyId::IronWeapons => egui_phosphor::regular::SWORD,
        TechnologyId::StrictDiscipline => egui_phosphor::regular::FLAG,
        TechnologyId::FortifiedGarrisons => egui_phosphor::regular::SHIELD,
        TechnologyId::SupplyEscort => egui_phosphor::regular::BARN,
        TechnologyId::CombinedArms => egui_phosphor::regular::ARROWS_OUT_CARDINAL,
        TechnologyId::GateFireTactics => egui_phosphor::regular::FIRE,
        TechnologyId::RotatingDefense => egui_phosphor::regular::ARROWS_CLOCKWISE,
        TechnologyId::MilitaryGranaries => egui_phosphor::regular::WAREHOUSE,
        TechnologyId::SiegeEngines => egui_phosphor::regular::HAMMER,
        TechnologyId::OfficerMerit => egui_phosphor::regular::MEDAL,
        TechnologyId::GrandCommandery => egui_phosphor::regular::CROWN,
        TechnologyId::HouseholdRegisters => egui_phosphor::regular::IDENTIFICATION_CARD,
        TechnologyId::IrrigationSurvey => egui_phosphor::regular::DROP,
        TechnologyId::MarketRegisters => egui_phosphor::regular::COINS,
        TechnologyId::GranarySystem => egui_phosphor::regular::BARN,
        TechnologyId::PriceStabilization => egui_phosphor::regular::SCALES,
        TechnologyId::ArtisanRegisters => egui_phosphor::regular::HAMMER,
        TechnologyId::CanalRestoration => egui_phosphor::regular::WAVES,
        TechnologyId::TradePasses => egui_phosphor::regular::BRIDGE,
        TechnologyId::BureaucraticRecords => egui_phosphor::regular::SCROLL,
        TechnologyId::EverNormalGranary => egui_phosphor::regular::WAREHOUSE,
        TechnologyId::WorkshopGuilds => egui_phosphor::regular::GEAR,
        TechnologyId::CommanderyReviews => egui_phosphor::regular::CLIPBOARD_TEXT,
        TechnologyId::CanalTaxation => egui_phosphor::regular::BANK,
        TechnologyId::MinistryOfFinance => egui_phosphor::regular::SEAL_CHECK,
    }
}

fn prerequisite_names(spec: &TechnologySpec) -> Vec<&'static str> {
    spec.prerequisites
        .iter()
        .map(|id| technology_spec(*id).name)
        .collect()
}

pub(super) fn city_list_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.city_list_open {
        return;
    }

    egui::Area::new(egui::Id::new("hud_city_list_scrim"))
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
                ui_state.city_list_open = false;
            }
        });

    let modal_width = (screen.width() * 0.64)
        .clamp(520.0, 860.0)
        .min((screen.width() - HUD_MARGIN * 2.0).max(360.0));
    let modal_height = (screen.height() * 0.72)
        .clamp(420.0, 640.0)
        .min((screen.height() - HUD_MARGIN * 2.0).max(320.0));
    egui::Area::new(egui::Id::new("hud_city_list"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                ui.set_min_height(modal_height);
                if modal_title_bar(ui, t, &t.text("city-list-title")) {
                    ui_state.city_list_open = false;
                }
                ui.separator();
                city_list(ui, ui_state, t, modal_width, modal_height);
            });
        });
}

pub(super) fn save_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    _screen: egui::Rect,
) {
    if !ui_state.save_panel_open {
        return;
    }
    egui::Area::new(egui::Id::new("hud_save_panel"))
        .order(egui::Order::Foreground)
        .anchor(
            egui::Align2::RIGHT_TOP,
            egui::vec2(-HUD_MARGIN, HUD_TOP_OFFSET + HUD_TOP_HEIGHT + 14.0),
        )
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(330.0);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new(t.text("save-title")).color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if collapse_icon_button(ui, t, true).clicked() {
                            ui_state.save_panel_open = false;
                        }
                    });
                });
                save_controls(ui, ui_state, t);
            });
        });
}

pub(super) fn city_drawer_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.city_drawer_open {
        return;
    }
    let modal_width = (screen.width() - HUD_MARGIN * 2.0).clamp(760.0, 1120.0);
    let modal_height = (screen.height() - HUD_MARGIN * 2.0).clamp(520.0, 760.0);
    egui::Area::new(egui::Id::new("hud_city_drawer"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                ui.set_min_height(modal_height);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new(t.text("command-tent-title")).color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if collapse_icon_button(ui, t, true).clicked() {
                            ui_state.city_drawer_open = false;
                        }
                    });
                });
                ui.separator();
                ui.allocate_ui_with_layout(
                    egui::vec2(modal_width, modal_height - 54.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        selected_city_panel(ui, ui_state, t);
                    },
                );
            });
        });
}

pub(super) fn report_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    let width = screen.width() * 0.5;
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
                    ui.heading(egui::RichText::new(t.text("report-title")).color(war_gold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if collapse_icon_button(ui, t, ui_state.reports_open).clicked() {
                            ui_state.reports_open = !ui_state.reports_open;
                        }
                    });
                });
                if ui_state.reports_open {
                    ui.separator();
                    report_panel(ui, ui_state, t, screen);
                } else if !ui_state.message.is_empty() {
                    ui.label(&ui_state.message);
                }
            });
        });
}

pub(super) fn officer_browser_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.officer_browser_open {
        return;
    }

    let width = modal_content_width(screen, OFFICER_BROWSER_MODAL_WIDTH);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_officer_browser"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("officer-browser-title")) {
                    ui_state.officer_browser_open = false;
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.officer_browser_filters,
                        "hud_officer_browser_filters",
                        t,
                    );
                } else {
                    ui.label(t.text("message-no-game-state"));
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    let response = officer_browser_table(
                        ui,
                        game,
                        &ui_state.officer_browser_filters,
                        OfficerBrowserTableOptions {
                            max_height: height - 118.0,
                            id_salt: "hud_officer_browser_table",
                            selected_officer_id: ui_state.officer_browser_selected_id.as_deref(),
                            editable: false,
                            retainer_faction_id: None,
                        },
                        t,
                    );
                    if let Some(officer_id) = response.selected_officer_id {
                        ui_state.officer_browser_selected_id = Some(officer_id);
                    }
                    if let Some(officer_id) = response.view_officer_id {
                        ui_state.officer_detail_id = Some(officer_id);
                    }
                }
            });
        });
}

pub(super) fn retainer_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.retainers_open {
        return;
    }

    let width = modal_content_width(screen, OFFICER_BROWSER_MODAL_WIDTH);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_retainers"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("retainer-title")) {
                    ui_state.retainers_open = false;
                }
                ui.separator();
                if let Some(game) = &ui_state.game {
                    officer_browser_filters(
                        ui,
                        game,
                        &mut ui_state.retainer_filters,
                        "hud_retainer_filters",
                        t,
                    );
                } else {
                    ui.label(t.text("message-no-game-state"));
                }
                ui.separator();
                let Some(game) = ui_state.game.as_ref() else {
                    return;
                };
                let player_faction_id = game.player_faction_id.clone();
                let response = officer_browser_table(
                    ui,
                    game,
                    &ui_state.retainer_filters,
                    OfficerBrowserTableOptions {
                        max_height: height - 142.0,
                        id_salt: "hud_retainer_table",
                        selected_officer_id: ui_state.retainer_selected_id.as_deref(),
                        editable: false,
                        retainer_faction_id: Some(player_faction_id.as_str()),
                    },
                    t,
                );
                if let Some(officer_id) = response.selected_officer_id.clone() {
                    ui_state.retainer_selected_id = Some(officer_id);
                }
                if let Some(officer_id) = response.view_officer_id {
                    ui_state.officer_detail_id = Some(officer_id);
                }

                if response.appoint_officer_id.is_some() || response.dismiss_officer_id.is_some() {
                    let Some(game) = ui_state.game.as_mut() else {
                        return;
                    };
                    if let Some((officer_id, office_id)) =
                        response.appoint_officer_id.zip(response.appoint_office_id)
                    {
                        match appoint_official_post(
                            game,
                            &player_faction_id,
                            &officer_id,
                            &office_id,
                        ) {
                            Ok(()) => {
                                let (officer_name, loyalty) = game
                                    .officers
                                    .get(&officer_id)
                                    .map(|officer| (officer.name.clone(), officer.loyalty))
                                    .unwrap_or((officer_id, 0));
                                let office_name = official_post_spec(&office_id)
                                    .map(|spec| spec.name)
                                    .unwrap_or(office_id.as_str());
                                ui_state.message = t.text_args(
                                    "message-officer-appointed",
                                    &args([
                                        ("officer", officer_name),
                                        ("office", office_name.to_string()),
                                        ("loyalty", loyalty.to_string()),
                                    ]),
                                );
                            }
                            Err(error) => ui_state.message = error.to_string(),
                        }
                    } else if let Some(officer_id) = response.dismiss_officer_id {
                        match dismiss_official_post(game, &player_faction_id, &officer_id) {
                            Ok(()) => {
                                let (officer_name, loyalty) = game
                                    .officers
                                    .get(&officer_id)
                                    .map(|officer| (officer.name.clone(), officer.loyalty))
                                    .unwrap_or((officer_id, 0));
                                ui_state.message = t.text_args(
                                    "message-officer-dismissed",
                                    &args([
                                        ("officer", officer_name),
                                        ("loyalty", loyalty.to_string()),
                                    ]),
                                );
                            }
                            Err(error) => ui_state.message = error.to_string(),
                        }
                    }
                }
            });
        });
}

pub(super) fn officer_detail_modal(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
    async_runtime: &CoreAsyncRuntime,
) {
    let officer_detail_id = ui_state.officer_detail_id.clone();
    let api_key = ui_state.applied_settings.ai.multimodal.api_key.clone();
    let model_name = ui_state.applied_settings.ai.multimodal.model_name.clone();
    let close_requested = match ui_state.game.as_ref() {
        Some(game) => officer_detail_modal_for_game(
            ctx,
            officer_detail_id.as_deref(),
            OfficerPortraitModalContext {
                store: &mut ui_state.officer_portraits,
                api_key: &api_key,
                model_name: &model_name,
                async_runtime,
            },
            t,
            screen,
            game,
        ),
        None => ui_state.officer_detail_id.is_some(),
    };
    if close_requested {
        ui_state.officer_detail_id = None;
    }
}

pub(super) struct OfficerPortraitModalContext<'a> {
    pub(super) store: &'a mut OfficerPortraitStore,
    pub(super) api_key: &'a str,
    pub(super) model_name: &'a str,
    pub(super) async_runtime: &'a CoreAsyncRuntime,
}

pub(super) fn officer_detail_modal_for_game(
    ctx: &egui::Context,
    officer_detail_id: Option<&str>,
    mut portrait_context: OfficerPortraitModalContext<'_>,
    t: &Translator,
    screen: egui::Rect,
    game: &GameState,
) -> bool {
    let Some(officer_id) = officer_detail_id else {
        return false;
    };
    let mut close_requested = false;

    if !game.officers.contains_key(officer_id) {
        return true;
    } else {
        let width = (screen.width() * 0.78).clamp(740.0, 980.0);
        let height = (screen.height() * 0.78).clamp(500.0, 720.0);
        egui::Area::new(egui::Id::new("officer_detail_modal"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                war_panel_frame().show(ui, |ui| {
                    ui.set_width(width);
                    ui.set_min_height(height);
                    let Some(officer) = game.officers.get(officer_id) else {
                        close_requested = true;
                        return;
                    };
                    let title = t.text_args(
                        "officer-detail-title",
                        &args([("officer", officer.name.clone())]),
                    );
                    if modal_title_bar(ui, t, &title) {
                        close_requested = true;
                        return;
                    }
                    ui.separator();
                    officer_detail_header(ui, game, officer, t);
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .id_salt(("officer_detail_body", &officer.id))
                        .max_height(height - 126.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.columns(2, |columns| {
                                columns[0].set_width((width * 0.36).clamp(278.0, 340.0));
                                officer_detail_portrait_section(
                                    ctx,
                                    &mut columns[0],
                                    officer,
                                    &mut portrait_context,
                                    t,
                                );
                                columns[0].add_space(8.0);
                                officer_detail_status_section(&mut columns[0], game, officer, t);
                                columns[0].add_space(8.0);
                                officer_detail_stats_section(&mut columns[0], officer, t);

                                officer_detail_relationship_section(
                                    &mut columns[1],
                                    game,
                                    officer,
                                    t,
                                );
                                columns[1].add_space(8.0);
                                officer_detail_history_section(&mut columns[1], game, officer, t);
                            });
                        });
                });
            });
    }
    close_requested
}

fn officer_detail_header(ui: &mut egui::Ui, game: &GameState, officer: &Officer, t: &Translator) {
    let profile = officer.profile.as_ref();
    let courtesy = profile
        .and_then(|profile| profile.courtesy_name.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| t.text("none"));
    let native_place = profile
        .and_then(|profile| profile.native_place.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| t.text("unknown"));
    let life = officer_life_span(officer, t);
    let faction = faction_name(game, &officer.faction_id);
    let city = officer_city_detail_name(game, officer, t);

    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(&officer.name)
                .size(24.0)
                .color(war_gold())
                .strong(),
        );
        ui.separator();
        ui.label(t.text_args(
            "officer-detail-header-courtesy",
            &args([("courtesy", courtesy)]),
        ));
        ui.separator();
        ui.label(t.text_args(
            "officer-detail-header-origin",
            &args([
                ("gender", officer_gender_label(t, &officer.gender)),
                ("native_place", native_place),
                ("life", life),
            ]),
        ));
        ui.separator();
        ui.label(t.text_args(
            "officer-detail-header-posting",
            &args([("faction", faction), ("city", city)]),
        ));
    });
}

fn officer_detail_portrait_section(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    officer: &Officer,
    portrait_context: &mut OfficerPortraitModalContext<'_>,
    t: &Translator,
) {
    let draft = OfficerEditDraft::from_officer(officer);
    let path = officer_portrait_path(&draft.id);
    let has_portrait = path.as_ref().is_ok_and(|path| path.is_file());
    let task_state = portrait_context.store.task_state(&draft.id);
    let generating = matches!(task_state, OfficerPortraitTaskState::Generating);
    let mut load_error = None;
    let texture = match &path {
        Ok(path) => match portrait_context.store.texture_for(ctx, &draft.id, path) {
            Ok(texture) => texture,
            Err(error) => {
                load_error = Some(error);
                None
            }
        },
        Err(error) => {
            load_error = Some(error.clone());
            None
        }
    };

    war_sub_panel_frame().show(ui, |ui| {
        officer_detail_section_title(ui, &t.text("officer-portrait-title"));
        let preview_width = ui
            .available_width()
            .clamp(0.0, OFFICER_DETAIL_PORTRAIT_WIDTH);
        let preview_height =
            preview_width * OFFICER_PORTRAIT_ASPECT_HEIGHT / OFFICER_PORTRAIT_ASPECT_WIDTH;
        let preview_size = egui::vec2(preview_width, preview_height);
        ui.horizontal(|ui| {
            ui.add_space(((ui.available_width() - preview_width) * 0.5).max(0.0));
            let (rect, _) = ui.allocate_exact_size(preview_size, egui::Sense::hover());
            paint_officer_portrait_preview(ui, rect, texture, generating, t);
        });

        ui.add_space(8.0);
        officer_portrait_status_line(ui, t, &task_state, has_portrait, load_error.as_deref());
        ui.add_space(6.0);

        let button_text = if generating {
            t.text("officer-portrait-generating")
        } else if has_portrait {
            t.text("officer-portrait-regenerate")
        } else {
            t.text("officer-portrait-generate")
        };
        let clicked = ui
            .add_enabled(
                !generating,
                egui::Button::new(button_text).min_size(egui::vec2(preview_width, 34.0)),
            )
            .clicked();
        if clicked {
            portrait_context.store.start_generation(
                portrait_context.async_runtime,
                draft,
                portrait_context.api_key.to_string(),
                portrait_context.model_name.to_string(),
                t.text("officer-portrait-api-key-required"),
            );
        }
    });
}

fn officer_detail_status_section(
    ui: &mut egui::Ui,
    game: &GameState,
    officer: &Officer,
    t: &Translator,
) {
    war_sub_panel_frame().show(ui, |ui| {
        officer_detail_section_title(ui, &t.text("officer-detail-section-status"));
        detail_kv(
            ui,
            &t.text("officer-column-status"),
            officer_status_label(&officer.status, t),
        );
        detail_kv(
            ui,
            &t.text("officer-column-age"),
            officer.age_at(game.year).to_string(),
        );
        detail_kv(
            ui,
            &t.text("officer-column-loyalty"),
            officer.loyalty.to_string(),
        );
        detail_kv(
            ui,
            &t.text("officer-column-office"),
            officer_office_detail_name(officer, t),
        );
        detail_kv(
            ui,
            &t.text("officer-column-salary"),
            officer_monthly_salary(officer).to_string(),
        );
        if let Some(profile) = &officer.profile {
            detail_kv(
                ui,
                &t.text("officer-detail-confidence"),
                confidence_label(t, &profile.confidence),
            );
        } else {
            detail_kv(ui, &t.text("officer-detail-confidence"), t.text("unknown"));
        }
    });
}

fn officer_detail_stats_section(ui: &mut egui::Ui, officer: &Officer, t: &Translator) {
    war_sub_panel_frame().show(ui, |ui| {
        officer_detail_section_title(ui, &t.text("officer-detail-section-stats"));
        ability_bar(ui, t.text("stat-leadership"), officer.stats.leadership);
        ability_bar(ui, t.text("stat-strength"), officer.stats.strength);
        ability_bar(ui, t.text("stat-intelligence"), officer.stats.intelligence);
        ability_bar(ui, t.text("stat-politics"), officer.stats.politics);
        ability_bar(ui, t.text("stat-charm"), officer.stats.charm);
    });
}

fn officer_detail_relationship_section(
    ui: &mut egui::Ui,
    game: &GameState,
    officer: &Officer,
    t: &Translator,
) {
    war_sub_panel_frame().show(ui, |ui| {
        officer_detail_section_title(ui, &t.text("officer-detail-section-relationships"));
        let graph = officer_relationship_graph(game, officer, t);
        if graph.edges.is_empty() {
            ui.colored_label(war_text_muted(), t.text("officer-detail-no-relationships"));
        } else {
            relationship_graph(ui, &graph, t);
        }
    });
}

fn officer_detail_history_section(
    ui: &mut egui::Ui,
    game: &GameState,
    officer: &Officer,
    t: &Translator,
) {
    war_sub_panel_frame().show(ui, |ui| {
        officer_detail_section_title(ui, &t.text("officer-detail-section-history"));
        if let Some(profile) = &officer.profile {
            if !profile.tags.is_empty() {
                let tags = profile
                    .tags
                    .iter()
                    .map(|tag_id| officer_tag_display_text(game, tag_id, t))
                    .collect::<Vec<_>>()
                    .join(", ");
                detail_kv(ui, &t.text("officer-detail-tags"), tags);
            }
            if !profile.biography.is_empty() {
                ui.label(egui::RichText::new(t.text("officer-biography")).color(war_text_muted()));
                egui::ScrollArea::vertical()
                    .id_salt(("officer_detail_bio", &profile.id))
                    .max_height(160.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.label(&profile.biography);
                    });
            }
            if !profile.notes.is_empty() {
                ui.separator();
                ui.label(
                    egui::RichText::new(t.text("officer-detail-notes")).color(war_text_muted()),
                );
                ui.label(&profile.notes);
            }
            if profile.tags.is_empty() && profile.biography.is_empty() && profile.notes.is_empty() {
                ui.colored_label(war_text_muted(), t.text("officer-detail-no-history"));
            }
        } else {
            ui.colored_label(war_text_muted(), t.text("officer-detail-no-history"));
        }
    });
}

fn officer_detail_section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(egui::RichText::new(title).color(war_gold()).strong());
    ui.add_space(4.0);
}

fn detail_kv(ui: &mut egui::Ui, key: &str, value: String) {
    ui.horizontal_wrapped(|ui| {
        ui.add_sized(
            [88.0, 20.0],
            egui::Label::new(egui::RichText::new(key).color(war_text_muted())),
        );
        ui.label(value);
    });
}

fn ability_bar(ui: &mut egui::Ui, label: String, value: u8) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [54.0, 18.0],
            egui::Label::new(egui::RichText::new(label).color(war_text_muted())),
        );
        ui.add_sized(
            [132.0, 14.0],
            egui::ProgressBar::new(f32::from(value) / 100.0)
                .fill(ability_color(value))
                .text(value.to_string()),
        );
    });
}

fn ability_color(value: u8) -> egui::Color32 {
    match value {
        85..=u8::MAX => war_success(),
        65..=84 => war_gold(),
        45..=64 => war_warning(),
        _ => war_danger(),
    }
}

fn officer_life_span(officer: &Officer, t: &Translator) -> String {
    let profile = officer.profile.as_ref();
    let birth = profile
        .and_then(|profile| profile.birth_year)
        .or((officer.birth_year != 0).then_some(officer.birth_year))
        .map(|year| year.to_string())
        .unwrap_or_else(|| t.text("unknown"));
    let death = profile
        .and_then(|profile| profile.death_year)
        .map(|year| year.to_string())
        .unwrap_or_else(|| t.text("unknown"));
    format!("{birth}-{death}")
}

fn officer_city_detail_name(game: &GameState, officer: &Officer, t: &Translator) -> String {
    officer
        .city_id
        .as_deref()
        .and_then(|city_id| game.cities.get(city_id))
        .map(|city| city.name.clone())
        .unwrap_or_else(|| t.text("officer-city-unassigned"))
}

fn officer_office_detail_name(officer: &Officer, t: &Translator) -> String {
    officer
        .office_id
        .as_deref()
        .and_then(official_post_spec)
        .map(|spec| {
            t.text_args(
                "officer-detail-office-ranked",
                &args([
                    ("office", spec.name.to_string()),
                    ("rank", official_rank_label(spec.rank).to_string()),
                ]),
            )
        })
        .unwrap_or_else(|| t.text("none"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OfficerRelationshipGraph {
    center_name: String,
    edges: Vec<OfficerRelationshipGraphEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OfficerRelationshipGraphEdge {
    target_id: OfficerId,
    target_name: String,
    label: String,
    tooltip: String,
    kind: RelationshipGraphKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RelationshipGraphKind {
    Ruler,
    Heir,
    Governor,
    Parent,
    Child,
    Spouse,
    SwornSibling,
    Enemy,
    RulerSubject,
    Other,
}

fn officer_relationship_graph(
    game: &GameState,
    officer: &Officer,
    t: &Translator,
) -> OfficerRelationshipGraph {
    let mut graph = OfficerRelationshipGraph {
        center_name: officer.name.clone(),
        edges: Vec::new(),
    };
    if let Some(profile) = &officer.profile {
        for relationship in &profile.relationships {
            graph.edges.push(static_relationship_edge(relationship, t));
        }
    }
    graph.edges.extend(dynamic_role_edges(game, officer, t));
    graph.edges.extend(dynamic_marriage_edges(game, officer, t));
    graph.edges.extend(dynamic_family_edges(game, officer, t));
    graph
}

fn static_relationship_edge(
    relationship: &OfficerRelationship,
    t: &Translator,
) -> OfficerRelationshipGraphEdge {
    let label = officer_relationship_label(t, &relationship.kind);
    let mut tooltip_parts = vec![
        label.clone(),
        t.text_args(
            "officer-detail-relation-confidence",
            &args([("confidence", confidence_label(t, &relationship.confidence))]),
        ),
    ];
    if !relationship.source.trim().is_empty() {
        tooltip_parts.push(t.text_args(
            "officer-detail-relation-source",
            &args([("source", relationship.source.clone())]),
        ));
    }
    if !relationship.notes.trim().is_empty() {
        tooltip_parts.push(t.text_args(
            "officer-detail-relation-notes",
            &args([("notes", relationship.notes.clone())]),
        ));
    }

    OfficerRelationshipGraphEdge {
        target_id: relationship.target_id.clone(),
        target_name: relationship.target_name.clone(),
        label,
        tooltip: tooltip_parts.join("\n"),
        kind: graph_kind_for_static_relationship(&relationship.kind),
    }
}

fn graph_kind_for_static_relationship(kind: &OfficerRelationshipKind) -> RelationshipGraphKind {
    match kind {
        OfficerRelationshipKind::RulerSubject => RelationshipGraphKind::RulerSubject,
        OfficerRelationshipKind::ParentChild | OfficerRelationshipKind::AdoptiveParentChild => {
            RelationshipGraphKind::Parent
        }
        OfficerRelationshipKind::Spouse => RelationshipGraphKind::Spouse,
        OfficerRelationshipKind::Sibling => RelationshipGraphKind::Other,
        OfficerRelationshipKind::SwornSibling => RelationshipGraphKind::SwornSibling,
        OfficerRelationshipKind::Enemy => RelationshipGraphKind::Enemy,
    }
}

fn dynamic_role_edges(
    game: &GameState,
    officer: &Officer,
    t: &Translator,
) -> Vec<OfficerRelationshipGraphEdge> {
    let mut edges = Vec::new();
    for faction in game.factions.values() {
        if faction.ruler_id == officer.id {
            edges.push(dynamic_context_edge(
                format!("faction:ruler:{}", faction.id),
                faction.name.clone(),
                t.text("officer-detail-graph-ruler"),
                t.text_args(
                    "officer-detail-dynamic-ruler",
                    &args([("faction", faction.name.clone())]),
                ),
                RelationshipGraphKind::Ruler,
            ));
        }
        if faction.heir_id.as_deref() == Some(officer.id.as_str()) {
            edges.push(dynamic_context_edge(
                format!("faction:heir:{}", faction.id),
                faction.name.clone(),
                t.text("officer-detail-graph-heir"),
                t.text_args(
                    "officer-detail-dynamic-heir",
                    &args([("faction", faction.name.clone())]),
                ),
                RelationshipGraphKind::Heir,
            ));
        }
    }
    for city in game.cities.values() {
        if city.governor_id.as_deref() == Some(officer.id.as_str()) {
            edges.push(dynamic_context_edge(
                format!("city:governor:{}", city.id),
                city.name.clone(),
                t.text("officer-detail-graph-governor"),
                t.text_args(
                    "officer-detail-dynamic-governor",
                    &args([("city", city.name.clone())]),
                ),
                RelationshipGraphKind::Governor,
            ));
        }
    }
    edges
}

fn dynamic_marriage_edges(
    game: &GameState,
    officer: &Officer,
    t: &Translator,
) -> Vec<OfficerRelationshipGraphEdge> {
    game.marriages
        .iter()
        .filter(|marriage| marriage.involves(&officer.id))
        .map(|marriage| {
            let spouse_id = if marriage.husband_id == officer.id {
                &marriage.wife_id
            } else {
                &marriage.husband_id
            };
            let spouse_name = officer_display_name(game, spouse_id);
            dynamic_context_edge(
                spouse_id.clone(),
                spouse_name.clone(),
                t.text("officer-detail-graph-spouse"),
                t.text_args(
                    "officer-detail-dynamic-spouse",
                    &args([
                        ("officer", spouse_name),
                        ("year", marriage.year.to_string()),
                        ("month", marriage.month.to_string()),
                    ]),
                ),
                RelationshipGraphKind::Spouse,
            )
        })
        .collect()
}

fn dynamic_family_edges(
    game: &GameState,
    officer: &Officer,
    t: &Translator,
) -> Vec<OfficerRelationshipGraphEdge> {
    let mut edges = Vec::new();
    for relationship in &game.family_relationships {
        if relationship.child_id == officer.id {
            let parent_name = officer_display_name(game, &relationship.parent_id);
            edges.push(dynamic_context_edge(
                relationship.parent_id.clone(),
                parent_name.clone(),
                t.text("officer-detail-graph-parent"),
                t.text_args(
                    "officer-detail-dynamic-parent",
                    &args([("officer", parent_name)]),
                ),
                RelationshipGraphKind::Parent,
            ));
        }
        if relationship.parent_id == officer.id {
            let child_name = officer_display_name(game, &relationship.child_id);
            edges.push(dynamic_context_edge(
                relationship.child_id.clone(),
                child_name.clone(),
                t.text("officer-detail-graph-child"),
                t.text_args(
                    "officer-detail-dynamic-child",
                    &args([("officer", child_name)]),
                ),
                RelationshipGraphKind::Child,
            ));
        }
    }
    edges
}

fn dynamic_context_edge(
    target_id: OfficerId,
    target_name: String,
    label: String,
    tooltip: String,
    kind: RelationshipGraphKind,
) -> OfficerRelationshipGraphEdge {
    OfficerRelationshipGraphEdge {
        target_id,
        target_name,
        label,
        tooltip,
        kind,
    }
}

fn relationship_graph(ui: &mut egui::Ui, graph: &OfficerRelationshipGraph, t: &Translator) {
    let desired_size = egui::vec2(ui.available_width().max(360.0), 260.0);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(18, 15, 11, 150),
    );
    painter.rect_stroke(
        rect.shrink(1.0),
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(138, 101, 58, 95)),
        egui::StrokeKind::Inside,
    );

    let center = rect.center();
    draw_relationship_node(
        ui,
        &painter,
        center,
        34.0,
        &graph.center_name,
        war_gold(),
        t.text("officer-detail-graph-center"),
    );

    let radius_x = (rect.width() * 0.36).max(118.0);
    let radius_y = (rect.height() * 0.34).max(72.0);
    for (index, edge) in graph.edges.iter().enumerate() {
        let angle = relationship_node_angle(index, graph.edges.len());
        let pos = egui::pos2(
            center.x + angle.cos() * radius_x,
            center.y + angle.sin() * radius_y,
        );
        let color = relationship_kind_color(edge.kind);
        painter.line_segment([center, pos], egui::Stroke::new(1.6, color));
        painter.text(
            center.lerp(pos, 0.55),
            egui::Align2::CENTER_CENTER,
            &edge.label,
            egui::FontId::proportional(11.0),
            color,
        );
        draw_relationship_node(
            ui,
            &painter,
            pos,
            28.0,
            &edge.target_name,
            color,
            edge.tooltip.clone(),
        );
    }
}

fn relationship_node_angle(index: usize, count: usize) -> f32 {
    -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * index as f32 / count.max(1) as f32
}

fn draw_relationship_node(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    name: &str,
    color: egui::Color32,
    tooltip: String,
) {
    let rect = egui::Rect::from_center_size(center, egui::vec2(radius * 2.2, radius * 1.45));
    let response = ui.interact(
        rect,
        egui::Id::new(("relationship_node", name)),
        egui::Sense::hover(),
    );
    if response.hovered() {
        response.on_hover_text(tooltip);
    }
    painter.circle_filled(
        center,
        radius,
        egui::Color32::from_rgba_unmultiplied(35, 29, 22, 235),
    );
    painter.circle_stroke(center, radius, egui::Stroke::new(1.4, color));
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        compact_node_label(name),
        egui::FontId::proportional(13.0),
        war_text(),
    );
}

fn compact_node_label(name: &str) -> String {
    let chars = name.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        name.to_string()
    } else {
        chars.into_iter().take(4).collect()
    }
}

fn relationship_kind_color(kind: RelationshipGraphKind) -> egui::Color32 {
    match kind {
        RelationshipGraphKind::Spouse => egui::Color32::from_rgb(217, 126, 118),
        RelationshipGraphKind::Parent | RelationshipGraphKind::Child => {
            egui::Color32::from_rgb(119, 184, 141)
        }
        RelationshipGraphKind::Ruler
        | RelationshipGraphKind::Heir
        | RelationshipGraphKind::Governor => war_gold(),
        RelationshipGraphKind::SwornSibling => egui::Color32::from_rgb(120, 178, 211),
        RelationshipGraphKind::Enemy => war_danger(),
        RelationshipGraphKind::RulerSubject => egui::Color32::from_rgb(190, 151, 88),
        RelationshipGraphKind::Other => war_text_muted(),
    }
}

fn officer_display_name(game: &GameState, officer_id: &str) -> String {
    game.officers
        .get(officer_id)
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| officer_id.to_string())
}

fn faction_name(game: &GameState, faction_id: &str) -> String {
    game.factions
        .get(faction_id)
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| faction_id.to_string())
}

pub(super) fn shrine_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.shrine_open {
        return;
    }

    let width = (screen.width() * 0.82).clamp(760.0, 1120.0);
    let height = (screen.height() * 0.76).clamp(420.0, 680.0);
    egui::Area::new(egui::Id::new("hud_shrine"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(width);
                ui.set_min_height(height);
                if modal_title_bar(ui, t, &t.text("shrine-title")) {
                    ui_state.shrine_open = false;
                }
                ui.separator();
                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (ShrineTab::Succession, t.text("shrine-tab-succession")),
                        (ShrineTab::Marriage, t.text("shrine-tab-marriage")),
                        (ShrineTab::Children, t.text("shrine-tab-children")),
                    ] {
                        if ui
                            .selectable_label(ui_state.shrine_tab == tab, label)
                            .clicked()
                        {
                            ui_state.shrine_tab = tab;
                        }
                    }
                });
                ui.separator();
                match ui_state.shrine_tab {
                    ShrineTab::Succession => shrine_succession_panel(ui, ui_state, t),
                    ShrineTab::Marriage => shrine_marriage_panel(ui, ui_state, t, height - 112.0),
                    ShrineTab::Children => shrine_children_panel(ui, ui_state, t, height - 112.0),
                }
            });
        });
}

fn shrine_succession_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let Some(game) = ui_state.game.as_ref() else {
        ui.label(t.text("message-no-game-state"));
        return;
    };
    let faction_id = game.player_faction_id.clone();
    let Some(faction) = game.factions.get(&faction_id) else {
        ui.label(t.text("unknown-faction"));
        return;
    };
    let ruler_name = game
        .officers
        .get(&faction.ruler_id)
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| faction.ruler_id.clone());
    let heir_name = faction
        .heir_id
        .as_deref()
        .and_then(|id| game.officers.get(id))
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| t.text("shrine-no-heir"));
    ui.heading(egui::RichText::new(t.text("shrine-succession-heading")).color(war_gold()));
    ui.label(t.text_args(
        "shrine-current-ruler",
        &args([("ruler", ruler_name), ("heir", heir_name)]),
    ));
    ui.add_space(8.0);

    let candidates = succession_candidate_ids(game, &faction_id, None);
    if candidates.is_empty() {
        ui.colored_label(war_warning(), t.text("shrine-no-heir-candidates"));
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("shrine_succession_candidates")
        .max_height(420.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for candidate_id in candidates {
                let Some(officer) = ui_state
                    .game
                    .as_ref()
                    .and_then(|game| game.officers.get(&candidate_id))
                else {
                    continue;
                };
                let label = t.text_args(
                    "shrine-heir-candidate",
                    &args([
                        ("name", officer.name.clone()),
                        (
                            "age",
                            officer
                                .age_at(ui_state.game.as_ref().unwrap().year)
                                .to_string(),
                        ),
                        ("loyalty", officer.loyalty.to_string()),
                    ]),
                );
                if ui.button(label).clicked()
                    && let Some(game) = ui_state.game.as_mut()
                {
                    match set_default_heir(game, &faction_id, &candidate_id) {
                        Ok(()) => {
                            let name = game.officers[&candidate_id].name.clone();
                            ui_state.message =
                                t.text_args("message-heir-set", &args([("officer", name)]));
                        }
                        Err(error) => ui_state.message = error.to_string(),
                    }
                }
            }
        });
}

fn shrine_marriage_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    max_height: f32,
) {
    let Some(game) = ui_state.game.as_ref() else {
        ui.label(t.text("message-no-game-state"));
        return;
    };
    let faction_id = game.player_faction_id.clone();
    let marriages = game.marriages.clone();
    ui.heading(egui::RichText::new(t.text("shrine-marriage-heading")).color(war_gold()));
    let mut marry_request = None;
    ui.horizontal_wrapped(|ui| {
        officer_combo_for_marriage(
            ui,
            game,
            &faction_id,
            &mut ui_state.shrine_marriage_first,
            "shrine_marriage_first",
            t.text("shrine-marriage-first"),
        );
        officer_combo_for_marriage(
            ui,
            game,
            &faction_id,
            &mut ui_state.shrine_marriage_second,
            "shrine_marriage_second",
            t.text("shrine-marriage-second"),
        );
        if ui.button(t.text("shrine-marry")).clicked()
            && let (Some(first_id), Some(second_id)) = (
                ui_state.shrine_marriage_first.clone(),
                ui_state.shrine_marriage_second.clone(),
            )
        {
            marry_request = Some((first_id, second_id));
        }
    });
    if let Some((first_id, second_id)) = marry_request
        && let Some(game) = ui_state.game.as_mut()
    {
        match marry_officers(game, &faction_id, &first_id, &second_id) {
            Ok(marriage) => {
                let husband = game.officers[&marriage.husband_id].name.clone();
                let wife = game.officers[&marriage.wife_id].name.clone();
                ui_state.message = t.text_args(
                    "message-marriage-created",
                    &args([("husband", husband), ("wife", wife)]),
                );
            }
            Err(error) => ui_state.message = error.to_string(),
        }
    }
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt("shrine_marriages")
        .max_height(max_height.max(260.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if marriages.is_empty() {
                ui.label(t.text("shrine-marriage-empty"));
            }
            let Some(game) = ui_state.game.as_ref() else {
                return;
            };
            for marriage in &marriages {
                let husband = game
                    .officers
                    .get(&marriage.husband_id)
                    .map(|officer| officer.name.clone())
                    .unwrap_or_else(|| marriage.husband_id.clone());
                let wife = game
                    .officers
                    .get(&marriage.wife_id)
                    .map(|officer| officer.name.clone())
                    .unwrap_or_else(|| marriage.wife_id.clone());
                ui.label(t.text_args(
                    "shrine-marriage-row",
                    &args([
                        ("husband", husband),
                        ("wife", wife),
                        ("year", marriage.year.to_string()),
                        ("month", marriage.month.to_string()),
                    ]),
                ));
            }
        });
}

fn officer_combo_for_marriage(
    ui: &mut egui::Ui,
    game: &GameState,
    faction_id: &str,
    selected: &mut Option<OfficerId>,
    id_salt: &'static str,
    label: String,
) {
    let selected_text = selected
        .as_deref()
        .and_then(|id| game.officers.get(id))
        .map(|officer| officer.name.clone())
        .unwrap_or(label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            for officer in game.officers.values().filter(|officer| {
                officer.faction_id == faction_id
                    && officer.is_active()
                    && officer.is_adult_at(game.year)
            }) {
                ui.selectable_value(selected, Some(officer.id.clone()), &officer.name);
            }
        });
}

fn shrine_children_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    max_height: f32,
) {
    let Some(game) = ui_state.game.as_ref() else {
        ui.label(t.text("message-no-game-state"));
        return;
    };
    ui.heading(egui::RichText::new(t.text("shrine-children-heading")).color(war_gold()));
    let child_ids = game
        .family_relationships
        .iter()
        .map(|relationship| relationship.child_id.clone())
        .collect::<BTreeSet<_>>();
    if child_ids.is_empty() {
        ui.label(t.text("shrine-children-empty"));
        return;
    }
    egui::ScrollArea::vertical()
        .id_salt("shrine_children")
        .max_height(max_height.max(260.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for child_id in child_ids {
                let Some(child) = game.officers.get(&child_id) else {
                    continue;
                };
                let parents = game
                    .family_relationships
                    .iter()
                    .filter(|relationship| relationship.child_id == child_id)
                    .filter_map(|relationship| game.officers.get(&relationship.parent_id))
                    .map(|officer| officer.name.clone())
                    .collect::<Vec<_>>()
                    .join("、");
                ui.label(t.text_args(
                    "shrine-child-row",
                    &args([
                        ("name", child.name.clone()),
                        ("age", child.age_at(game.year).to_string()),
                        ("status", officer_status_label(&child.status, t)),
                        ("parents", parents),
                    ]),
                ));
            }
        });
}

pub(super) fn selected_city_summary(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    let summary = ui_state.game.as_ref().and_then(|game| {
        let city = game.cities.get(ui_state.selected_city_id.as_deref()?)?;
        let faction_name = game
            .factions
            .get(&city.faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| t.text("unknown"));
        Some((city.id.clone(), city.clone(), faction_name))
    });

    let Some((city_id, city, faction_name)) = summary else {
        ui.label(t.text("selected-city-none"));
        return;
    };

    city_summary_intel(ui, &city, &faction_name, t);
    ui.add_space(8.0);
    if ui.button(t.text("open-command-tent")).clicked() {
        open_city(ui_state, city_id);
    }
}

pub(super) fn map_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal(|ui| {
        ui.heading(egui::RichText::new(t.text("map-title")).color(war_gold()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_enabled(
                ui_state.map_boundaries.is_some(),
                egui::Checkbox::new(
                    &mut ui_state.map_boundaries_enabled,
                    t.text("map-boundaries"),
                ),
            );
        });
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if ui.button("-").clicked() {
            zoom_map(ui_state, 1.0 / MAP_ZOOM_STEP, None, None);
        }
        ui.add_sized(
            [58.0, 28.0],
            egui::Label::new(format!("{:.0}%", ui_state.map_zoom * 100.0)),
        );
        if ui.button("+").clicked() {
            zoom_map(ui_state, MAP_ZOOM_STEP, None, None);
        }
        if ui.button(t.text("common-reset")).clicked() {
            reset_map_view(ui_state);
        }
    });
    if ui_state.map_boundaries.is_none() {
        ui.colored_label(war_text_muted(), t.text("map-boundary-asset-missing"));
    }
}

pub(super) fn city_list(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    width: f32,
    height: f32,
) {
    let Some(game) = &ui_state.game else {
        return;
    };
    let mut rows: Vec<_> = game
        .cities
        .values()
        .map(|city| {
            let faction = game.factions.get(&city.faction_id);
            let faction_name = faction
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| t.text("unknown"));
            let color = faction.map(faction_color).unwrap_or_else(war_border);
            (
                city.id.clone(),
                city.name.clone(),
                faction_name,
                city.gold,
                city.food,
                city.troops.total(),
                color,
                city.faction_id == game.player_faction_id,
            )
        })
        .collect();
    rows.sort_by(|a, b| a.1.cmp(&b.1));

    let body_height = (height - 70.0).max(300.0);
    let content_width = ui.available_width().min(width);
    let panel_gap = 12.0;
    let panel_inner_margin_x = 20.0;
    let panel_inner_margin_y = 16.0;
    let list_outer_width = (content_width * 0.42).clamp(260.0, 380.0).max(220.0);
    let preview_outer_width = (content_width - list_outer_width - panel_gap).max(0.0);
    let list_width = (list_outer_width - panel_inner_margin_x).max(220.0);
    let preview_width = (preview_outer_width - panel_inner_margin_x).max(180.0);
    let panel_inner_height = (body_height - panel_inner_margin_y).max(260.0);

    StripBuilder::new(ui)
        .size(Size::exact(list_outer_width))
        .size(Size::exact(panel_gap))
        .size(Size::exact(preview_outer_width))
        .cell_layout(egui::Layout::top_down(egui::Align::Min))
        .clip(true)
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.set_height(body_height);
                war_sub_panel_frame().show(ui, |ui| {
                    ui.set_width(list_width);
                    egui::ScrollArea::vertical()
                        .id_salt("city_list")
                        .max_height(panel_inner_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (
                                city_id,
                                city_name,
                                faction_name,
                                gold,
                                food,
                                troops,
                                color,
                                player_owned,
                            ) in rows
                            {
                                let selected =
                                    ui_state.selected_city_id.as_deref() == Some(city_id.as_str());
                                let resources = t.text_args(
                                    "city-list-row-resources",
                                    &args([
                                        ("gold", gold.to_string()),
                                        ("food", food.to_string()),
                                        ("troops", troops.to_string()),
                                    ]),
                                );
                                let response = city_list_row(
                                    ui,
                                    selected,
                                    &city_name,
                                    &faction_name,
                                    &resources,
                                    color,
                                    player_owned,
                                );

                                if response.clicked() {
                                    ui_state.selected_city_id = Some(city_id.clone());
                                }
                                if response.double_clicked() {
                                    open_city(ui_state, city_id.clone());
                                    ui_state.city_list_open = false;
                                }
                            }
                        });
                });
            });

            let preview = ui_state.game.as_ref().and_then(|game| {
                let city = game.cities.get(ui_state.selected_city_id.as_deref()?)?;
                let faction_name = game
                    .factions
                    .get(&city.faction_id)
                    .map(|faction| faction.name.clone())
                    .unwrap_or_else(|| t.text("unknown"));
                Some((city.id.clone(), city.clone(), faction_name))
            });

            strip.empty();

            strip.cell(|ui| {
                ui.set_height(body_height);
                war_sub_panel_frame().show(ui, |ui| {
                    ui.set_width(preview_width);
                    ui.set_min_height(panel_inner_height);
                    if let Some((city_id, city, faction_name)) = preview {
                        city_summary_intel(ui, &city, &faction_name, t);
                        ui.add_space(10.0);
                        if ui
                            .add_sized(
                                [preview_width, 30.0],
                                egui::Button::new(t.text("open-command-tent")),
                            )
                            .clicked()
                        {
                            open_city(ui_state, city_id);
                            ui_state.city_list_open = false;
                        }
                    } else {
                        ui.label(t.text("selected-city-none"));
                    }
                });
            });
        });
}

fn city_list_row(
    ui: &mut egui::Ui,
    selected: bool,
    city_name: &str,
    faction_name: &str,
    resources: &str,
    color: egui::Color32,
    player_owned: bool,
) -> egui::Response {
    let row_size = egui::vec2(ui.available_width(), 64.0);
    let (rect, response) = ui.allocate_exact_size(row_size, egui::Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }

    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(113, 80, 42, 220)
    } else if response.hovered() {
        egui::Color32::from_rgba_unmultiplied(52, 42, 29, 220)
    } else {
        egui::Color32::from_rgba_unmultiplied(24, 21, 16, 96)
    };
    let stroke = egui::Stroke::new(
        if selected { 1.6 } else { 1.0 },
        if selected {
            war_gold()
        } else {
            color.gamma_multiply(0.65)
        },
    );

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect(
        rect.shrink(1.0),
        5.0,
        fill,
        stroke,
        egui::StrokeKind::Inside,
    );

    let icon_center = egui::pos2(rect.left() + 31.0, rect.center().y + 1.0);
    draw_city_marker_icon(
        &painter,
        icon_center,
        0.42,
        0.42,
        color,
        selected,
        player_owned,
    );

    let text_x = rect.left() + 62.0;
    painter.text(
        egui::pos2(text_x, rect.top() + 8.0),
        egui::Align2::LEFT_TOP,
        city_name,
        egui::FontId::proportional(18.0),
        if selected { war_gold() } else { war_text() },
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 31.0),
        egui::Align2::LEFT_TOP,
        faction_name,
        egui::FontId::proportional(13.0),
        war_text_muted(),
    );
    painter.text(
        egui::pos2(text_x, rect.top() + 47.0),
        egui::Align2::LEFT_TOP,
        resources,
        egui::FontId::proportional(12.0),
        war_text_muted(),
    );

    response
}

pub(super) fn officer_browser_filters(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &mut OfficerBrowserFilters,
    id_salt: &'static str,
    t: &Translator,
) {
    const FILTER_HEIGHT: f32 = 30.0;
    ui.set_max_width(ui.available_width());
    egui::ScrollArea::horizontal()
        .id_salt((id_salt, "filter_scroll"))
        .scroll_bar_visibility(egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden)
        .max_height(FILTER_HEIGHT + 4.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            officer_browser_filter_controls(ui, game, filters, id_salt, t, FILTER_HEIGHT);
        });
}

fn officer_browser_filter_controls(
    ui: &mut egui::Ui,
    game: &GameState,
    filters: &mut OfficerBrowserFilters,
    id_salt: &'static str,
    t: &Translator,
    filter_height: f32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().interact_size.y = filter_height;
        ui.spacing_mut().item_spacing.x = 12.0;
        ui.label(t.text("officer-filter-search"));
        ui.add_sized(
            [260.0, filter_height],
            egui::TextEdit::singleline(&mut filters.search)
                .hint_text(t.text("officer-filter-search-hint")),
        );

        egui::ComboBox::from_id_salt((id_salt, "gender"))
            .width(160.0)
            .selected_text(officer_gender_filter_label(filters.gender, t))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerGenderFilter::All,
                    OfficerGenderFilter::Male,
                    OfficerGenderFilter::Female,
                ] {
                    ui.selectable_value(
                        &mut filters.gender,
                        filter,
                        officer_gender_filter_label(filter, t),
                    );
                }
            });

        let selected_faction_text = filters
            .faction_id
            .as_deref()
            .and_then(|id| game.factions.get(id))
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| t.text("officer-filter-all-factions"));
        egui::ComboBox::from_id_salt((id_salt, "faction"))
            .width(170.0)
            .selected_text(selected_faction_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut filters.faction_id,
                    None,
                    t.text("officer-filter-all-factions"),
                );
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
            .selected_text(officer_status_filter_label(filters.status, t))
            .show_ui(ui, |ui| {
                for filter in [
                    OfficerStatusFilter::All,
                    OfficerStatusFilter::Active,
                    OfficerStatusFilter::Minor,
                    OfficerStatusFilter::Wild,
                    OfficerStatusFilter::Unavailable,
                    OfficerStatusFilter::Dead,
                ] {
                    ui.selectable_value(
                        &mut filters.status,
                        filter,
                        officer_status_filter_label(filter, t),
                    );
                }
            });

        let selected_city_text = filters
            .city_id
            .as_deref()
            .and_then(|id| game.cities.get(id))
            .map(|city| city.name.clone())
            .unwrap_or_else(|| t.text("officer-filter-all-cities"));
        egui::ComboBox::from_id_salt((id_salt, "city"))
            .width(170.0)
            .selected_text(selected_city_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut filters.city_id,
                    None,
                    t.text("officer-filter-all-cities"),
                );
                let mut cities: Vec<_> = game.cities.values().collect();
                cities.sort_by(|a, b| a.name.cmp(&b.name));
                for city in cities {
                    ui.selectable_value(&mut filters.city_id, Some(city.id.clone()), &city.name);
                }
            });

        let selected_tag_text = selected_tag_filter_text(game, filters, t);
        egui::ComboBox::from_id_salt((id_salt, "tags"))
            .width(190.0)
            .selected_text(selected_tag_text)
            .show_ui(ui, |ui| {
                if ui.button(t.text("officer-filter-clear-tags")).clicked() {
                    filters.tag_ids.clear();
                    ui.close();
                }
                let definitions_by_category = officer_tag_definitions_by_category(game);
                for (category, definitions) in definitions_by_category {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(officer_tag_category_label(category, t))
                            .color(war_text_muted()),
                    );
                    for definition in definitions {
                        let mut selected = filters.tag_ids.contains(&definition.id);
                        if ui
                            .checkbox(&mut selected, officer_tag_label(definition, t))
                            .changed()
                        {
                            if selected {
                                filters.tag_ids.insert(definition.id.clone());
                            } else {
                                filters.tag_ids.remove(&definition.id);
                            }
                        }
                    }
                }
            });

        if ui
            .add_sized(
                [filter_height, filter_height],
                egui::Button::new(
                    egui::RichText::new(egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE)
                        .size(17.0)
                        .color(war_gold()),
                ),
            )
            .on_hover_text(t.text("common-reset"))
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
    options: OfficerBrowserTableOptions<'_>,
    t: &Translator,
) -> OfficerBrowserTableResponse {
    let rows = options.retainer_faction_id.map_or_else(
        || filtered_officer_rows(filters, game, t),
        |faction_id| retainer_officer_rows(filters, game, faction_id, t),
    );
    let mut table_response = OfficerBrowserTableResponse::default();
    ui.label(t.text_args(
        "officer-browser-count",
        &args([("count", rows.len().to_string())]),
    ));

    let table_height = options.max_height.max(260.0);
    TableBuilder::new(ui)
        .id_salt(options.id_salt)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(table_height)
        .max_scroll_height(table_height)
        .auto_shrink([false, false])
        .column(Column::exact(54.0))
        .column(Column::remainder().at_least(96.0).clip(true))
        .column(Column::exact(52.0))
        .column(Column::exact(46.0))
        .column(Column::remainder().at_least(96.0).clip(true))
        .column(Column::initial(82.0).at_least(64.0).clip(true))
        .column(Column::initial(88.0).at_least(68.0).clip(true))
        .column(Column::exact(58.0))
        .column(Column::initial(72.0).at_least(58.0).clip(true))
        .column(Column::exact(58.0))
        .columns(Column::exact(52.0), 5)
        .header(26.0, |mut header| {
            officer_table_header_cell_centered(&mut header, t.text("officer-column-tools"));
            officer_table_header_cell(&mut header, t.text("officer-column-name"));
            officer_table_header_cell(&mut header, t.text("officer-column-gender"));
            officer_table_header_cell(&mut header, t.text("officer-column-age"));
            officer_table_header_cell(&mut header, t.text("officer-column-faction"));
            officer_table_header_cell(&mut header, t.text("officer-column-city"));
            officer_table_header_cell(&mut header, t.text("officer-column-office"));
            officer_table_header_cell(&mut header, t.text("officer-column-salary"));
            officer_table_header_cell(&mut header, t.text("officer-column-status"));
            officer_table_header_cell(&mut header, t.text("officer-column-loyalty"));
            officer_table_header_cell(&mut header, t.text("stat-leadership"));
            officer_table_header_cell(&mut header, t.text("stat-strength"));
            officer_table_header_cell(&mut header, t.text("stat-intelligence"));
            officer_table_header_cell(&mut header, t.text("stat-politics"));
            officer_table_header_cell(&mut header, t.text("stat-charm"));
        })
        .body(|mut body| {
            for row in rows {
                let selected = options.selected_officer_id == Some(row.id.as_str());
                body.row(30.0, |mut table_row| {
                    table_row.set_selected(selected);
                    table_row.col(|ui| officer_row_tools(ui, &row, &mut table_response, t));
                    table_row.col(|ui| officer_row_cell(ui, &row.name));
                    table_row.col(|ui| officer_row_cell(ui, &row.gender));
                    table_row.col(|ui| officer_row_cell(ui, row.age.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, &row.faction_name));
                    table_row.col(|ui| officer_row_cell(ui, &row.city_name));
                    table_row.col(|ui| officer_row_cell(ui, &row.office_name));
                    table_row.col(|ui| officer_row_cell(ui, row.salary.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, &row.status));
                    table_row.col(|ui| officer_row_cell(ui, row.loyalty.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.leadership.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.strength.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.intelligence.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.politics.to_string()));
                    table_row.col(|ui| officer_row_cell(ui, row.charm.to_string()));

                    let row_response = table_row.response();
                    if row_response.clicked() || row_response.secondary_clicked() {
                        table_response.selected_officer_id = Some(row.id.clone());
                    }
                    if options.editable || options.retainer_faction_id.is_some() {
                        row_response.context_menu(|ui| {
                            officer_row_context_menu(
                                ui,
                                game,
                                &row,
                                &mut table_response,
                                options.editable,
                                options.retainer_faction_id,
                                t,
                            );
                        });
                    }
                });
            }
        });
    table_response
}

pub(super) fn officer_tag_label(definition: &OfficerTagDefinition, t: &Translator) -> String {
    match t.language() {
        UiLanguage::English => definition.label_en.clone(),
        UiLanguage::SimplifiedChinese => definition.label_zh.clone(),
    }
}

pub(super) fn officer_tag_display_text(game: &GameState, tag_id: &str, t: &Translator) -> String {
    game.officer_tag_definitions
        .iter()
        .find(|definition| definition.id == tag_id)
        .map(|definition| officer_tag_label(definition, t))
        .unwrap_or_else(|| tag_id.to_string())
}

fn selected_tag_filter_text(
    game: &GameState,
    filters: &OfficerBrowserFilters,
    t: &Translator,
) -> String {
    match filters.tag_ids.len() {
        0 => t.text("officer-filter-all-tags"),
        1 => filters
            .tag_ids
            .iter()
            .next()
            .map(|tag_id| officer_tag_display_text(game, tag_id, t))
            .unwrap_or_else(|| t.text("officer-filter-all-tags")),
        count => t.text_args(
            "officer-filter-selected-tags",
            &args([("count", count.to_string())]),
        ),
    }
}

pub(super) fn officer_tag_definitions_by_category(
    game: &GameState,
) -> BTreeMap<OfficerTagCategory, Vec<&OfficerTagDefinition>> {
    let mut grouped: BTreeMap<OfficerTagCategory, Vec<&OfficerTagDefinition>> = BTreeMap::new();
    for definition in &game.officer_tag_definitions {
        grouped
            .entry(definition.category)
            .or_default()
            .push(definition);
    }
    grouped
}

pub(super) fn officer_tag_category_label(category: OfficerTagCategory, t: &Translator) -> String {
    let key = match category {
        OfficerTagCategory::Role => "officer-tag-category-role",
        OfficerTagCategory::Affiliation => "officer-tag-category-affiliation",
        OfficerTagCategory::Source => "officer-tag-category-source",
        OfficerTagCategory::Batch => "officer-tag-category-batch",
        OfficerTagCategory::Basis => "officer-tag-category-basis",
        OfficerTagCategory::Region => "officer-tag-category-region",
        OfficerTagCategory::Context => "officer-tag-category-context",
    };
    t.text(key)
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OfficerBrowserTableOptions<'a> {
    pub(super) max_height: f32,
    pub(super) id_salt: &'static str,
    pub(super) selected_officer_id: Option<&'a str>,
    pub(super) editable: bool,
    pub(super) retainer_faction_id: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub(super) struct OfficerBrowserRow {
    id: OfficerId,
    name: String,
    faction_id: FactionId,
    gender: String,
    age: u32,
    faction_name: String,
    city_name: String,
    office_name: String,
    salary: i32,
    status: String,
    loyalty: u8,
    leadership: u8,
    strength: u8,
    intelligence: u8,
    politics: u8,
    charm: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OfficerBrowserTableResponse {
    pub(super) selected_officer_id: Option<OfficerId>,
    pub(super) view_officer_id: Option<OfficerId>,
    pub(super) edit_officer_id: Option<OfficerId>,
    pub(super) appoint_officer_id: Option<OfficerId>,
    pub(super) appoint_office_id: Option<OfficialPostId>,
    pub(super) dismiss_officer_id: Option<OfficerId>,
}

fn officer_table_header_cell(row: &mut egui_extras::TableRow<'_, '_>, text: String) {
    row.col(|ui| {
        ui.strong(text);
    });
}

fn officer_table_header_cell_centered(row: &mut egui_extras::TableRow<'_, '_>, text: String) {
    row.col(|ui| {
        ui.centered_and_justified(|ui| {
            ui.strong(text);
        });
    });
}

fn officer_row_tools(
    ui: &mut egui::Ui,
    row: &OfficerBrowserRow,
    table_response: &mut OfficerBrowserTableResponse,
    t: &Translator,
) {
    let response = ui
        .centered_and_justified(|ui| {
            ui.add_sized(
                [32.0, 24.0],
                egui::Button::new(
                    egui::RichText::new(egui_phosphor::regular::EYE)
                        .size(16.0)
                        .color(war_gold()),
                ),
            )
        })
        .inner
        .on_hover_text(t.text("officer-action-view"));
    if response.clicked() {
        table_response.selected_officer_id = Some(row.id.clone());
        table_response.view_officer_id = Some(row.id.clone());
    }
}

fn officer_row_cell(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
    ui.add(egui::Label::new(text).truncate());
}

fn officer_row_context_menu(
    ui: &mut egui::Ui,
    game: &GameState,
    row: &OfficerBrowserRow,
    table_response: &mut OfficerBrowserTableResponse,
    editable: bool,
    retainer_faction_id: Option<&str>,
    t: &Translator,
) {
    if editable && ui.button(t.text("officer-action-edit")).clicked() {
        table_response.selected_officer_id = Some(row.id.clone());
        table_response.edit_officer_id = Some(row.id.clone());
        ui.close();
    }
    if retainer_faction_id.is_some() {
        ui.menu_button(t.text("officer-action-appoint-office"), |ui| {
            for spec in official_post_specs() {
                let occupant = game.officers.values().find(|officer| {
                    officer.faction_id == row.faction_id
                        && officer.office_id.as_deref() == Some(spec.id)
                });
                let label = if let Some(occupant) = occupant {
                    if occupant.id == row.id {
                        t.text_args(
                            "official-post-current",
                            &args([
                                ("name", spec.name.to_string()),
                                ("rank", official_rank_label(spec.rank).to_string()),
                            ]),
                        )
                    } else {
                        t.text_args(
                            "official-post-occupied",
                            &args([
                                ("name", spec.name.to_string()),
                                ("rank", official_rank_label(spec.rank).to_string()),
                                ("occupant", occupant.name.clone()),
                            ]),
                        )
                    }
                } else {
                    t.text_args(
                        "official-post-empty",
                        &args([
                            ("name", spec.name.to_string()),
                            ("rank", official_rank_label(spec.rank).to_string()),
                        ]),
                    )
                };
                if ui.button(label).clicked() {
                    table_response.selected_officer_id = Some(row.id.clone());
                    table_response.appoint_officer_id = Some(row.id.clone());
                    table_response.appoint_office_id = Some(spec.id.to_string());
                    ui.close();
                }
            }
        });
        if row.office_name != t.text("none")
            && ui.button(t.text("officer-action-dismiss-office")).clicked()
        {
            table_response.selected_officer_id = Some(row.id.clone());
            table_response.dismiss_officer_id = Some(row.id.clone());
            ui.close();
        }
    }
}

pub(super) fn filtered_officer_rows(
    filters: &OfficerBrowserFilters,
    game: &GameState,
    t: &Translator,
) -> Vec<OfficerBrowserRow> {
    let search = filters.search.trim().to_lowercase();
    let officers: Vec<_> = game.officers.values().collect();
    let mut rows: Vec<_> = officers
        .into_iter()
        .filter(|officer| officer_matches_filters(officer, game, filters, &search))
        .map(|officer| {
            let faction_name = game
                .factions
                .get(&officer.faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| t.text("unknown"));
            let city_name = officer
                .city_id
                .as_deref()
                .and_then(|city_id| game.cities.get(city_id))
                .map(|city| city.name.clone())
                .unwrap_or_else(|| t.text("officer-city-unassigned"));
            let office_name = officer
                .office_id
                .as_deref()
                .and_then(official_post_spec)
                .map(|spec| spec.name.to_string())
                .unwrap_or_else(|| t.text("none"));
            OfficerBrowserRow {
                id: officer.id.clone(),
                name: officer.name.clone(),
                faction_id: officer.faction_id.clone(),
                gender: officer_gender_label(t, &officer.gender),
                age: officer.age_at(game.year),
                faction_name,
                city_name,
                office_name,
                salary: officer_monthly_salary(officer),
                status: officer_status_label(&officer.status, t),
                loyalty: officer.loyalty,
                leadership: officer.stats.leadership,
                strength: officer.stats.strength,
                intelligence: officer.stats.intelligence,
                politics: officer.stats.politics,
                charm: officer.stats.charm,
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        (&a.faction_name, &a.city_name, &a.name, &a.id).cmp(&(
            &b.faction_name,
            &b.city_name,
            &b.name,
            &b.id,
        ))
    });
    rows
}

pub(super) fn retainer_officer_rows(
    filters: &OfficerBrowserFilters,
    game: &GameState,
    faction_id: &str,
    t: &Translator,
) -> Vec<OfficerBrowserRow> {
    let mut locked_filters = filters.clone();
    locked_filters.faction_id = Some(faction_id.to_string());
    locked_filters.status = OfficerStatusFilter::Active;
    filtered_officer_rows(&locked_filters, game, t)
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
    if !officer_tags_match_filters(officer, game, filters) {
        return false;
    }
    search.is_empty() || officer_search_text(officer, game).contains(search)
}

fn officer_tags_match_filters(
    officer: &Officer,
    game: &GameState,
    filters: &OfficerBrowserFilters,
) -> bool {
    if filters.tag_ids.is_empty() {
        return true;
    }
    let officer_tags = normalized_officer_tag_ids(officer, game);
    let definition_by_id = game
        .officer_tag_definitions
        .iter()
        .map(|definition| (definition.id.as_str(), definition.category))
        .collect::<BTreeMap<_, _>>();
    let mut selected_by_category: BTreeMap<OfficerTagCategory, BTreeSet<&str>> = BTreeMap::new();
    for tag_id in &filters.tag_ids {
        let Some(category) = definition_by_id.get(tag_id.as_str()).copied() else {
            return false;
        };
        selected_by_category
            .entry(category)
            .or_default()
            .insert(tag_id.as_str());
    }
    selected_by_category
        .values()
        .all(|selected| selected.iter().any(|tag_id| officer_tags.contains(*tag_id)))
}

fn normalized_officer_tag_ids(officer: &Officer, game: &GameState) -> BTreeSet<String> {
    officer
        .profile
        .as_ref()
        .into_iter()
        .flat_map(|profile| profile.tags.iter())
        .filter_map(|tag| {
            if game
                .officer_tag_definitions
                .iter()
                .any(|definition| definition.id == *tag)
            {
                Some(tag.clone())
            } else {
                game.officer_tag_aliases.get(tag).cloned()
            }
        })
        .collect()
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
        for tag_id in normalized_officer_tag_ids(officer, game) {
            text.push(' ');
            text.push_str(&tag_id);
            if let Some(definition) = game
                .officer_tag_definitions
                .iter()
                .find(|definition| definition.id == tag_id)
            {
                text.push(' ');
                text.push_str(&definition.label_zh);
                text.push(' ');
                text.push_str(&definition.label_en);
            }
        }
    }
    if let Some(spec) = officer.office_id.as_deref().and_then(official_post_spec) {
        text.push(' ');
        text.push_str(spec.name);
        text.push(' ');
        text.push_str(official_rank_label(spec.rank));
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
        OfficerStatusFilter::Minor => *status == OfficerStatus::Minor,
        OfficerStatusFilter::Wild => *status == OfficerStatus::Wild,
        OfficerStatusFilter::Unavailable => *status == OfficerStatus::Unavailable,
        OfficerStatusFilter::Dead => *status == OfficerStatus::Dead,
    }
}

fn officer_gender_filter_label(filter: OfficerGenderFilter, t: &Translator) -> String {
    match filter {
        OfficerGenderFilter::All => t.text("officer-filter-all-genders"),
        OfficerGenderFilter::Male => t.text("gender-male"),
        OfficerGenderFilter::Female => t.text("gender-female"),
    }
}

fn officer_status_filter_label(filter: OfficerStatusFilter, t: &Translator) -> String {
    match filter {
        OfficerStatusFilter::All => t.text("officer-filter-all-statuses"),
        OfficerStatusFilter::Active => t.text("officer-status-active"),
        OfficerStatusFilter::Minor => t.text("officer-status-minor"),
        OfficerStatusFilter::Wild => t.text("officer-status-wild"),
        OfficerStatusFilter::Unavailable => t.text("officer-status-unavailable"),
        OfficerStatusFilter::Dead => t.text("officer-status-dead"),
    }
}

fn officer_status_label(status: &OfficerStatus, t: &Translator) -> String {
    match status {
        OfficerStatus::Active => t.text("officer-status-active"),
        OfficerStatus::Minor => t.text("officer-status-minor"),
        OfficerStatus::Wild => t.text("officer-status-wild"),
        OfficerStatus::Unavailable => t.text("officer-status-unavailable"),
        OfficerStatus::Dead => t.text("officer-status-dead"),
    }
}

pub(super) fn save_controls(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    ui.horizontal(|ui| {
        ui.label(t.text("save-slot"));
        egui::ComboBox::from_id_salt("save_slot_combo")
            .selected_text(&ui_state.save_slot_id)
            .show_ui(ui, |ui| {
                for slot_id in ["slot1", "slot2", "slot3", "slot4", "slot5"] {
                    ui.selectable_value(&mut ui_state.save_slot_id, slot_id.to_string(), slot_id);
                }
            });
    });
    ui.horizontal(|ui| {
        ui.label(t.text("save-name"));
        ui.text_edit_singleline(&mut ui_state.save_display_name);
    });
    ui.horizontal(|ui| {
        if ui.button(t.text("common-save")).clicked()
            && let Some(game) = &ui_state.game
        {
            match ui_state.save_manager.save_slot(
                &ui_state.save_slot_id,
                &ui_state.save_display_name,
                game,
            ) {
                Ok(meta) => {
                    refresh_saves(ui_state);
                    ui_state.message =
                        t.text_args("message-save-saved", &args([("name", meta.display_name)]));
                }
                Err(error) => ui_state.message = error.to_string(),
            }
        }
        if ui.button(t.text("save-load-current-slot")).clicked() {
            match ui_state.save_manager.load_slot(&ui_state.save_slot_id) {
                Ok(game) => {
                    enter_game(ui_state, game, t.text("message-save-loaded-current"));
                }
                Err(error) => {
                    let slot_id = ui_state.save_slot_id.clone();
                    let _ = ui_state.save_manager.delete_slot(&slot_id);
                    refresh_saves(ui_state);
                    ui_state.message = t.text_args(
                        "message-save-invalid-discarded",
                        &args([("error", error.to_string())]),
                    );
                }
            }
        }
    });
    if !ui_state.message.is_empty() {
        ui.label(&ui_state.message);
    }
}

pub(super) fn report_panel(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
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
                ui.label(t.text("report-empty"));
            }
            let highlight_entities = report_highlight_entities(game);
            for report in game.reports.iter().skip(visible_start) {
                ui.label(t.text_args(
                    "report-turn-heading",
                    &args([
                        ("year", report.year.to_string()),
                        ("month", report.month.to_string()),
                        ("turn", report.turn.to_string()),
                    ]),
                ));
                for entry in &report.entries {
                    match entry.severity {
                        ReportSeverity::Info => {
                            highlighted_report_label(
                                ui,
                                &format!("· {}", entry.message),
                                &highlight_entities,
                                war_text(),
                            );
                        }
                        ReportSeverity::Warning => {
                            highlighted_report_label(
                                ui,
                                &format!("! {}", entry.message),
                                &highlight_entities,
                                war_warning(),
                            );
                        }
                    }
                }
                ui.separator();
            }
        });
}

fn highlighted_report_label(
    ui: &mut egui::Ui,
    text: &str,
    entities: &[ReportHighlightEntity],
    base_color: egui::Color32,
) {
    ui.label(highlighted_report_job(text, entities, base_color));
}

fn highlighted_report_job(
    text: &str,
    entities: &[ReportHighlightEntity],
    base_color: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for token in tokenize_report_message(text, entities) {
        let color = match token.kind {
            Some(ReportHighlightKind::Officer) => war_gold(),
            Some(ReportHighlightKind::City) => report_city_highlight(),
            None => base_color,
        };
        job.append(&token.text, 0.0, report_text_format(color));
    }
    job
}

fn report_text_format(color: egui::Color32) -> egui::TextFormat {
    egui::TextFormat {
        color,
        ..Default::default()
    }
}

fn report_city_highlight() -> egui::Color32 {
    egui::Color32::from_rgb(132, 184, 207)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReportHighlightEntity {
    name: String,
    kind: ReportHighlightKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReportHighlightKind {
    Officer,
    City,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReportHighlightToken {
    text: String,
    kind: Option<ReportHighlightKind>,
}

fn report_highlight_entities(game: &GameState) -> Vec<ReportHighlightEntity> {
    let mut seen = BTreeSet::new();
    let mut entities = Vec::new();
    for officer in game.officers.values() {
        if !officer.name.is_empty() && seen.insert(officer.name.clone()) {
            entities.push(ReportHighlightEntity {
                name: officer.name.clone(),
                kind: ReportHighlightKind::Officer,
            });
        }
    }
    for city in game.cities.values() {
        if !city.name.is_empty() && seen.insert(city.name.clone()) {
            entities.push(ReportHighlightEntity {
                name: city.name.clone(),
                kind: ReportHighlightKind::City,
            });
        }
    }
    entities
}

fn tokenize_report_message(
    message: &str,
    entities: &[ReportHighlightEntity],
) -> Vec<ReportHighlightToken> {
    if message.is_empty() {
        return Vec::new();
    }

    let mut ordered_entities: Vec<_> = entities.iter().collect();
    ordered_entities.sort_by(|a, b| {
        b.name
            .len()
            .cmp(&a.name.len())
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut tokens = Vec::new();
    let mut plain_start = 0;
    let mut index = 0;

    while index < message.len() {
        let rest = &message[index..];
        if let Some(entity) = ordered_entities
            .iter()
            .find(|entity| rest.starts_with(entity.name.as_str()))
        {
            if plain_start < index {
                tokens.push(ReportHighlightToken {
                    text: message[plain_start..index].to_string(),
                    kind: None,
                });
            }
            tokens.push(ReportHighlightToken {
                text: entity.name.clone(),
                kind: Some(entity.kind),
            });
            index += entity.name.len();
            plain_start = index;
        } else if let Some(ch) = rest.chars().next() {
            index += ch.len_utf8();
        } else {
            break;
        }
    }

    if plain_start < message.len() {
        tokens.push(ReportHighlightToken {
            text: message[plain_start..].to_string(),
            kind: None,
        });
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::i18n::{Translator, UiLanguage};
    use crate::core::settings::{GameSettings, GameSettingsStore, LoadedGameSettings};
    use crate::core::state::{GameUiState, OfficerGenderFilter, OfficerStatusFilter};
    use crate::game::{
        FamilyRelationship, Marriage, OfficerGender, OfficerStatus, SqliteHistoricalCatalog,
    };

    fn ui_state_with_game() -> GameUiState {
        let mut state = GameUiState::new(
            GameSettingsStore::with_default_path(),
            LoadedGameSettings {
                settings: GameSettings::default(),
                message: None,
            },
        );
        state.game = Some(
            SqliteHistoricalCatalog::in_memory_from_seed()
                .unwrap()
                .build_game("ad200", "liu_bei")
                .unwrap(),
        );
        state
    }

    fn zh() -> Translator {
        Translator::new(UiLanguage::SimplifiedChinese)
    }

    #[test]
    fn report_highlight_tokenizer_matches_officers_and_cities() {
        let entities = vec![
            ReportHighlightEntity {
                name: "关羽".to_string(),
                kind: ReportHighlightKind::Officer,
            },
            ReportHighlightEntity {
                name: "平原".to_string(),
                kind: ReportHighlightKind::City,
            },
        ];

        let tokens = tokenize_report_message("关羽 进驻 平原", &entities);

        assert_eq!(
            tokens,
            vec![
                ReportHighlightToken {
                    text: "关羽".to_string(),
                    kind: Some(ReportHighlightKind::Officer),
                },
                ReportHighlightToken {
                    text: " 进驻 ".to_string(),
                    kind: None,
                },
                ReportHighlightToken {
                    text: "平原".to_string(),
                    kind: Some(ReportHighlightKind::City),
                },
            ]
        );
    }

    #[test]
    fn report_highlight_tokenizer_prefers_longest_match() {
        let entities = vec![
            ReportHighlightEntity {
                name: "关羽".to_string(),
                kind: ReportHighlightKind::Officer,
            },
            ReportHighlightEntity {
                name: "关羽改".to_string(),
                kind: ReportHighlightKind::Officer,
            },
        ];

        let tokens = tokenize_report_message("关羽改 完成训练", &entities);

        assert_eq!(
            tokens,
            vec![
                ReportHighlightToken {
                    text: "关羽改".to_string(),
                    kind: Some(ReportHighlightKind::Officer),
                },
                ReportHighlightToken {
                    text: " 完成训练".to_string(),
                    kind: None,
                },
            ]
        );
    }

    #[test]
    fn report_highlight_tokenizer_keeps_unmatched_text_plain() {
        let entities = vec![ReportHighlightEntity {
            name: "关羽".to_string(),
            kind: ReportHighlightKind::Officer,
        }];

        let tokens = tokenize_report_message("本月无事", &entities);

        assert_eq!(
            tokens,
            vec![ReportHighlightToken {
                text: "本月无事".to_string(),
                kind: None,
            }]
        );
    }

    #[test]
    fn report_highlight_entities_collect_officers_before_cities() {
        let state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        let entities = report_highlight_entities(game);

        assert!(entities.iter().any(|entity| {
            entity.name == "刘备" && entity.kind == ReportHighlightKind::Officer
        }));
        assert!(
            entities.iter().any(|entity| {
                entity.name == "平原" && entity.kind == ReportHighlightKind::City
            })
        );
    }

    #[test]
    fn officer_browser_search_matches_name_id_faction_and_city() {
        let mut state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        state.officer_browser_filters.search = "关羽".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game, &zh())
                .iter()
                .any(|row| row.name == "关羽")
        );

        state.officer_browser_filters.search = "guan_yu".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game, &zh())
                .iter()
                .any(|row| row.id == "guan_yu" && row.name == "关羽")
        );

        state.officer_browser_filters.search = "刘备军".to_string();
        assert!(!filtered_officer_rows(&state.officer_browser_filters, game, &zh()).is_empty());

        state.officer_browser_filters.search = "平原".to_string();
        assert!(
            filtered_officer_rows(&state.officer_browser_filters, game, &zh())
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
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| {
            row.gender == "男"
                && row.faction_name == "刘备军"
                && row.status == "在任"
                && row.city_name == "平原"
        }));
    }

    #[test]
    fn officer_browser_tag_filters_use_or_within_category_and_and_across_categories() {
        let mut state = ui_state_with_game();
        state
            .officer_browser_filters
            .tag_ids
            .insert("role:general".to_string());
        state
            .officer_browser_filters
            .tag_ids
            .insert("role:administrator".to_string());
        state
            .officer_browser_filters
            .tag_ids
            .insert("affiliation:shu_han".to_string());

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(!rows.is_empty());
        assert!(rows.iter().all(|row| {
            let officer = &game.officers[&row.id];
            let tags = normalized_officer_tag_ids(officer, game);
            (tags.contains("role:general") || tags.contains("role:administrator"))
                && tags.contains("affiliation:shu_han")
        }));
    }

    #[test]
    fn officer_browser_tag_filter_matches_legacy_profile_tags_and_resets() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            let officer = game.officers.get_mut("liu_bei").unwrap();
            let profile = officer.profile.as_mut().unwrap();
            profile.tags = vec!["ruler".to_string(), "shu_han".to_string()];
        }
        state
            .officer_browser_filters
            .tag_ids
            .insert("role:ruler".to_string());
        state
            .officer_browser_filters
            .tag_ids
            .insert("affiliation:shu_han".to_string());

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(rows.iter().any(|row| row.id == "liu_bei"));
        state.officer_browser_filters.reset();
        assert!(state.officer_browser_filters.tag_ids.is_empty());
    }

    #[test]
    fn officer_browser_empty_filters_return_all_officers_stably_sorted() {
        let state = ui_state_with_game();
        let game = state.game.as_ref().unwrap();

        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());
        let sorted_names = rows.windows(2).all(|pair| {
            (
                &pair[0].faction_name,
                &pair[0].city_name,
                &pair[0].name,
                &pair[0].id,
            ) <= (
                &pair[1].faction_name,
                &pair[1].city_name,
                &pair[1].name,
                &pair[1].id,
            )
        });

        assert_eq!(rows.len(), game.officers.len());
        assert!(sorted_names);
    }

    #[test]
    fn officer_browser_rows_reflect_updated_officer_profile_fields() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            let officer = game.officers.get_mut("guan_yu").unwrap();
            officer.name = "关羽改".to_string();
            officer.gender = OfficerGender::Female;
            officer.stats.leadership = 91;
            officer.stats.strength = 92;
        }
        state.officer_browser_filters.search = "guan_yu".to_string();

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "guan_yu");
        assert_eq!(rows[0].name, "关羽改");
        assert_eq!(rows[0].gender, "女");
        assert_eq!(rows[0].leadership, 91);
        assert_eq!(rows[0].strength, 92);
    }

    #[test]
    fn officer_browser_rows_include_office_and_salary() {
        let mut state = ui_state_with_game();
        let base_salary;
        {
            let game = state.game.as_mut().unwrap();
            base_salary = officer_base_monthly_salary(&game.officers["guan_yu"]);
            appoint_official_post(game, "liu_bei", "guan_yu", "taifu").unwrap();
        }
        state.officer_browser_filters.search = "万石".to_string();

        let game = state.game.as_ref().unwrap();
        let rows = filtered_officer_rows(&state.officer_browser_filters, game, &zh());

        assert!(rows.iter().any(|row| {
            row.id == "guan_yu"
                && row.office_name == "太傅"
                && row.salary == base_salary + official_rank_salary_bonus(OfficialRank::WanShi)
        }));
    }

    #[test]
    fn retainer_rows_only_return_player_active_officers() {
        let mut state = ui_state_with_game();
        state.retainer_filters.faction_id = Some("cao_cao".to_string());
        state.retainer_filters.status = OfficerStatusFilter::Dead;
        {
            let game = state.game.as_mut().unwrap();
            game.officers.get_mut("jian_yong").unwrap().status = OfficerStatus::Unavailable;
        }

        let game = state.game.as_ref().unwrap();
        let rows = retainer_officer_rows(
            &state.retainer_filters,
            game,
            &game.player_faction_id,
            &zh(),
        );

        assert!(!rows.is_empty());
        assert!(
            rows.iter()
                .all(|row| row.faction_id == game.player_faction_id && row.status == "在任")
        );
        assert!(rows.iter().all(|row| row.id != "cao_cao"));
        assert!(rows.iter().all(|row| row.id != "jian_yong"));
    }

    #[test]
    fn dynamic_relationship_lines_include_current_game_relationships() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            game.factions.get_mut("liu_bei").unwrap().heir_id = Some("guan_yu".to_string());
            game.cities.get_mut("pingyuan").unwrap().governor_id = Some("liu_bei".to_string());
            game.marriages.push(Marriage::new(
                "liu_bei".to_string(),
                "zhang_fei".to_string(),
                200,
                1,
            ));
            game.family_relationships.push(FamilyRelationship {
                parent_id: "liu_bei".to_string(),
                child_id: "zhao_yun".to_string(),
            });
            game.family_relationships.push(FamilyRelationship {
                parent_id: "guan_yu".to_string(),
                child_id: "zhao_yun".to_string(),
            });
        }
        let t = zh();
        let game = state.game.as_ref().unwrap();

        let liu_bei_graph = officer_relationship_graph(game, &game.officers["liu_bei"], &t);
        assert!(
            liu_bei_graph
                .edges
                .iter()
                .any(|edge| edge.kind == RelationshipGraphKind::Ruler
                    && edge.target_name == "刘备军")
        );
        assert!(
            liu_bei_graph
                .edges
                .iter()
                .any(|edge| edge.kind == RelationshipGraphKind::Governor
                    && edge.target_name == "平原")
        );
        assert!(
            liu_bei_graph.edges.iter().any(
                |edge| edge.kind == RelationshipGraphKind::Spouse && edge.target_name == "张飞"
            )
        );
        assert!(
            liu_bei_graph
                .edges
                .iter()
                .any(|edge| edge.kind == RelationshipGraphKind::Child
                    && edge.target_name == "赵云")
        );

        let guan_yu_graph = officer_relationship_graph(game, &game.officers["guan_yu"], &t);
        assert!(
            guan_yu_graph.edges.iter().any(
                |edge| edge.kind == RelationshipGraphKind::Heir && edge.target_name == "刘备军"
            )
        );

        let zhao_yun_graph = officer_relationship_graph(game, &game.officers["zhao_yun"], &t);
        assert!(
            zhao_yun_graph
                .edges
                .iter()
                .filter(|edge| edge.kind == RelationshipGraphKind::Parent)
                .map(|edge| edge.target_name.as_str())
                .collect::<Vec<_>>()
                .contains(&"刘备")
        );
        assert!(
            zhao_yun_graph
                .edges
                .iter()
                .filter(|edge| edge.kind == RelationshipGraphKind::Parent)
                .map(|edge| edge.target_name.as_str())
                .collect::<Vec<_>>()
                .contains(&"关羽")
        );
    }
}
