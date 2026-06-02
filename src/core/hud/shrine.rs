use crate::game::*;
use bevy_egui::egui;
use egui_extras::{Size, StripBuilder};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::super::i18n::{Translator, args};
use super::super::state::{GameUiState, ShrineTab};
use super::super::style::{
    modal_title_bar, square_icon_button, war_border, war_danger, war_gold, war_panel_frame,
    war_sub_panel_frame, war_success, war_text, war_text_muted, war_warning,
};
use super::officer_common::{
    RelationshipGraphKind, compact_node_label, officer_display_name, officer_status_label,
    relationship_kind_color,
};
pub(super) fn shrine_hud(
    ctx: &egui::Context,
    ui_state: &mut GameUiState,
    t: &Translator,
    screen: egui::Rect,
) {
    if !ui_state.shrine_open {
        return;
    }

    let width = (screen.width() * 0.84).clamp(800.0, 1180.0);
    let height = (screen.height() * 0.78).clamp(460.0, 720.0);
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
                    for (tab, icon, label, accent) in [
                        (
                            ShrineTab::Kinship,
                            egui_phosphor::regular::TREE_STRUCTURE,
                            t.text("shrine-tab-kinship"),
                            egui::Color32::from_rgb(119, 184, 141),
                        ),
                        (
                            ShrineTab::Marriage,
                            egui_phosphor::regular::HEART,
                            t.text("shrine-tab-marriage"),
                            egui::Color32::from_rgb(217, 126, 118),
                        ),
                        (
                            ShrineTab::ChildrenHeir,
                            egui_phosphor::regular::BABY,
                            t.text("shrine-tab-children-heir"),
                            egui::Color32::from_rgb(120, 178, 211),
                        ),
                        (
                            ShrineTab::Abdication,
                            egui_phosphor::regular::SEAL,
                            t.text("shrine-tab-abdication"),
                            war_gold(),
                        ),
                    ] {
                        if shrine_tab_button(ui, ui_state.shrine_tab == tab, icon, label, accent)
                            .clicked()
                        {
                            ui_state.shrine_tab = tab;
                            ui_state.shrine_abdication_confirm = false;
                        }
                    }
                });
                ui.separator();
                match ui_state.shrine_tab {
                    ShrineTab::Kinship => shrine_kinship_panel(ui, ui_state, t, height - 116.0),
                    ShrineTab::Marriage => shrine_marriage_panel(ui, ui_state, t, height - 112.0),
                    ShrineTab::ChildrenHeir => {
                        shrine_children_heir_panel(ui, ui_state, t, height - 112.0)
                    }
                    ShrineTab::Abdication => {
                        shrine_abdication_panel(ui, ui_state, t, height - 112.0)
                    }
                }
            });
        });
}

fn shrine_tab_button(
    ui: &mut egui::Ui,
    selected: bool,
    icon: &str,
    label: String,
    accent: egui::Color32,
) -> egui::Response {
    let color = if selected { accent } else { war_text() };
    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(84, 54, 30, 190)
    } else {
        egui::Color32::from_rgba_unmultiplied(38, 31, 23, 120)
    };
    ui.add_sized(
        egui::vec2(142.0, 32.0),
        egui::Button::new(
            egui::RichText::new(format!("{icon} {label}"))
                .size(15.0)
                .color(color),
        )
        .selected(selected)
        .fill(fill),
    )
}

fn shrine_kinship_panel(
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
    let Some(faction) = game.factions.get(&faction_id) else {
        ui.label(t.text("unknown-faction"));
        return;
    };
    let Some(graph) = shrine_kinship_graph(game, faction, t) else {
        ui.colored_label(war_warning(), t.text("shrine-kinship-empty"));
        return;
    };

    ui.horizontal_wrapped(|ui| {
        ui.heading(egui::RichText::new(t.text("shrine-kinship-heading")).color(war_gold()));
        ui.separator();
        shrine_legend_item(
            ui,
            relationship_kind_color(RelationshipGraphKind::Spouse),
            t.text("shrine-kinship-legend-spouse"),
        );
        shrine_legend_item(
            ui,
            relationship_kind_color(RelationshipGraphKind::Child),
            t.text("shrine-kinship-legend-parent-child"),
        );
        shrine_legend_item(ui, war_text_muted(), t.text("shrine-kinship-legend-weak"));
    });
    ui.add_space(4.0);
    shrine_kinship_graph_view(ui, &graph, t, max_height.max(320.0));
    if graph.omitted_count > 0 {
        ui.colored_label(
            war_text_muted(),
            t.text_args(
                "shrine-kinship-omitted",
                &args([("count", graph.omitted_count.to_string())]),
            ),
        );
    }
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
    let marriage_entries = shrine_marriage_entries(game, &faction_id);
    let mut marriage_first = ui_state.shrine_marriage_first.clone();
    let mut marriage_second = ui_state.shrine_marriage_second.clone();
    let mut pending_divorce = ui_state.shrine_pending_divorce.clone();
    let mut marry_request = None;
    let mut divorce_request = None;

    StripBuilder::new(ui)
        .size(Size::relative(0.36))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                war_sub_panel_frame().show(ui, |ui| {
                    ui.heading(
                        egui::RichText::new(format!(
                            "{} {}",
                            egui_phosphor::regular::HEART,
                            t.text("shrine-marriage-create-heading")
                        ))
                        .color(war_gold()),
                    );
                    ui.label(t.text("shrine-marriage-create-note"));
                    ui.add_space(8.0);
                    officer_combo_for_marriage(
                        ui,
                        game,
                        &faction_id,
                        &mut marriage_first,
                        "shrine_marriage_first",
                        t.text("shrine-marriage-first"),
                    );
                    officer_combo_for_marriage(
                        ui,
                        game,
                        &faction_id,
                        &mut marriage_second,
                        "shrine_marriage_second",
                        t.text("shrine-marriage-second"),
                    );
                    ui.add_space(6.0);
                    if ui
                        .add_sized(
                            egui::vec2(ui.available_width(), 32.0),
                            egui::Button::new(format!(
                                "{} {}",
                                egui_phosphor::regular::HANDSHAKE,
                                t.text("shrine-marry")
                            )),
                        )
                        .clicked()
                        && let (Some(first_id), Some(second_id)) =
                            (marriage_first.clone(), marriage_second.clone())
                    {
                        marry_request = Some((first_id, second_id));
                    }
                });
            });
            strip.cell(|ui| {
                ui.heading(
                    egui::RichText::new(t.text("shrine-marriage-list-heading")).color(war_gold()),
                );
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("shrine_marriages")
                    .max_height(max_height.max(260.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if marriage_entries.is_empty() {
                            ui.colored_label(war_text_muted(), t.text("shrine-marriage-empty"));
                        }
                        for entry in &marriage_entries {
                            shrine_marriage_row(
                                ui,
                                t,
                                entry,
                                &mut pending_divorce,
                                &mut divorce_request,
                            );
                        }
                    });
            });
        });

    ui_state.shrine_marriage_first = marriage_first;
    ui_state.shrine_marriage_second = marriage_second;
    ui_state.shrine_pending_divorce = pending_divorce;

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
    if let Some((first_id, second_id)) = divorce_request
        && let Some(game) = ui_state.game.as_mut()
    {
        let first = officer_display_name(game, &first_id);
        let second = officer_display_name(game, &second_id);
        match divorce_officers(game, &faction_id, &first_id, &second_id) {
            Ok(()) => {
                ui_state.shrine_pending_divorce = None;
                ui_state.message = t.text_args(
                    "message-marriage-divorced",
                    &args([("first", first), ("second", second)]),
                );
            }
            Err(error) => ui_state.message = error.to_string(),
        }
    }
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

