use bevy_egui::egui;
use std::fs;
use std::path::PathBuf;

use super::i18n::Translator;
use super::state::GameUiState;

pub(super) fn configure_egui_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(war_text());
    style.visuals.weak_text_color = Some(war_text_muted());
    style.visuals.window_fill = war_panel_fill();
    style.visuals.panel_fill = egui::Color32::TRANSPARENT;
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(15, 14, 12);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(43, 38, 29);
    style.visuals.hyperlink_color = war_gold();
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(122, 59, 39);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, war_gold());
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 36, 27);
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(57, 45, 32);
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, war_border());
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(83, 61, 37);
    style.visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(94, 69, 42);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, war_gold());
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(110, 53, 37);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, war_gold());
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

pub(super) fn war_panel_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 12))
        .fill(war_panel_fill())
        .stroke(egui::Stroke::new(1.0, war_border()))
        .corner_radius(6)
}

pub(super) fn war_bar_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(14, 9))
        .fill(egui::Color32::from_rgba_unmultiplied(18, 17, 14, 238))
        .stroke(egui::Stroke::new(1.0, war_border()))
        .corner_radius(4)
}

pub(super) fn war_sub_panel_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(10, 8))
        .fill(egui::Color32::from_rgba_unmultiplied(34, 29, 22, 210))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(138, 101, 58, 120),
        ))
        .corner_radius(4)
}

pub(super) fn war_panel_fill() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(24, 21, 16, 232)
}

pub(super) fn war_text() -> egui::Color32 {
    egui::Color32::from_rgb(226, 213, 184)
}

pub(super) fn war_text_muted() -> egui::Color32 {
    egui::Color32::from_rgb(168, 154, 124)
}

pub(super) fn war_gold() -> egui::Color32 {
    egui::Color32::from_rgb(215, 162, 72)
}

pub(super) fn war_border() -> egui::Color32 {
    egui::Color32::from_rgb(118, 85, 48)
}

pub(super) fn war_success() -> egui::Color32 {
    egui::Color32::from_rgb(118, 186, 122)
}

pub(super) fn war_warning() -> egui::Color32 {
    egui::Color32::from_rgb(218, 174, 88)
}

pub(super) fn war_danger() -> egui::Color32 {
    egui::Color32::from_rgb(218, 95, 76)
}

pub(super) fn modal_title_bar(ui: &mut egui::Ui, t: &Translator, title: &str) -> bool {
    let row_width = ui.available_width();
    let button_size = egui::vec2(30.0, 30.0);
    let mut close_clicked = false;

    ui.allocate_ui_with_layout(
        egui::vec2(row_width, 34.0),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(row_width);
            ui.heading(egui::RichText::new(title).color(war_gold()));
            ui.add_space((ui.available_width() - button_size.x).max(0.0));
            close_clicked = close_icon_button(ui, t, button_size).clicked();
        },
    );

    close_clicked
}

fn close_icon_button(ui: &mut egui::Ui, t: &Translator, size: egui::Vec2) -> egui::Response {
    ui.add_sized(
        size,
        egui::Button::new(
            egui::RichText::new(egui_phosphor::regular::X)
                .size(18.0)
                .color(war_gold()),
        ),
    )
    .on_hover_text(t.text("common-close"))
}

pub(super) fn draw_strategy_map_background(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(35, 39, 29));

    let mut x = rect.left() - rect.left().rem_euclid(72.0);
    while x < rect.right() {
        painter.line_segment(
            [
                egui::pos2(x, rect.top()),
                egui::pos2(x + 44.0, rect.bottom()),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(120, 104, 65, 24)),
        );
        x += 72.0;
    }

    let mut y = rect.top() - rect.top().rem_euclid(58.0);
    while y < rect.bottom() {
        painter.line_segment(
            [
                egui::pos2(rect.left(), y),
                egui::pos2(rect.right(), y + 18.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(94, 120, 93, 18)),
        );
        y += 58.0;
    }

    let river_y = rect.center().y + rect.height() * 0.12;
    let river = [
        egui::pos2(rect.left() - 80.0, river_y - 28.0),
        egui::pos2(rect.left() + rect.width() * 0.22, river_y + 22.0),
        egui::pos2(rect.left() + rect.width() * 0.44, river_y - 10.0),
        egui::pos2(rect.left() + rect.width() * 0.68, river_y + 30.0),
        egui::pos2(rect.right() + 80.0, river_y - 18.0),
    ];
    for segment in river.windows(2) {
        painter.line_segment(
            [segment[0], segment[1]],
            egui::Stroke::new(15.0, egui::Color32::from_rgba_unmultiplied(40, 83, 84, 48)),
        );
        painter.line_segment(
            [segment[0], segment[1]],
            egui::Stroke::new(
                3.0,
                egui::Color32::from_rgba_unmultiplied(111, 143, 126, 74),
            ),
        );
    }

    painter.rect_stroke(
        rect.shrink(8.0),
        0.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(180, 132, 74, 76)),
        egui::StrokeKind::Inside,
    );
}

pub(super) fn configure_egui_fonts(ctx: &egui::Context, ui_state: &mut GameUiState) {
    if ui_state.egui_font_configured {
        return;
    }
    ui_state.egui_font_configured = true;

    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    if let Some(bytes) = load_cjk_font_bytes() {
        fonts.font_data.insert(
            "shogun_cjk".to_string(),
            egui::FontData::from_owned(bytes).into(),
        );
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            family.push("shogun_cjk".to_string());
        }
        if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            family.push("shogun_cjk".to_string());
        }
    }

    ctx.set_fonts(fonts);
}

pub(super) fn load_cjk_font_bytes() -> Option<Vec<u8>> {
    let candidates = vec![
        PathBuf::from("assets/fonts/LXGWWenKai-Regular.ttf"),
        PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
        PathBuf::from("/System/Library/Fonts/STHeiti Light.ttc"),
        PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("C:/Windows/Fonts/msyh.ttc"),
    ];

    candidates.into_iter().find_map(|path| fs::read(path).ok())
}
