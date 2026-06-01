use bevy_egui::egui;

use super::i18n::{Translator, args};
use super::portraits::{OfficerPortraitTaskState, OfficerPortraitTextureView};
use super::style::{war_border, war_danger, war_success, war_text_muted, war_warning};

pub(super) fn paint_officer_portrait_preview(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    texture: Option<OfficerPortraitTextureView>,
    generating: bool,
    t: &Translator,
) {
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(13, 12, 10, 225),
    );
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, war_border()),
        egui::StrokeKind::Inside,
    );

    if let Some(texture) = texture {
        let image_size =
            fit_contained_size(texture.image_size, rect.size() - egui::vec2(12.0, 12.0));
        let image_rect = egui::Align2::CENTER_CENTER.align_size_within_rect(image_size, rect);
        painter.image(
            texture.texture_id,
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        let text = if generating {
            t.text("officer-portrait-generating")
        } else {
            t.text("officer-portrait-empty")
        };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(15.0),
            war_text_muted(),
        );
    }
}

pub(super) fn officer_portrait_status_line(
    ui: &mut egui::Ui,
    t: &Translator,
    task_state: &OfficerPortraitTaskState,
    has_portrait: bool,
    load_error: Option<&str>,
) {
    if let Some(error) = load_error {
        ui.colored_label(
            war_danger(),
            t.text_args(
                "officer-portrait-load-failed",
                &args([("error", error.to_string())]),
            ),
        );
        return;
    }

    match task_state {
        OfficerPortraitTaskState::Idle => {
            if has_portrait {
                ui.colored_label(war_success(), t.text("officer-portrait-generated"));
            } else {
                ui.colored_label(war_text_muted(), t.text("officer-portrait-ready"));
            }
        }
        OfficerPortraitTaskState::Generating => {
            ui.colored_label(war_warning(), t.text("officer-portrait-generating"));
        }
        OfficerPortraitTaskState::Succeeded { .. } => {
            ui.colored_label(war_success(), t.text("officer-portrait-generated"));
        }
        OfficerPortraitTaskState::Failed(error) => {
            ui.colored_label(
                war_danger(),
                t.text_args("officer-portrait-failed", &args([("error", error.clone())])),
            );
        }
    }
}

pub(super) fn fit_contained_size(image_size: egui::Vec2, available_size: egui::Vec2) -> egui::Vec2 {
    if image_size.x <= 0.0
        || image_size.y <= 0.0
        || available_size.x <= 0.0
        || available_size.y <= 0.0
    {
        return egui::Vec2::ZERO;
    }

    let scale = (available_size.x / image_size.x)
        .min(available_size.y / image_size.y)
        .max(0.0);
    image_size * scale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_fit_respects_display_bounds() {
        let image_size = egui::vec2(768.0, 1024.0);

        for available_size in [egui::vec2(224.0, 298.0), egui::vec2(180.0, 340.0)] {
            let display_size = fit_contained_size(image_size, available_size);

            assert!(display_size.x <= available_size.x + f32::EPSILON);
            assert!(display_size.y <= available_size.y + f32::EPSILON);
            assert!((display_size.x / display_size.y - image_size.x / image_size.y).abs() < 0.001);
        }
    }
}