fn shrine_children_heir_panel(
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
    let Some(faction) = game.factions.get(&faction_id) else {
        ui.label(t.text("unknown-faction"));
        return;
    };
    let child_ids = child_ids_for_parent(game, &faction.ruler_id);
    let candidates = succession_candidate_ids(game, &faction_id, None);
    let mut set_heir_request = None;

    StripBuilder::new(ui)
        .size(Size::relative(0.52))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.heading(
                    egui::RichText::new(t.text("shrine-children-heading")).color(war_gold()),
                );
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("shrine_children_heir_children")
                    .max_height(max_height.max(260.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if child_ids.is_empty() {
                            ui.colored_label(war_text_muted(), t.text("shrine-children-empty"));
                        }
                        for child_id in &child_ids {
                            let Some(child) = game.officers.get(child_id) else {
                                continue;
                            };
                            shrine_child_card(ui, game, faction, child, t);
                        }
                    });
            });
            strip.cell(|ui| {
                ui.heading(
                    egui::RichText::new(t.text("shrine-succession-heading")).color(war_gold()),
                );
                ui.label(
                    t.text_args(
                        "shrine-current-ruler",
                        &args([
                            ("ruler", officer_display_name(game, &faction.ruler_id)),
                            (
                                "heir",
                                faction
                                    .heir_id
                                    .as_deref()
                                    .map(|id| officer_display_name(game, id))
                                    .unwrap_or_else(|| t.text("shrine-no-heir")),
                            ),
                        ]),
                    ),
                );
                ui.add_space(6.0);
                egui::ScrollArea::vertical()
                    .id_salt("shrine_succession_candidates")
                    .max_height((max_height - 42.0).max(240.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if candidates.is_empty() {
                            ui.colored_label(war_warning(), t.text("shrine-no-heir-candidates"));
                        }
                        for candidate_id in &candidates {
                            let Some(officer) = game.officers.get(candidate_id) else {
                                continue;
                            };
                            shrine_heir_candidate_row(
                                ui,
                                game,
                                faction,
                                officer,
                                t,
                                &mut set_heir_request,
                            );
                        }
                    });
            });
        });

    if let Some(candidate_id) = set_heir_request
        && let Some(game) = ui_state.game.as_mut()
    {
        match set_default_heir(game, &faction_id, &candidate_id) {
            Ok(()) => {
                let name = game.officers[&candidate_id].name.clone();
                ui_state.message = t.text_args("message-heir-set", &args([("officer", name)]));
            }
            Err(error) => ui_state.message = error.to_string(),
        }
    }
}

