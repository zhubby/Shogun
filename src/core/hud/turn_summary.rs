use crate::game::*;
use bevy_egui::egui;

use super::super::HUD_MARGIN;
use super::super::i18n::{Translator, args};
use super::super::state::GameUiState;
use super::super::style::{
    modal_title_bar, war_danger, war_gold, war_panel_frame, war_sub_panel_frame, war_success,
    war_text, war_text_muted, war_warning,
};
use super::save_report::{highlighted_report_label, report_highlight_entities};

pub(super) fn turn_summary_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.turn_summary_open {
        return;
    }

    let Some(snapshot) = TurnSummarySnapshot::from_ui_state(ui_state, t) else {
        ui_state.turn_summary_open = false;
        ui_state.turn_summary_report_index = None;
        return;
    };

    egui::Area::new(egui::Id::new("hud_turn_summary_scrim"))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(screen.size(), egui::Sense::click());
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 145),
            );
            if response.clicked() {
                close_turn_summary(ui_state);
            }
        });

    let modal_width = (screen.width() * 0.56)
        .clamp(560.0, 780.0)
        .min((screen.width() - HUD_MARGIN * 2.0).max(360.0));
    let modal_height = (screen.height() * 0.72)
        .clamp(430.0, 620.0)
        .min((screen.height() - HUD_MARGIN * 2.0).max(340.0));

    egui::Area::new(egui::Id::new("hud_turn_summary"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            war_panel_frame().show(ui, |ui| {
                ui.set_width(modal_width);
                ui.set_min_height(modal_height);
                if modal_title_bar(ui, t, &t.text("turn-summary-title")) {
                    close_turn_summary(ui_state);
                }
                ui.separator();
                turn_summary_content(ui, ui_state, t, &snapshot, modal_height);
            });
        });
}

fn turn_summary_content(
    ui: &mut egui::Ui,
    ui_state: &mut GameUiState,
    t: &Translator,
    snapshot: &TurnSummarySnapshot,
    modal_height: f32,
) {
    ui.label(
        egui::RichText::new(t.text_args(
            "turn-summary-subtitle",
            &args([
                ("year", snapshot.report.year.to_string()),
                ("month", snapshot.report.month.to_string()),
                ("turn", snapshot.report.turn.to_string()),
                ("faction", snapshot.faction_name.clone()),
            ]),
        ))
        .color(war_text_muted()),
    );
    ui.add_space(8.0);
    summary_metrics(ui, t, &snapshot.summary);
    ui.add_space(10.0);
    report_section(ui, t, snapshot, modal_height);
    ui.add_space(10.0);

    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), 36.0),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            if ui
                .add_sized(
                    egui::vec2(108.0, 32.0),
                    egui::Button::new(t.text("turn-summary-continue")),
                )
                .clicked()
            {
                close_turn_summary(ui_state);
            }
            if ui
                .add_sized(
                    egui::vec2(108.0, 32.0),
                    egui::Button::new(t.text("turn-summary-open-report")),
                )
                .clicked()
            {
                close_turn_summary(ui_state);
                ui_state.reports_open = true;
            }
        },
    );
}

fn summary_metrics(ui: &mut egui::Ui, t: &Translator, summary: &FactionTurnSummary) {
    let metrics = [
        SummaryMetric::new(
            egui_phosphor::regular::BANK,
            t.text("turn-summary-cities"),
            summary.city_count.to_string(),
            war_gold(),
        ),
        SummaryMetric::new(
            egui_phosphor::regular::COINS,
            t.text("resource-gold"),
            summary.gold.to_string(),
            war_gold(),
        ),
        SummaryMetric::new(
            egui_phosphor::regular::GRAINS,
            t.text("resource-food"),
            summary.food.to_string(),
            war_success(),
        ),
        SummaryMetric::new(
            egui_phosphor::regular::STACK,
            t.text("resource-materials"),
            summary.materials.to_string(),
            egui::Color32::from_rgb(176, 153, 116),
        ),
        SummaryMetric::new(
            egui_phosphor::regular::SWORD,
            t.text("resource-troops"),
            summary.troops.to_string(),
            egui::Color32::from_rgb(185, 128, 96),
        ),
        SummaryMetric::new(
            egui_phosphor::regular::HEART,
            t.text("turn-summary-wounded-population"),
            t.text_args(
                "turn-summary-wounded-population-value",
                &args([
                    ("wounded", summary.wounded.to_string()),
                    ("population", summary.population.to_string()),
                ]),
            ),
            war_danger(),
        ),
    ];

    for row in metrics.chunks(3) {
        ui.columns(3, |columns| {
            for (column, metric) in columns.iter_mut().zip(row) {
                metric_card(column, metric);
            }
        });
        ui.add_space(6.0);
    }
}

