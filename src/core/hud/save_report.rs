use crate::game::*;
use bevy_egui::egui;
use std::collections::BTreeSet;

use super::super::actions::{enter_game, refresh_saves};
use super::super::i18n::{Translator, args};
use super::super::state::GameUiState;
use super::super::style::{collapse_icon_button, war_gold, war_panel_frame, war_text, war_warning};
use super::super::{HUD_MARGIN, HUD_TOP_HEIGHT, HUD_TOP_OFFSET};
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

pub(in crate::core::hud) fn highlighted_report_label(
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
pub(in crate::core::hud) struct ReportHighlightEntity {
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

pub(in crate::core::hud) fn report_highlight_entities(
    game: &GameState,
) -> Vec<ReportHighlightEntity> {
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
#[cfg(test)]
mod tests {
    use super::super::test_support::ui_state_with_game;
    use super::*;
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
}