fn shrine_abdication_panel(
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
    let Some(faction) = game.factions.get(&faction_id) else {
        ui.label(t.text("unknown-faction"));
        return;
    };
    let candidates = succession_candidate_ids(game, &faction_id, Some(&faction.ruler_id));
    let selected_invalid = match &ui_state.shrine_abdication_successor {
        Some(selected_id) => !candidates
            .iter()
            .any(|candidate_id| candidate_id == selected_id),
        None => true,
    };
    if selected_invalid {
        ui_state.shrine_abdication_successor = candidates.first().cloned();
        ui_state.shrine_abdication_confirm = false;
    }

    let selected_id = ui_state.shrine_abdication_successor.clone();
    let mut abdication_request = None;
    StripBuilder::new(ui)
        .size(Size::relative(0.42))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                war_sub_panel_frame().show(ui, |ui| {
                    ui.heading(
                        egui::RichText::new(t.text("shrine-abdication-heading")).color(war_gold()),
                    );
                    ui.label(t.text("shrine-abdication-note"));
                    ui.add_space(8.0);
                    ui.label(t.text_args(
                        "shrine-abdication-current",
                        &args([("ruler", officer_display_name(game, &faction.ruler_id))]),
                    ));
                    egui::ComboBox::from_id_salt("shrine_abdication_successor")
                        .selected_text(
                            selected_id
                                .as_deref()
                                .map(|id| officer_display_name(game, id))
                                .unwrap_or_else(|| t.text("common-none-selected")),
                        )
                        .show_ui(ui, |ui| {
                            for candidate_id in &candidates {
                                ui.selectable_value(
                                    &mut ui_state.shrine_abdication_successor,
                                    Some(candidate_id.clone()),
                                    officer_display_name(game, candidate_id),
                                );
                            }
                        });
                    ui.add_space(10.0);
                    if let Some(successor_id) = &ui_state.shrine_abdication_successor {
                        ui.label(t.text_args(
                            "shrine-abdication-preview",
                            &args([
                                ("old", officer_display_name(game, &faction.ruler_id)),
                                ("new", officer_display_name(game, successor_id)),
                            ]),
                        ));
                    }
                    if candidates.is_empty() {
                        ui.colored_label(war_warning(), t.text("shrine-abdication-no-candidates"));
                    } else if ui_state.shrine_abdication_confirm {
                        ui.colored_label(war_danger(), t.text("shrine-abdication-confirm-warning"));
                        ui.horizontal(|ui| {
                            if ui.button(t.text("common-cancel")).clicked() {
                                ui_state.shrine_abdication_confirm = false;
                            }
                            if ui
                                .add(
                                    egui::Button::new(t.text("common-confirm")).fill(
                                        egui::Color32::from_rgba_unmultiplied(122, 45, 34, 220),
                                    ),
                                )
                                .clicked()
                                && let Some(successor_id) =
                                    ui_state.shrine_abdication_successor.clone()
                            {
                                abdication_request = Some(successor_id);
                            }
                        });
                    } else if ui
                        .add_sized(
                            egui::vec2(ui.available_width(), 32.0),
                            egui::Button::new(format!(
                                "{} {}",
                                egui_phosphor::regular::SEAL,
                                t.text("shrine-abdication-open-confirm")
                            )),
                        )
                        .clicked()
                    {
                        ui_state.shrine_abdication_confirm = true;
                    }
                });
            });
            strip.cell(|ui| {
                ui.heading(
                    egui::RichText::new(t.text("shrine-abdication-candidates")).color(war_gold()),
                );
                egui::ScrollArea::vertical()
                    .id_salt("shrine_abdication_candidates")
                    .max_height(max_height.max(260.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for candidate_id in &candidates {
                            let Some(officer) = game.officers.get(candidate_id) else {
                                continue;
                            };
                            let selected = ui_state.shrine_abdication_successor.as_deref()
                                == Some(candidate_id.as_str());
                            if shrine_selectable_officer_card(ui, game, officer, t, selected)
                                .clicked()
                            {
                                ui_state.shrine_abdication_successor = Some(candidate_id.clone());
                                ui_state.shrine_abdication_confirm = false;
                            }
                        }
                    });
            });
        });

    if let Some(successor_id) = abdication_request
        && let Some(game) = ui_state.game.as_mut()
    {
        let old_ruler = game
            .factions
            .get(&faction_id)
            .map(|faction| officer_display_name(game, &faction.ruler_id))
            .unwrap_or_else(|| t.text("unknown"));
        let successor = officer_display_name(game, &successor_id);
        match abdicate_ruler(game, &faction_id, &successor_id) {
            Ok(()) => {
                ui_state.shrine_abdication_confirm = false;
                ui_state.shrine_tab = ShrineTab::Kinship;
                ui_state.message = t.text_args(
                    "message-ruler-abdicated",
                    &args([("old", old_ruler), ("new", successor)]),
                );
            }
            Err(error) => ui_state.message = error.to_string(),
        }
    }
}

fn shrine_legend_item(ui: &mut egui::Ui, color: egui::Color32, label: String) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 10.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.left_center(), rect.right_center()],
        egui::Stroke::new(2.0, color),
    );
    ui.label(egui::RichText::new(label).color(war_text_muted()));
}

fn shrine_badge(ui: &mut egui::Ui, icon: &str, label: String, color: egui::Color32) {
    let fill = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 36);
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(6, 3))
        .fill(fill)
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 130),
        ))
        .corner_radius(4)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!("{icon} {label}"))
                    .size(12.0)
                    .color(color),
            );
        });
}

fn shrine_source_badge(ui: &mut egui::Ui, t: &Translator, source: ShrineRelationshipSource) {
    let (icon, color) = match source {
        ShrineRelationshipSource::Dynamic => (
            egui_phosphor::regular::GAME_CONTROLLER,
            egui::Color32::from_rgb(120, 178, 211),
        ),
        ShrineRelationshipSource::Historical => (
            egui_phosphor::regular::SCROLL,
            egui::Color32::from_rgb(190, 151, 88),
        ),
        ShrineRelationshipSource::Mixed => (
            egui_phosphor::regular::SPARKLE,
            egui::Color32::from_rgb(217, 126, 118),
        ),
    };
    shrine_badge(ui, icon, shrine_relationship_source_label(t, source), color);
}

fn shrine_gender_icon(gender: &OfficerGender) -> (&'static str, egui::Color32) {
    match gender {
        OfficerGender::Male => (
            egui_phosphor::regular::GENDER_MALE,
            egui::Color32::from_rgb(120, 178, 211),
        ),
        OfficerGender::Female => (
            egui_phosphor::regular::GENDER_FEMALE,
            egui::Color32::from_rgb(217, 126, 118),
        ),
    }
}

fn shrine_status_color(status: &OfficerStatus) -> egui::Color32 {
    match status {
        OfficerStatus::Active => war_success(),
        OfficerStatus::Minor => egui::Color32::from_rgb(120, 178, 211),
        OfficerStatus::Wild => war_warning(),
        OfficerStatus::Unavailable => war_text_muted(),
        OfficerStatus::Dead => war_danger(),
    }
}

