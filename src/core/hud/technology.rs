use crate::game::*;
use bevy_egui::egui;
use std::collections::BTreeSet;

use super::super::i18n::{Translator, args};
use super::super::labels::technology_branch_label;
use super::super::state::GameUiState;
use super::super::style::{
    modal_title_bar, war_border, war_danger, war_gold, war_panel_frame, war_sub_panel_frame,
    war_success, war_text_muted, war_warning,
};
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
    if game.technology_catalog.is_empty() {
        ui.label(t.text("technology-catalog-empty"));
        return;
    }
    let branch = ui_state.selected_technology_branch;
    if !game
        .technology_catalog
        .spec(&ui_state.selected_technology_id)
        .is_some_and(|spec| spec.branch == branch)
    {
        ui_state.selected_technology_id = game
            .technology_catalog
            .first_id_for_branch(branch)
            .cloned()
            .unwrap_or_default();
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
                ui_state.selected_technology_id = game
                    .technology_catalog
                    .first_id_for_branch(branch)
                    .cloned()
                    .unwrap_or_else(|| ui_state.selected_technology_id.clone());
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
            for spec in game
                .technology_catalog
                .specs_for_branch(ui_state.selected_technology_branch)
            {
                let selected = ui_state.selected_technology_id == spec.id;
                let status = technology_node_status(game, faction_state, spec);
                let response = technology_tree_node(
                    ui,
                    &game.technology_catalog,
                    spec,
                    selected,
                    status,
                    faction_state,
                    t,
                );
                if response.clicked() {
                    ui_state.selected_technology_id = spec.id.clone();
                }
                response.on_hover_text(spec.effect.as_str());
                ui.add_space(7.0);
            }
        });
}

fn technology_tree_node(
    ui: &mut egui::Ui,
    catalog: &TechnologyCatalog,
    spec: &TechnologySpec,
    selected: bool,
    status: TechnologyNodeStatus,
    faction_state: Option<&FactionTechnologyState>,
    t: &Translator,
) -> egui::Response {
    const ROW_HEIGHT: f32 = 48.0;
    let depth = technology_depth(spec, catalog);
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
        spec.name.as_str(),
        egui::FontId::proportional(18.0),
        if status == TechnologyNodeStatus::Locked {
            war_text_muted()
        } else {
            egui::Color32::from_rgb(238, 225, 193)
        },
    );

    let progress = faction_state
        .map(|state| technology_progress(state, &spec.id))
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
    let Some(spec) = game
        .technology_catalog
        .spec(&ui_state.selected_technology_id)
    else {
        ui.label(t.text("technology-catalog-empty"));
        return;
    };
    let faction_state = faction_technology_state(game, &faction_id);
    let progress = faction_state
        .map(|state| technology_progress(state, &spec.id))
        .unwrap_or_default();
    let total_gold = faction_total_gold(game, &faction_id);
    let cost = effective_technology_cost(game, &faction_id, &spec.id).unwrap_or(spec.gold_cost);
    let missing = missing_prerequisite_names(faction_state, &game.technology_catalog, &spec.id)
        .unwrap_or_default();
    let is_completed = faction_state.is_some_and(|state| state.completed.contains(&spec.id));
    let is_funded = faction_state.is_some_and(|state| state.funded.contains(&spec.id));
    let is_active =
        faction_state.is_some_and(|state| state.active.as_deref() == Some(spec.id.as_str()));

    ui.heading(egui::RichText::new(spec.name.as_str()).color(war_gold()));
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
                &args([(
                    "names",
                    prerequisite_names(spec, &game.technology_catalog).join("、"),
                )]),
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
    ui.colored_label(war_text_muted(), spec.effect.as_str());
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
            start_player_research(ui_state, spec.id.clone(), t);
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
        start_player_research(ui_state, spec.id.clone(), t);
    }
}