fn metric_card(ui: &mut egui::Ui, metric: &SummaryMetric) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.set_min_height(54.0);
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(metric.icon)
                    .size(19.0)
                    .color(metric.color),
            );
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&metric.label)
                        .size(12.0)
                        .color(war_text_muted()),
                );
                ui.label(
                    egui::RichText::new(&metric.value)
                        .size(18.0)
                        .color(war_text())
                        .strong(),
                );
            });
        });
    });
}

fn report_section(
    ui: &mut egui::Ui,
    t: &Translator,
    snapshot: &TurnSummarySnapshot,
    modal_height: f32,
) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            egui::RichText::new(t.text("turn-summary-report-section"))
                .color(war_gold())
                .strong(),
        );
        ui.separator();
        let report_height = (modal_height * 0.34).clamp(150.0, 240.0);
        egui::ScrollArea::vertical()
            .id_salt("turn_summary_report_scroll")
            .max_height(report_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &snapshot.report.entries {
                    match entry.severity {
                        ReportSeverity::Info => highlighted_report_label(
                            ui,
                            &format!("· {}", entry.message),
                            &snapshot.highlight_entities,
                            war_text(),
                        ),
                        ReportSeverity::Warning => highlighted_report_label(
                            ui,
                            &format!("! {}", entry.message),
                            &snapshot.highlight_entities,
                            war_warning(),
                        ),
                    }
                }
            });
    });
}

fn close_turn_summary(ui_state: &mut GameUiState) {
    ui_state.turn_summary_open = false;
    ui_state.turn_summary_report_index = None;
}

struct TurnSummarySnapshot {
    report: TurnReport,
    faction_name: String,
    summary: FactionTurnSummary,
    highlight_entities: Vec<super::save_report::ReportHighlightEntity>,
}

impl TurnSummarySnapshot {
    fn from_ui_state(ui_state: &GameUiState, t: &Translator) -> Option<Self> {
        let game = ui_state.game.as_ref()?;
        let report_index = ui_state.turn_summary_report_index?;
        let report = game.reports.get(report_index)?.clone();
        let faction_name = game
            .factions
            .get(&game.player_faction_id)
            .map(|faction| faction.name.clone())
            .unwrap_or_else(|| t.text("unknown-faction"));
        Some(Self {
            report,
            faction_name,
            summary: FactionTurnSummary::from_game(game),
            highlight_entities: report_highlight_entities(game),
        })
    }
}

#[derive(Default)]
struct FactionTurnSummary {
    city_count: usize,
    gold: i64,
    food: i64,
    materials: i64,
    troops: u64,
    wounded: u64,
    population: u64,
}

impl FactionTurnSummary {
    fn from_game(game: &GameState) -> Self {
        let mut summary = Self::default();
        for city in game
            .cities
            .values()
            .filter(|city| city.faction_id == game.player_faction_id)
        {
            summary.city_count += 1;
            summary.gold += i64::from(city.gold);
            summary.food += i64::from(city.food);
            summary.materials += i64::from(city.materials);
            summary.troops += u64::from(city.troops.total());
            summary.wounded += u64::from(city.wounded_troops.total());
            summary.population += u64::from(city.population);
        }
        summary
    }
}

struct SummaryMetric {
    icon: &'static str,
    label: String,
    value: String,
    color: egui::Color32,
}

impl SummaryMetric {
    fn new(icon: &'static str, label: String, value: String, color: egui::Color32) -> Self {
        Self {
            icon,
            label,
            value,
            color,
        }
    }
}