const SHRINE_MAX_KINSHIP_NODES: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShrineKinshipGraph {
    nodes: Vec<ShrineKinshipNode>,
    edges: Vec<ShrineKinshipEdge>,
    omitted_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShrineKinshipNode {
    id: OfficerId,
    name: String,
    gender: OfficerGender,
    generation: i32,
    tooltip: String,
    role: ShrineKinshipNodeRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShrineKinshipNodeRole {
    Ruler,
    Heir,
    Officer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShrineKinshipEdge {
    from_id: OfficerId,
    to_id: OfficerId,
    kind: ShrineKinshipEdgeKind,
    source: ShrineRelationshipSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ShrineKinshipEdgeKind {
    Spouse,
    ParentChild,
    WeakParentChild,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShrineRelationshipSource {
    Dynamic,
    Historical,
    Mixed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ShrineMarriageEntry {
    husband_id: OfficerId,
    wife_id: OfficerId,
    husband_name: String,
    wife_name: String,
    date: Option<(i32, u8)>,
    source: ShrineRelationshipSource,
}

fn shrine_kinship_graph(
    game: &GameState,
    faction: &Faction,
    t: &Translator,
) -> Option<ShrineKinshipGraph> {
    if !game.officers.contains_key(&faction.ruler_id) {
        return None;
    }
    let all_edges = shrine_all_kinship_edges(game);
    let connected_order = shrine_connected_kinship_order(&faction.ruler_id, &all_edges);
    let omitted_count = connected_order
        .len()
        .saturating_sub(SHRINE_MAX_KINSHIP_NODES);
    let selected_ids = connected_order
        .into_iter()
        .take(SHRINE_MAX_KINSHIP_NODES)
        .collect::<BTreeSet<_>>();
    let edges = all_edges
        .into_iter()
        .filter(|edge| selected_ids.contains(&edge.from_id) && selected_ids.contains(&edge.to_id))
        .collect::<Vec<_>>();
    let generations = shrine_kinship_generations(&faction.ruler_id, &selected_ids, &edges);
    let nodes = selected_ids
        .into_iter()
        .filter_map(|officer_id| {
            let officer = game.officers.get(&officer_id)?;
            Some(ShrineKinshipNode {
                id: officer.id.clone(),
                name: officer.name.clone(),
                gender: officer.gender.clone(),
                generation: *generations.get(&officer.id).unwrap_or(&0),
                tooltip: shrine_officer_tooltip(game, faction, officer, t),
                role: if faction.ruler_id == officer.id {
                    ShrineKinshipNodeRole::Ruler
                } else if faction.heir_id.as_deref() == Some(officer.id.as_str()) {
                    ShrineKinshipNodeRole::Heir
                } else {
                    ShrineKinshipNodeRole::Officer
                },
            })
        })
        .collect();
    Some(ShrineKinshipGraph {
        nodes,
        edges,
        omitted_count,
    })
}

fn shrine_all_kinship_edges(game: &GameState) -> Vec<ShrineKinshipEdge> {
    let mut edges =
        BTreeMap::<(ShrineKinshipEdgeKind, OfficerId, OfficerId), ShrineRelationshipSource>::new();

    for marriage in &game.marriages {
        let (first, second) = sorted_pair(&marriage.husband_id, &marriage.wife_id);
        shrine_insert_edge_source(
            &mut edges,
            ShrineKinshipEdgeKind::Spouse,
            first,
            second,
            ShrineRelationshipSource::Dynamic,
        );
    }
    for relationship in &game.family_relationships {
        if game.officers.contains_key(&relationship.parent_id)
            && game.officers.contains_key(&relationship.child_id)
        {
            shrine_insert_edge_source(
                &mut edges,
                ShrineKinshipEdgeKind::ParentChild,
                relationship.parent_id.clone(),
                relationship.child_id.clone(),
                ShrineRelationshipSource::Dynamic,
            );
        }
    }
    for officer in game.officers.values() {
        let Some(profile) = &officer.profile else {
            continue;
        };
        for relationship in &profile.relationships {
            if !game.officers.contains_key(&relationship.target_id) {
                continue;
            }
            match relationship.kind {
                OfficerRelationshipKind::Spouse => {
                    let (first, second) = sorted_pair(&officer.id, &relationship.target_id);
                    shrine_insert_edge_source(
                        &mut edges,
                        ShrineKinshipEdgeKind::Spouse,
                        first,
                        second,
                        ShrineRelationshipSource::Historical,
                    );
                }
                OfficerRelationshipKind::ParentChild
                | OfficerRelationshipKind::AdoptiveParentChild => {
                    if let Some((parent_id, child_id)) = historical_parent_child_direction(
                        game,
                        &officer.id,
                        &relationship.target_id,
                    ) {
                        shrine_insert_edge_source(
                            &mut edges,
                            ShrineKinshipEdgeKind::ParentChild,
                            parent_id,
                            child_id,
                            ShrineRelationshipSource::Historical,
                        );
                    } else {
                        let (first, second) = sorted_pair(&officer.id, &relationship.target_id);
                        shrine_insert_edge_source(
                            &mut edges,
                            ShrineKinshipEdgeKind::WeakParentChild,
                            first,
                            second,
                            ShrineRelationshipSource::Historical,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    edges
        .into_iter()
        .map(|((kind, from_id, to_id), source)| ShrineKinshipEdge {
            from_id,
            to_id,
            kind,
            source,
        })
        .collect()
}

fn shrine_insert_edge_source(
    edges: &mut BTreeMap<(ShrineKinshipEdgeKind, OfficerId, OfficerId), ShrineRelationshipSource>,
    kind: ShrineKinshipEdgeKind,
    from_id: OfficerId,
    to_id: OfficerId,
    source: ShrineRelationshipSource,
) {
    edges
        .entry((kind, from_id, to_id))
        .and_modify(|existing| *existing = merge_relationship_source(*existing, source))
        .or_insert(source);
}

fn merge_relationship_source(
    first: ShrineRelationshipSource,
    second: ShrineRelationshipSource,
) -> ShrineRelationshipSource {
    if first == second {
        first
    } else {
        ShrineRelationshipSource::Mixed
    }
}

fn shrine_connected_kinship_order(ruler_id: &str, edges: &[ShrineKinshipEdge]) -> Vec<OfficerId> {
    let mut adjacency = BTreeMap::<OfficerId, BTreeSet<OfficerId>>::new();
    for edge in edges {
        adjacency
            .entry(edge.from_id.clone())
            .or_default()
            .insert(edge.to_id.clone());
        adjacency
            .entry(edge.to_id.clone())
            .or_default()
            .insert(edge.from_id.clone());
    }

    let mut queue = VecDeque::from([ruler_id.to_string()]);
    let mut seen = BTreeSet::new();
    let mut order = Vec::new();
    while let Some(officer_id) = queue.pop_front() {
        if !seen.insert(officer_id.clone()) {
            continue;
        }
        order.push(officer_id.clone());
        if let Some(neighbors) = adjacency.get(&officer_id) {
            for neighbor_id in neighbors {
                if !seen.contains(neighbor_id) {
                    queue.push_back(neighbor_id.clone());
                }
            }
        }
    }
    order
}

fn shrine_kinship_generations(
    ruler_id: &str,
    selected_ids: &BTreeSet<OfficerId>,
    edges: &[ShrineKinshipEdge],
) -> BTreeMap<OfficerId, i32> {
    let mut generations = BTreeMap::from([(ruler_id.to_string(), 0)]);
    for _ in 0..selected_ids.len().max(1) {
        let mut changed = false;
        for edge in edges {
            match edge.kind {
                ShrineKinshipEdgeKind::Spouse => {
                    changed |=
                        shrine_copy_generation(&mut generations, &edge.from_id, &edge.to_id, 0);
                    changed |=
                        shrine_copy_generation(&mut generations, &edge.to_id, &edge.from_id, 0);
                }
                ShrineKinshipEdgeKind::ParentChild => {
                    changed |=
                        shrine_copy_generation(&mut generations, &edge.from_id, &edge.to_id, 1);
                    changed |=
                        shrine_copy_generation(&mut generations, &edge.to_id, &edge.from_id, -1);
                }
                ShrineKinshipEdgeKind::WeakParentChild => {}
            }
        }
        if !changed {
            break;
        }
    }
    for selected_id in selected_ids {
        generations.entry(selected_id.clone()).or_insert(0);
    }
    generations
}

fn shrine_copy_generation(
    generations: &mut BTreeMap<OfficerId, i32>,
    known_id: &str,
    unknown_id: &str,
    delta: i32,
) -> bool {
    let Some(known_generation) = generations.get(known_id).copied() else {
        return false;
    };
    if generations.contains_key(unknown_id) {
        return false;
    }
    generations.insert(unknown_id.to_string(), known_generation + delta);
    true
}

fn historical_parent_child_direction(
    game: &GameState,
    first_id: &str,
    second_id: &str,
) -> Option<(OfficerId, OfficerId)> {
    let first = game.officers.get(first_id)?;
    let second = game.officers.get(second_id)?;
    let first_age = first.age_at(game.year);
    let second_age = second.age_at(game.year);
    if first_age > second_age {
        Some((first.id.clone(), second.id.clone()))
    } else if second_age > first_age {
        Some((second.id.clone(), first.id.clone()))
    } else {
        None
    }
}

fn shrine_kinship_graph_view(
    ui: &mut egui::Ui,
    graph: &ShrineKinshipGraph,
    t: &Translator,
    max_height: f32,
) {
    let desired_size = egui::vec2(ui.available_width().max(560.0), max_height.max(320.0));
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        5.0,
        egui::Color32::from_rgba_unmultiplied(18, 15, 11, 160),
    );
    painter.rect_stroke(
        rect.shrink(1.0),
        5.0,
        egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(138, 101, 58, 110),
        ),
        egui::StrokeKind::Inside,
    );

    if graph.nodes.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            t.text("shrine-kinship-empty"),
            egui::FontId::proportional(14.0),
            war_text_muted(),
        );
        return;
    }

    let mut by_generation = BTreeMap::<i32, Vec<&ShrineKinshipNode>>::new();
    for node in &graph.nodes {
        by_generation.entry(node.generation).or_default().push(node);
    }
    for nodes in by_generation.values_mut() {
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
    }
    let max_nodes_in_row = by_generation
        .values()
        .map(|nodes| nodes.len())
        .max()
        .unwrap_or(1);
    let node_size = egui::vec2(
        ((rect.width() - 48.0) / max_nodes_in_row as f32 - 10.0).clamp(74.0, 116.0),
        54.0,
    );
    let graph_rect = rect.shrink2(egui::vec2(22.0, 20.0));
    let row_count = by_generation.len().max(1);
    let mut positions = BTreeMap::<OfficerId, egui::Pos2>::new();
    for (row_index, (_generation, nodes)) in by_generation.iter().enumerate() {
        let y =
            graph_rect.top() + graph_rect.height() * (row_index as f32 + 0.5) / row_count as f32;
        let count = nodes.len().max(1);
        for (column_index, node) in nodes.iter().enumerate() {
            let x =
                graph_rect.left() + graph_rect.width() * (column_index as f32 + 0.5) / count as f32;
            positions.insert(node.id.clone(), egui::pos2(x, y));
        }
    }

    for edge in &graph.edges {
        let (Some(from), Some(to)) = (positions.get(&edge.from_id), positions.get(&edge.to_id))
        else {
            continue;
        };
        let color = shrine_kinship_edge_color(edge.kind);
        let stroke_width = if edge.kind == ShrineKinshipEdgeKind::WeakParentChild {
            1.0
        } else {
            2.0
        };
        painter.line_segment([*from, *to], egui::Stroke::new(stroke_width, color));
        let label = match edge.kind {
            ShrineKinshipEdgeKind::Spouse => egui_phosphor::regular::HEART,
            ShrineKinshipEdgeKind::ParentChild => egui_phosphor::regular::TREE_STRUCTURE,
            ShrineKinshipEdgeKind::WeakParentChild => egui_phosphor::regular::CIRCLE_NOTCH,
        };
        painter.text(
            from.lerp(*to, 0.5),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(14.0),
            color,
        );
    }

    for node in &graph.nodes {
        if let Some(position) = positions.get(&node.id) {
            draw_shrine_kinship_node(ui, &painter, *position, node_size, node);
        }
    }
}

fn shrine_kinship_edge_color(kind: ShrineKinshipEdgeKind) -> egui::Color32 {
    match kind {
        ShrineKinshipEdgeKind::Spouse => relationship_kind_color(RelationshipGraphKind::Spouse),
        ShrineKinshipEdgeKind::ParentChild => relationship_kind_color(RelationshipGraphKind::Child),
        ShrineKinshipEdgeKind::WeakParentChild => war_text_muted(),
    }
}

fn draw_shrine_kinship_node(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    center: egui::Pos2,
    size: egui::Vec2,
    node: &ShrineKinshipNode,
) {
    let rect = egui::Rect::from_center_size(center, size);
    let response = ui.interact(
        rect,
        egui::Id::new(("shrine_kinship_node", node.id.clone())),
        egui::Sense::hover(),
    );
    if response.hovered() {
        response.on_hover_text(node.tooltip.clone());
    }
    let (icon, stroke_color) = match node.role {
        ShrineKinshipNodeRole::Ruler => (egui_phosphor::regular::CROWN, war_gold()),
        ShrineKinshipNodeRole::Heir => (egui_phosphor::regular::SEAL_CHECK, war_success()),
        ShrineKinshipNodeRole::Officer => (egui_phosphor::regular::USER, war_border()),
    };
    let fill = match node.role {
        ShrineKinshipNodeRole::Ruler => egui::Color32::from_rgba_unmultiplied(65, 43, 22, 238),
        ShrineKinshipNodeRole::Heir => egui::Color32::from_rgba_unmultiplied(31, 55, 36, 238),
        ShrineKinshipNodeRole::Officer => egui::Color32::from_rgba_unmultiplied(35, 29, 22, 235),
    };
    painter.rect_filled(rect, 5.0, fill);
    painter.rect_stroke(
        rect,
        5.0,
        egui::Stroke::new(1.4, stroke_color),
        egui::StrokeKind::Inside,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 13.0),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(15.0),
        stroke_color,
    );
    let (gender_icon, gender_color) = shrine_gender_icon(&node.gender);
    painter.text(
        egui::pos2(rect.right() - 10.0, rect.top() + 12.0),
        egui::Align2::CENTER_CENTER,
        gender_icon,
        egui::FontId::proportional(12.0),
        gender_color,
    );
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 18.0),
        egui::Align2::CENTER_CENTER,
        compact_node_label(&node.name),
        egui::FontId::proportional(13.0),
        war_text(),
    );
}

fn shrine_officer_tooltip(
    game: &GameState,
    faction: &Faction,
    officer: &Officer,
    t: &Translator,
) -> String {
    let role = if faction.ruler_id == officer.id {
        t.text("shrine-role-ruler")
    } else if faction.heir_id.as_deref() == Some(officer.id.as_str()) {
        t.text("shrine-role-heir")
    } else {
        t.text("shrine-role-officer")
    };
    t.text_args(
        "shrine-officer-tooltip",
        &args([
            ("name", officer.name.clone()),
            ("role", role),
            ("age", officer.age_at(game.year).to_string()),
            ("status", officer_status_label(&officer.status, t)),
            ("loyalty", officer.loyalty.to_string()),
        ]),
    )
}

fn shrine_marriage_entries(game: &GameState, faction_id: &str) -> Vec<ShrineMarriageEntry> {
    let mut entries = BTreeMap::<(OfficerId, OfficerId), ShrineMarriageEntry>::new();
    for marriage in &game.marriages {
        if !marriage_relevant_to_faction(game, faction_id, &marriage.husband_id, &marriage.wife_id)
        {
            continue;
        }
        let key = marriage_pair_ids(game, &marriage.husband_id, &marriage.wife_id);
        entries
            .entry(key.clone())
            .and_modify(|entry| {
                entry.source =
                    merge_relationship_source(entry.source, ShrineRelationshipSource::Dynamic);
                entry.date.get_or_insert((marriage.year, marriage.month));
            })
            .or_insert_with(|| ShrineMarriageEntry {
                husband_id: key.0.clone(),
                wife_id: key.1.clone(),
                husband_name: officer_display_name(game, &key.0),
                wife_name: officer_display_name(game, &key.1),
                date: Some((marriage.year, marriage.month)),
                source: ShrineRelationshipSource::Dynamic,
            });
    }
    for officer in game.officers.values() {
        let Some(profile) = &officer.profile else {
            continue;
        };
        for relationship in profile.relationships.iter().filter(|relationship| {
            relationship.kind == OfficerRelationshipKind::Spouse
                && game.officers.contains_key(&relationship.target_id)
                && marriage_relevant_to_faction(
                    game,
                    faction_id,
                    &officer.id,
                    &relationship.target_id,
                )
        }) {
            let key = marriage_pair_ids(game, &officer.id, &relationship.target_id);
            entries
                .entry(key.clone())
                .and_modify(|entry| {
                    entry.source = merge_relationship_source(
                        entry.source,
                        ShrineRelationshipSource::Historical,
                    );
                })
                .or_insert_with(|| ShrineMarriageEntry {
                    husband_id: key.0.clone(),
                    wife_id: key.1.clone(),
                    husband_name: officer_display_name(game, &key.0),
                    wife_name: officer_display_name(game, &key.1),
                    date: None,
                    source: ShrineRelationshipSource::Historical,
                });
        }
    }
    entries.into_values().collect()
}

fn marriage_relevant_to_faction(
    game: &GameState,
    faction_id: &str,
    first_id: &str,
    second_id: &str,
) -> bool {
    game.officers
        .get(first_id)
        .is_some_and(|officer| officer.faction_id == faction_id)
        || game
            .officers
            .get(second_id)
            .is_some_and(|officer| officer.faction_id == faction_id)
}

fn marriage_pair_ids(game: &GameState, first_id: &str, second_id: &str) -> (OfficerId, OfficerId) {
    let first = game.officers.get(first_id);
    let second = game.officers.get(second_id);
    match (
        first.map(|officer| officer.gender.clone()),
        second.map(|officer| officer.gender.clone()),
    ) {
        (Some(OfficerGender::Male), Some(OfficerGender::Female)) => {
            (first_id.to_string(), second_id.to_string())
        }
        (Some(OfficerGender::Female), Some(OfficerGender::Male)) => {
            (second_id.to_string(), first_id.to_string())
        }
        _ => sorted_pair(first_id, second_id),
    }
}

fn sorted_pair(first_id: &str, second_id: &str) -> (OfficerId, OfficerId) {
    if first_id <= second_id {
        (first_id.to_string(), second_id.to_string())
    } else {
        (second_id.to_string(), first_id.to_string())
    }
}

fn shrine_marriage_row(
    ui: &mut egui::Ui,
    t: &Translator,
    entry: &ShrineMarriageEntry,
    pending_divorce: &mut Option<(OfficerId, OfficerId)>,
    divorce_request: &mut Option<(OfficerId, OfficerId)>,
) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(egui_phosphor::regular::HEART)
                    .size(18.0)
                    .color(relationship_kind_color(RelationshipGraphKind::Spouse)),
            );
            ui.vertical(|ui| {
                let husband_gender = shrine_gender_icon(&OfficerGender::Male);
                let wife_gender = shrine_gender_icon(&OfficerGender::Female);
                ui.label(
                    egui::RichText::new(format!(
                        "{} {}  -  {} {}",
                        husband_gender.0, entry.husband_name, wife_gender.0, entry.wife_name
                    ))
                    .strong(),
                );
                let date = entry
                    .date
                    .map(|(year, month)| {
                        t.text_args(
                            "shrine-marriage-date",
                            &args([("year", year.to_string()), ("month", month.to_string())]),
                        )
                    })
                    .unwrap_or_else(|| t.text("shrine-marriage-date-historical"));
                ui.horizontal_wrapped(|ui| {
                    shrine_source_badge(ui, t, entry.source);
                    shrine_badge(
                        ui,
                        if entry.date.is_some() {
                            egui_phosphor::regular::CALENDAR_HEART
                        } else {
                            egui_phosphor::regular::SCROLL
                        },
                        date,
                        if entry.date.is_some() {
                            relationship_kind_color(RelationshipGraphKind::Spouse)
                        } else {
                            war_text_muted()
                        },
                    );
                });
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let pair = (entry.husband_id.clone(), entry.wife_id.clone());
                if pending_divorce.as_ref() == Some(&pair) {
                    if square_icon_button(
                        ui,
                        egui_phosphor::regular::CHECK,
                        t.text("common-confirm"),
                        egui::vec2(30.0, 30.0),
                    )
                    .clicked()
                    {
                        *divorce_request = Some(pair);
                    }
                    if square_icon_button(
                        ui,
                        egui_phosphor::regular::X,
                        t.text("common-cancel"),
                        egui::vec2(30.0, 30.0),
                    )
                    .clicked()
                    {
                        *pending_divorce = None;
                    }
                    ui.colored_label(war_danger(), t.text("shrine-marriage-confirm-divorce"));
                } else if square_icon_button(
                    ui,
                    egui_phosphor::regular::HEART_BREAK,
                    t.text("shrine-divorce"),
                    egui::vec2(30.0, 30.0),
                )
                .clicked()
                {
                    *pending_divorce = Some(pair);
                }
            });
        });
    });
}

