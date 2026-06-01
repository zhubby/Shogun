use crate::game::{City, CityMonthlyProjection};
use bevy_egui::egui;

use super::i18n::{Translator, args};
use super::labels::facility_kind_label;
use super::style::{war_danger, war_gold, war_success, war_text, war_text_muted};

const TILE_HEIGHT: f32 = 46.0;
const COMPACT_TILE_HEIGHT: f32 = 42.0;
const TILE_GAP: f32 = 6.0;
const COMPACT_TILE_GAP: f32 = 5.0;
const DEVELOPMENT_MAX: f32 = 999.0;
const STABILITY_MAX: f32 = 100.0;

pub(super) fn city_summary_intel(
    ui: &mut egui::Ui,
    city: &City,
    faction_name: &str,
    t: &Translator,
) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(&city.name)
                .size(22.0)
                .color(war_gold())
                .strong(),
        );
        ui.horizontal_wrapped(|ui| {
            muted_label(
                ui,
                t.text_args("city-owner", &args([("faction", faction_name.to_string())])),
            );
            ui.separator();
            muted_label(
                ui,
                t.text_args(
                    "city-level-slots",
                    &args([
                        ("level", city.level.to_string()),
                        ("slots", city.facility_slots().to_string()),
                    ]),
                ),
            );
        });
    });

    ui.add_space(7.0);
    metric_tiles(
        ui,
        &[
            MetricTile::new(
                egui_phosphor::regular::USERS,
                t.text("resource-population"),
                city.population,
            ),
            MetricTile::new(
                egui_phosphor::regular::SWORD,
                t.text("resource-troops"),
                city.troops.total(),
            ),
            MetricTile::new(
                egui_phosphor::regular::COINS,
                t.text("resource-gold"),
                city.gold,
            ),
            MetricTile::new(
                egui_phosphor::regular::BARN,
                t.text("resource-food"),
                city.food,
            ),
            MetricTile::new(
                egui_phosphor::regular::STACK,
                t.text("resource-materials"),
                city.materials,
            ),
        ],
        2,
        COMPACT_TILE_HEIGHT,
        COMPACT_TILE_GAP,
        true,
    );
}

pub(super) fn city_overview_intel(
    ui: &mut egui::Ui,
    city: &City,
    faction_name: &str,
    t: &Translator,
) {
    header_band(ui, city, faction_name, t);
}

pub(super) fn city_resource_intel(ui: &mut egui::Ui, city: &City, t: &Translator) {
    intel_group(ui, &t.text("city-section-resources"), |ui| {
        metric_tiles(
            ui,
            &[
                MetricTile::new(
                    egui_phosphor::regular::COINS,
                    t.text("resource-gold"),
                    city.gold,
                ),
                MetricTile::new(
                    egui_phosphor::regular::BARN,
                    t.text("resource-food"),
                    city.food,
                ),
                MetricTile::new(
                    egui_phosphor::regular::STACK,
                    t.text("resource-materials"),
                    city.materials,
                ),
                MetricTile::new(
                    egui_phosphor::regular::USERS,
                    t.text("resource-population"),
                    city.population,
                ),
                MetricTile::new(
                    egui_phosphor::regular::SWORD,
                    t.text("resource-troops"),
                    city.troops.total(),
                ),
            ],
            2,
            TILE_HEIGHT,
            TILE_GAP,
            false,
        );
        muted_label(ui, troop_pool_summary(city, t));
    });
}

pub(super) fn city_development_intel(ui: &mut egui::Ui, city: &City, t: &Translator) {
    intel_group(ui, &t.text("city-section-development"), |ui| {
        progress_metric(
            ui,
            egui_phosphor::regular::PLANT,
            t.text("development-focus-agriculture"),
            city.agriculture,
            DEVELOPMENT_MAX,
            war_success(),
        );
        progress_metric(
            ui,
            egui_phosphor::regular::BANK,
            t.text("development-focus-commerce"),
            city.commerce,
            DEVELOPMENT_MAX,
            war_gold(),
        );
        progress_metric(
            ui,
            egui_phosphor::regular::SHIELD,
            t.text("development-focus-defense"),
            city.defense,
            DEVELOPMENT_MAX,
            egui::Color32::from_rgb(142, 169, 191),
        );
    });
}

