use crate::game::*;
use bevy_egui::egui;

use super::super::i18n::Translator;
use super::super::style::{war_danger, war_gold, war_text_muted};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::core::hud) enum RelationshipGraphKind {
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

pub(in crate::core::hud) fn compact_node_label(name: &str) -> String {
    let chars = name.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        name.to_string()
    } else {
        chars.into_iter().take(4).collect()
    }
}

pub(in crate::core::hud) fn relationship_kind_color(kind: RelationshipGraphKind) -> egui::Color32 {
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

pub(in crate::core::hud) fn officer_display_name(game: &GameState, officer_id: &str) -> String {
    game.officers
        .get(officer_id)
        .map(|officer| officer.name.clone())
        .unwrap_or_else(|| officer_id.to_string())
}

pub(in crate::core::hud) fn faction_name(game: &GameState, faction_id: &str) -> String {
    game.factions
        .get(faction_id)
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| faction_id.to_string())
}

pub(in crate::core::hud) fn officer_status_label(status: &OfficerStatus, t: &Translator) -> String {
    match status {
        OfficerStatus::Active => t.text("officer-status-active"),
        OfficerStatus::Minor => t.text("officer-status-minor"),
        OfficerStatus::Wild => t.text("officer-status-wild"),
        OfficerStatus::Unavailable => t.text("officer-status-unavailable"),
        OfficerStatus::Dead => t.text("officer-status-dead"),
    }
}