fn shrine_relationship_source_label(t: &Translator, source: ShrineRelationshipSource) -> String {
    match source {
        ShrineRelationshipSource::Dynamic => t.text("shrine-source-dynamic"),
        ShrineRelationshipSource::Historical => t.text("shrine-source-historical"),
        ShrineRelationshipSource::Mixed => t.text("shrine-source-mixed"),
    }
}

fn shrine_child_card(
    ui: &mut egui::Ui,
    game: &GameState,
    faction: &Faction,
    child: &Officer,
    t: &Translator,
) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            let is_heir = faction.heir_id.as_deref() == Some(child.id.as_str());
            ui.label(
                egui::RichText::new(if is_heir {
                    egui_phosphor::regular::CROWN
                } else {
                    egui_phosphor::regular::BABY
                })
                .color(if is_heir { war_gold() } else { war_success() }),
            );
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&child.name).strong());
                ui.horizontal_wrapped(|ui| {
                    shrine_badge(
                        ui,
                        egui_phosphor::regular::CLOCK,
                        t.text_args(
                            "shrine-chip-age",
                            &args([("age", child.age_at(game.year).to_string())]),
                        ),
                        egui::Color32::from_rgb(120, 178, 211),
                    );
                    shrine_badge(
                        ui,
                        egui_phosphor::regular::USER_CHECK,
                        officer_status_label(&child.status, t),
                        shrine_status_color(&child.status),
                    );
                    shrine_badge(
                        ui,
                        egui_phosphor::regular::USERS_THREE,
                        t.text_args(
                            "shrine-chip-parents",
                            &args([("parents", shrine_parent_names(game, &child.id, t))]),
                        ),
                        war_text_muted(),
                    );
                });
            });
        });
    });
}