pub(super) fn city_stability_intel(ui: &mut egui::Ui, city: &City, t: &Translator) {
    intel_group(ui, &t.text("city-section-stability"), |ui| {
        progress_metric(
            ui,
            egui_phosphor::regular::SWORD,
            t.text("city-training"),
            city.training,
            STABILITY_MAX,
            egui::Color32::from_rgb(185, 128, 96),
        );
        progress_metric(
            ui,
            egui_phosphor::regular::SCALES,
            t.text("development-focus-order"),
            city.order,
            STABILITY_MAX,
            war_success(),
        );
    });
}

pub(super) fn city_monthly_trend_intel(
    ui: &mut egui::Ui,
    projection: &CityMonthlyProjection,
    t: &Translator,
) {
    intel_group(ui, &t.text("city-section-monthly-trend"), |ui| {
        trend_rows(
            ui,
            &[
                TrendMetric::new(
                    egui_phosphor::regular::COINS,
                    t.text("resource-gold"),
                    projection.net_gold,
                ),
                TrendMetric::new(
                    egui_phosphor::regular::BARN,
                    t.text("resource-food"),
                    projection.net_food,
                ),
                TrendMetric::new(
                    egui_phosphor::regular::STACK,
                    t.text("resource-materials"),
                    projection.net_materials,
                ),
                TrendMetric::new(
                    egui_phosphor::regular::USERS,
                    t.text("resource-population"),
                    projection.population_delta,
                ),
                TrendMetric::new(
                    egui_phosphor::regular::SWORD,
                    t.text("resource-troops"),
                    projection.troop_delta,
                ),
            ],
        );
    });
}

pub(super) fn city_facility_intel(ui: &mut egui::Ui, city: &City, t: &Translator) {
    intel_group(ui, &t.text("city-section-facilities"), |ui| {
        if city.facilities.is_empty() {
            muted_label(ui, t.text("city-no-facilities"));
        } else {
            ui.label(facility_list(city, t));
        }
    });
}

fn header_band(ui: &mut egui::Ui, city: &City, faction_name: &str, t: &Translator) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(8, 7))
        .fill(egui::Color32::from_rgba_unmultiplied(44, 36, 25, 150))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(174, 126, 63, 105),
        ))
        .corner_radius(4)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(
                        t.text_args("city-level", &args([("level", city.level.to_string())])),
                    )
                    .color(war_gold())
                    .strong(),
                );
                ui.separator();
                ui.label(t.text_args(
                    "city-slots",
                    &args([("slots", city.facility_slots().to_string())]),
                ));
                ui.separator();
                muted_label(
                    ui,
                    t.text_args("city-owner", &args([("faction", faction_name.to_string())])),
                );
            });
        });
}

fn intel_group(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).color(war_gold()).strong());
            ui.add_space(2.0);
            ui.separator();
        });
        ui.add_space(3.0);
        add_contents(ui);
    });
}

fn metric_tiles(
    ui: &mut egui::Ui,
    metrics: &[MetricTile],
    columns: usize,
    height: f32,
    gap: f32,
    compact: bool,
) {
    let columns = columns.max(1);
    for row in metrics.chunks(columns) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            let total_gap = gap * (columns.saturating_sub(1) as f32);
            let tile_width = ((ui.available_width() - total_gap) / columns as f32).max(76.0);
            for metric in row {
                draw_metric_tile(ui, metric, egui::vec2(tile_width, height), compact);
            }
        });
        ui.add_space(gap);
    }
}

fn draw_metric_tile(ui: &mut egui::Ui, metric: &MetricTile, size: egui::Vec2, compact: bool) {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter();
    painter.rect_filled(
        rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(28, 24, 18, 190),
    );
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 86, 46, 95)),
        egui::StrokeKind::Inside,
    );

    let icon_pos = egui::pos2(
        rect.left() + 7.0,
        rect.top() + if compact { 6.0 } else { 7.0 },
    );
    painter.text(
        icon_pos,
        egui::Align2::LEFT_TOP,
        metric.icon,
        egui::FontId::proportional(if compact { 15.0 } else { 16.0 }),
        war_gold(),
    );

    painter.text(
        egui::pos2(
            rect.left() + 28.0,
            rect.top() + if compact { 5.0 } else { 6.0 },
        ),
        egui::Align2::LEFT_TOP,
        &metric.label,
        egui::FontId::proportional(12.0),
        war_text_muted(),
    );

    let value_size = if compact { 16.0 } else { 17.0 };
    painter.text(
        egui::pos2(
            rect.left() + 8.0,
            rect.bottom() - if compact { 19.0 } else { 20.0 },
        ),
        egui::Align2::LEFT_TOP,
        format_number(metric.value),
        egui::FontId::proportional(value_size),
        war_text(),
    );

    response.on_hover_text(format!("{} {}", metric.label, metric.value));
}

