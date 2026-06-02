use crate::game::*;
use bevy_egui::egui;

use super::super::HUD_MARGIN;
use super::super::i18n::{Translator, args};
use super::super::state::GameUiState;
use super::super::style::{
    modal_title_bar, war_danger, war_gold, war_panel_frame, war_sub_panel_frame, war_success,
    war_text_muted, war_warning,
};
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

pub(in crate::core::hud) fn event_button_label(ui_state: &GameUiState, t: &Translator) -> String {
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

pub(in crate::core::hud) fn ensure_selected_event(ui_state: &mut GameUiState) {
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
        GameEventKind::NaturalDisaster => t.text("event-kind-natural-disaster"),
        GameEventKind::PublicOrder => t.text("event-kind-public-order"),
        GameEventKind::Economy => t.text("event-kind-economy"),
        GameEventKind::Military => t.text("event-kind-military"),
        GameEventKind::Diplomacy => t.text("event-kind-diplomacy"),
        GameEventKind::Opportunity => t.text("event-kind-opportunity"),
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