fn shrine_heir_candidate_row(
    ui: &mut egui::Ui,
    game: &GameState,
    faction: &Faction,
    officer: &Officer,
    t: &Translator,
    set_heir_request: &mut Option<OfficerId>,
) {
    war_sub_panel_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            let selected = faction.heir_id.as_deref() == Some(officer.id.as_str());
            ui.label(
                egui::RichText::new(if selected {
                    egui_phosphor::regular::CROWN
                } else {
                    egui_phosphor::regular::USER_CIRCLE
                })
                .color(if selected {
                    war_gold()
                } else {
                    war_text_muted()
                }),
            );
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&officer.name).strong());
                ui.horizontal_wrapped(|ui| {
                    shrine_badge(
                        ui,
                        egui_phosphor::regular::CLOCK,
                        t.text_args(
                            "shrine-chip-age",
                            &args([("age", officer.age_at(game.year).to_string())]),
                        ),
                        egui::Color32::from_rgb(120, 178, 211),
                    );
                    shrine_badge(
                        ui,
                        egui_phosphor::regular::MEDAL,
                        t.text_args(
                            "shrine-chip-loyalty",
                            &args([("loyalty", officer.loyalty.to_string())]),
                        ),
                        war_gold(),
                    );
                    shrine_badge(
                        ui,
                        egui_phosphor::regular::USER_CHECK,
                        officer_status_label(&officer.status, t),
                        shrine_status_color(&officer.status),
                    );
                });
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if square_icon_button(
                    ui,
                    egui_phosphor::regular::CROWN,
                    t.text("shrine-set-heir"),
                    egui::vec2(30.0, 30.0),
                )
                .clicked()
                {
                    *set_heir_request = Some(officer.id.clone());
                }
            });
        });
    });
}