fn progress_metric(
    ui: &mut egui::Ui,
    icon: &'static str,
    label: String,
    value: impl Into<i32>,
    max: f32,
    fill: egui::Color32,
) {
    let value = value.into();
    ui.horizontal(|ui| {
        ui.set_height(22.0);
        ui.label(egui::RichText::new(icon).size(15.0).color(war_gold()));
        ui.add_sized(
            egui::vec2(36.0, 18.0),
            egui::Label::new(egui::RichText::new(label).color(war_text_muted())),
        );
        let progress = (value as f32 / max).clamp(0.0, 1.0);
        let bar_width = (ui.available_width() - 52.0).max(90.0);
        ui.add(
            egui::ProgressBar::new(progress)
                .desired_width(bar_width)
                .desired_height(10.0)
                .fill(fill)
                .corner_radius(2),
        );
        ui.add_sized(
            egui::vec2(42.0, 18.0),
            egui::Label::new(
                egui::RichText::new(value.to_string())
                    .color(war_text())
                    .strong(),
            ),
        );
    });
}

fn trend_rows(ui: &mut egui::Ui, metrics: &[TrendMetric]) {
    for row in metrics.chunks(2) {
        ui.columns(2, |columns| {
            for (column, metric) in columns.iter_mut().zip(row) {
                trend_metric(column, metric);
            }
        });
    }
}

fn trend_metric(ui: &mut egui::Ui, metric: &TrendMetric) {
    ui.horizontal(|ui| {
        let color = trend_color(metric.delta);
        let trend_icon = if metric.delta < 0 {
            egui_phosphor::regular::TREND_DOWN
        } else {
            egui_phosphor::regular::TREND_UP
        };
        ui.label(
            egui::RichText::new(metric.icon)
                .size(14.0)
                .color(war_gold()),
        );
        ui.add_sized(
            egui::vec2(34.0, 18.0),
            egui::Label::new(egui::RichText::new(&metric.label).color(war_text_muted())),
        );
        ui.label(
            egui::RichText::new(format!("{trend_icon} {:+}", metric.delta))
                .color(color)
                .strong(),
        );
    });
}

fn muted_label(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.colored_label(war_text_muted(), text.into());
}

fn trend_color(delta: i32) -> egui::Color32 {
    match delta.cmp(&0) {
        std::cmp::Ordering::Greater => war_success(),
        std::cmp::Ordering::Less => war_danger(),
        std::cmp::Ordering::Equal => war_text_muted(),
    }
}

fn format_number(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let digits = value.abs().to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3 + sign.len());
    output.push_str(sign);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(digit);
    }
    output
}

fn facility_list(city: &City, t: &Translator) -> String {
    city.facilities
        .iter()
        .map(|facility| {
            t.text_args(
                "facility-level",
                &args([
                    ("name", facility_kind_label(t, facility.kind)),
                    ("level", facility.level.to_string()),
                ]),
            )
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn troop_pool_summary(city: &City, t: &Translator) -> String {
    t.text_args(
        "troop-pool-short",
        &args([
            ("infantry", city.troops.infantry.to_string()),
            ("cavalry", city.troops.cavalry.to_string()),
            ("archers", city.troops.archers.to_string()),
        ]),
    )
}

struct MetricTile {
    icon: &'static str,
    label: String,
    value: i64,
}

impl MetricTile {
    fn new(icon: &'static str, label: String, value: impl Into<i64>) -> Self {
        Self {
            icon,
            label,
            value: value.into(),
        }
    }
}

struct TrendMetric {
    icon: &'static str,
    label: String,
    delta: i32,
}

impl TrendMetric {
    fn new(icon: &'static str, label: String, delta: i32) -> Self {
        Self { icon, label, delta }
    }
}
