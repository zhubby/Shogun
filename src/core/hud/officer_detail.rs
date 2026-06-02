use crate::game::*;
use bevy_egui::egui;

use super::super::i18n::{Translator, args};
use super::super::labels::{confidence_label, officer_gender_label, officer_relationship_label};
use super::super::officer_portrait_ui::{
    officer_portrait_status_line, paint_officer_portrait_preview,
};
use super::super::portraits::{
    OFFICER_PORTRAIT_ASPECT_HEIGHT, OFFICER_PORTRAIT_ASPECT_WIDTH, OfficerPortraitStore,
    OfficerPortraitTaskState, officer_portrait_path,
};
use super::super::runtime::CoreAsyncRuntime;
use super::super::state::{GameUiState, OfficerEditDraft};
use super::super::style::{
    modal_title_bar, war_danger, war_gold, war_panel_frame, war_sub_panel_frame, war_success,
    war_text, war_text_muted, war_warning,
};
use super::officer_browser::officer_tag_display_text;
use super::officer_common::{
    RelationshipGraphKind, compact_node_label, faction_name, officer_display_name,
    officer_status_label, relationship_kind_color,
};
const OFFICER_DETAIL_PORTRAIT_WIDTH: f32 = 224.0;

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

pub(in crate::core) struct OfficerPortraitModalContext<'a> {
    pub(in crate::core) store: &'a mut OfficerPortraitStore,
    pub(in crate::core) api_key: &'a str,
    pub(in crate::core) model_name: &'a str,
    pub(in crate::core) async_runtime: &'a CoreAsyncRuntime,
}

pub(in crate::core) fn officer_detail_modal_for_game(
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

#[cfg(test)]
mod tests {
    use super::super::test_support::{ui_state_with_game, zh};
    use super::*;
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