fn shrine_selectable_officer_card(
    ui: &mut egui::Ui,
    game: &GameState,
    officer: &Officer,
    t: &Translator,
    selected: bool,
) -> egui::Response {
    let label = format!(
        "{}  {}   {} {}   {} {}   {} {}",
        if selected {
            egui_phosphor::regular::SEAL_CHECK
        } else {
            egui_phosphor::regular::USER
        },
        officer.name,
        egui_phosphor::regular::CLOCK,
        t.text_args(
            "shrine-chip-age",
            &args([("age", officer.age_at(game.year).to_string())]),
        ),
        egui_phosphor::regular::MEDAL,
        t.text_args(
            "shrine-chip-loyalty",
            &args([("loyalty", officer.loyalty.to_string())]),
        ),
        egui_phosphor::regular::USER_CHECK,
        officer_status_label(&officer.status, t),
    );
    ui.add_sized(
        egui::vec2(ui.available_width(), 36.0),
        egui::Button::new(egui::RichText::new(label).color(if selected {
            war_gold()
        } else {
            war_text()
        }))
        .selected(selected)
        .fill(if selected {
            egui::Color32::from_rgba_unmultiplied(84, 54, 30, 190)
        } else {
            egui::Color32::from_rgba_unmultiplied(38, 31, 23, 120)
        }),
    )
}