fn start_player_research(ui_state: &mut GameUiState, technology_id: TechnologyId, t: &Translator) {
    let Some(game) = ui_state.game.as_mut() else {
        ui_state.message = t.text("message-game-not-started");
        return;
    };
    let faction_id = game.player_faction_id.clone();
    let technology_name = game
        .technology_catalog
        .spec(&technology_id)
        .map(|spec| spec.name.clone())
        .unwrap_or_else(|| technology_id.clone());
    match start_research(game, &faction_id, &technology_id) {
        Ok(outcome) if outcome.resumed => {
            ui_state.message = t.text_args(
                "message-research-resumed",
                &args([("name", technology_name)]),
            );
        }
        Ok(outcome) => {
            ui_state.message = t.text_args(
                "message-research-started",
                &args([
                    ("name", technology_name),
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
    if faction_state.is_some_and(|state| state.active.as_deref() == Some(spec.id.as_str())) {
        return TechnologyNodeStatus::Active;
    }
    if faction_state.is_some_and(|state| state.funded.contains(&spec.id)) {
        return TechnologyNodeStatus::Funded;
    }
    if missing_prerequisite_names(faction_state, &game.technology_catalog, &spec.id)
        .is_ok_and(|missing| !missing.is_empty())
    {
        return TechnologyNodeStatus::Locked;
    }
    if effective_technology_cost(game, &game.player_faction_id, &spec.id)
        .is_ok_and(|cost| faction_total_gold(game, &game.player_faction_id) >= cost)
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

fn technology_depth(spec: &TechnologySpec, catalog: &TechnologyCatalog) -> usize {
    technology_depth_inner(spec, catalog, &mut BTreeSet::new())
}

fn technology_depth_inner(
    spec: &TechnologySpec,
    catalog: &TechnologyCatalog,
    visited: &mut BTreeSet<TechnologyId>,
) -> usize {
    if !visited.insert(spec.id.clone()) {
        return 0;
    }
    let depth = spec
        .prerequisites
        .iter()
        .filter_map(|id| catalog.spec(id))
        .map(|prerequisite| technology_depth_inner(prerequisite, catalog, visited) + 1)
        .max()
        .unwrap_or_default();
    visited.remove(&spec.id);
    depth
}

fn technology_icon(spec: &TechnologySpec) -> &'static str {
    match spec.icon_id.as_str() {
        "users" => egui_phosphor::regular::USERS,
        "stack" => egui_phosphor::regular::STACK,
        "map_trifold" => egui_phosphor::regular::MAP_TRIFOLD,
        "sword" => egui_phosphor::regular::SWORD,
        "flag" => egui_phosphor::regular::FLAG,
        "shield" => egui_phosphor::regular::SHIELD,
        "barn" => egui_phosphor::regular::BARN,
        "arrows_out_cardinal" => egui_phosphor::regular::ARROWS_OUT_CARDINAL,
        "fire" => egui_phosphor::regular::FIRE,
        "arrows_clockwise" => egui_phosphor::regular::ARROWS_CLOCKWISE,
        "warehouse" => egui_phosphor::regular::WAREHOUSE,
        "hammer" => egui_phosphor::regular::HAMMER,
        "medal" => egui_phosphor::regular::MEDAL,
        "crown" => egui_phosphor::regular::CROWN,
        "identification_card" => egui_phosphor::regular::IDENTIFICATION_CARD,
        "drop" => egui_phosphor::regular::DROP,
        "coins" => egui_phosphor::regular::COINS,
        "scales" => egui_phosphor::regular::SCALES,
        "waves" => egui_phosphor::regular::WAVES,
        "bridge" => egui_phosphor::regular::BRIDGE,
        "scroll" => egui_phosphor::regular::SCROLL,
        "gear" => egui_phosphor::regular::GEAR,
        "clipboard_text" => egui_phosphor::regular::CLIPBOARD_TEXT,
        "bank" => egui_phosphor::regular::BANK,
        "seal_check" => egui_phosphor::regular::SEAL_CHECK,
        _ => egui_phosphor::regular::CIRCLES_THREE_PLUS,
    }
}

fn prerequisite_names(spec: &TechnologySpec, catalog: &TechnologyCatalog) -> Vec<String> {
    spec.prerequisites
        .iter()
        .map(|id| {
            catalog
                .spec(id)
                .map(|spec| spec.name.clone())
                .unwrap_or_else(|| id.clone())
        })
        .collect()
}