fn shrine_parent_names(game: &GameState, child_id: &str, t: &Translator) -> String {
    let mut parent_ids = BTreeSet::new();
    for relationship in &game.family_relationships {
        if relationship.child_id == child_id {
            parent_ids.insert(relationship.parent_id.clone());
        }
    }
    for officer in game.officers.values() {
        let Some(profile) = &officer.profile else {
            continue;
        };
        for relationship in &profile.relationships {
            if !matches!(
                relationship.kind,
                OfficerRelationshipKind::ParentChild | OfficerRelationshipKind::AdoptiveParentChild
            ) {
                continue;
            }
            if let Some((parent_id, historical_child_id)) =
                historical_parent_child_direction(game, &officer.id, &relationship.target_id)
                && historical_child_id == child_id
            {
                parent_ids.insert(parent_id);
            }
        }
    }
    let names = parent_ids
        .iter()
        .map(|parent_id| officer_display_name(game, parent_id))
        .collect::<Vec<_>>();
    if names.is_empty() {
        t.text("common-none-selected")
    } else {
        names.join("、")
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{add_static_spouse_and_child, ui_state_with_game, zh};
    use super::*;
    #[test]
    fn shrine_kinship_graph_merges_historical_and_dynamic_family_links() {
        let mut state = ui_state_with_game();
        {
            let game = state.game.as_mut().unwrap();
            add_static_spouse_and_child(game);
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
        }
        let game = state.game.as_ref().unwrap();
        let graph = shrine_kinship_graph(game, &game.factions["liu_bei"], &zh()).unwrap();

        assert!(graph.nodes.iter().any(|node| node.id == "lady_static"));
        assert!(graph.nodes.iter().any(|node| node.id == "liu_static_child"));
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.kind == ShrineKinshipEdgeKind::Spouse
                    && edge.source == ShrineRelationshipSource::Historical
                    && edge.from_id == "lady_static"
                    && edge.to_id == "liu_bei")
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.kind == ShrineKinshipEdgeKind::Spouse
                    && edge.source == ShrineRelationshipSource::Dynamic
                    && edge.from_id == "liu_bei"
                    && edge.to_id == "zhang_fei")
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.kind == ShrineKinshipEdgeKind::ParentChild
                    && edge.from_id == "liu_bei"
                    && edge.to_id == "liu_static_child")
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|edge| edge.kind == ShrineKinshipEdgeKind::ParentChild
                    && edge.source == ShrineRelationshipSource::Dynamic
                    && edge.from_id == "liu_bei"
                    && edge.to_id == "zhao_yun")
        );
    }

    #[test]
    fn shrine_kinship_graph_deduplicates_bidirectional_historical_relationships() {
        let mut state = ui_state_with_game();
        add_static_spouse_and_child(state.game.as_mut().unwrap());
        let game = state.game.as_ref().unwrap();
        let graph = shrine_kinship_graph(game, &game.factions["liu_bei"], &zh()).unwrap();

        let spouse_edges = graph
            .edges
            .iter()
            .filter(|edge| {
                edge.kind == ShrineKinshipEdgeKind::Spouse
                    && edge.from_id == "lady_static"
                    && edge.to_id == "liu_bei"
            })
            .count();
        let parent_child_edges = graph
            .edges
            .iter()
            .filter(|edge| {
                edge.kind == ShrineKinshipEdgeKind::ParentChild
                    && edge.from_id == "liu_bei"
                    && edge.to_id == "liu_static_child"
            })
            .count();

        assert_eq!(spouse_edges, 1);
        assert_eq!(parent_child_edges, 1);
    }
}
