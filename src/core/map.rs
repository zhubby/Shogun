use crate::game::*;
use bevy_egui::egui;
use std::collections::{BTreeMap, BTreeSet};

use super::actions::open_city;
use super::i18n::{Translator, args};
use super::state::GameUiState;
use super::style::{draw_strategy_map_background, war_border, war_gold, war_text};
use super::{MAP_MAX_ZOOM, MAP_MIN_ZOOM};

pub(super) fn map_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState, t: &Translator) {
    if ui_state.game.is_none() {
        ui.centered_and_justified(|ui| {
            ui.label(t.text("message-game-not-started"));
        });
        return;
    }
    let desired = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());
    let response = response.on_hover_cursor(egui::CursorIcon::Grab);
    let painter = ui.painter_at(rect);
    draw_strategy_map_background(&painter, rect);

    let Some(bounds) = ui_state.game.as_ref().and_then(map_bounds) else {
        return;
    };
    let limits = map_pan_limits_for_state(ui_state, bounds, rect);
    clamp_map_pan(ui_state, limits);

    let scroll_delta = ui.input(|input| input.raw_scroll_delta.y);
    if response.hovered() && scroll_delta.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + scroll_delta * 0.0015).clamp(0.8, 1.25);
        zoom_map(
            ui_state,
            zoom_factor,
            response.hover_pos(),
            Some(rect.center()),
        );
        let limits = map_pan_limits_for_state(ui_state, bounds, rect);
        clamp_map_pan(ui_state, limits);
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        ui_state.map_pan += response.drag_delta();
        let limits = map_pan_limits_for_state(ui_state, bounds, rect);
        clamp_map_pan(ui_state, limits);
        ui.output_mut(|output| output.cursor_icon = egui::CursorIcon::Grabbing);
    }

    let Some(game) = &ui_state.game else {
        return;
    };

    if ui_state.map_boundaries_enabled
        && let Some(catalog) = &ui_state.map_boundaries
    {
        let view = MapBoundaryView::from_ui_state(ui_state);
        let (cells, province_segments) = ui_state
            .map_boundary_view_cache
            .boundaries_for_year(catalog, game.year);
        draw_map_boundaries(&painter, game, cells, province_segments, bounds, rect, view);
    }

    for road in &game.roads {
        let Some(from) = game.cities.get(&road.from) else {
            continue;
        };
        let Some(to) = game.cities.get(&road.to) else {
            continue;
        };
        let a = map_to_screen(from.position, bounds, rect, ui_state);
        let b = map_to_screen(to.position, bounds, rect, ui_state);
        let selected = ui_state
            .selected_city_id
            .as_deref()
            .is_some_and(|city_id| road.from.as_str() == city_id || road.to.as_str() == city_id);
        draw_map_road(&painter, game, road, a, b, selected, t);
    }

    draw_expedition_movements(&painter, game, bounds, rect, ui_state, t);

    for city in game.cities.values() {
        let pos = map_to_screen(city.position, bounds, rect, ui_state);
        let faction = &game.factions[&city.faction_id];
        let color = faction_color(faction);
        let selected = ui_state.selected_city_id.as_deref() == Some(city.id.as_str());
        let player_owned = city.faction_id == game.player_faction_id;
        draw_city_marker(
            &painter,
            pos,
            city,
            CityMarkerStyle {
                color,
                selected,
                player_owned,
                map_zoom: ui_state.map_zoom,
            },
            t,
        );
    }

    draw_siege_overlays(&painter, game, bounds, rect, ui_state, t);

    let picked_city = response
        .interact_pointer_pos()
        .and_then(|pointer_pos| city_at_position(game, bounds, rect, pointer_pos, ui_state));

    if response.clicked()
        && let Some(city_id) = picked_city.clone()
    {
        ui_state.selected_city_id = Some(city_id);
    }
    if response.secondary_clicked()
        && let Some(city_id) = picked_city.clone()
    {
        ui_state.selected_city_id = Some(city_id);
    }

    let context_city_id = picked_city
        .clone()
        .or_else(|| ui_state.selected_city_id.clone());
    response.context_menu(|ui| {
        if let Some(city_id) = context_city_id.clone()
            && ui.button(t.text("open-command-tent")).clicked()
        {
            open_city(ui_state, city_id);
            ui.close();
        }
    });
}

fn draw_expedition_movements(
    painter: &egui::Painter,
    game: &GameState,
    bounds: MapBounds,
    rect: egui::Rect,
    ui_state: &GameUiState,
    t: &Translator,
) {
    for (index, movement) in active_expedition_movements(game).enumerate() {
        let Some(source) = game.cities.get(&movement.source_city_id) else {
            continue;
        };
        let Some(target) = game.cities.get(&movement.target_city_id) else {
            continue;
        };
        let Some(progress) = expedition_movement_line_progress(movement, game.turn) else {
            continue;
        };
        let Some(faction) = game.factions.get(&movement.issuer_faction_id) else {
            continue;
        };

        let start = map_to_screen(source.position, bounds, rect, ui_state);
        let end = map_to_screen(target.position, bounds, rect, ui_state);
        draw_expedition_marker(
            painter,
            start,
            end,
            progress,
            index,
            faction_color(faction),
            compact_troops(movement.troops.total(), t),
        );
    }
}

fn draw_expedition_marker(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    progress: f32,
    index: usize,
    faction_color: egui::Color32,
    troops: String,
) {
    let delta = end - start;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }

    let direction = delta / length;
    let normal = egui::vec2(-direction.y, direction.x);
    let lane_offset = match index % 3 {
        0 => 0.0,
        1 => 13.0,
        _ => -13.0,
    };
    let marker_pos = start + delta * progress + normal * lane_offset;
    let marker_bounds = egui::Rect::from_center_size(marker_pos, egui::vec2(72.0, 66.0));
    if !marker_bounds.intersects(painter.clip_rect()) {
        return;
    }

    let shadow = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128);
    let ring = egui::Color32::from_rgba_unmultiplied(242, 208, 142, 230);
    let label_fill = egui::Color32::from_rgba_unmultiplied(18, 15, 11, 236);
    let fill = faction_color.gamma_multiply(0.78);
    let icon_pos = marker_pos - normal * 0.5;

    painter.circle_filled(marker_pos + egui::vec2(1.8, 2.2), 16.5, shadow);
    painter.circle_filled(
        marker_pos,
        16.0,
        egui::Color32::from_rgba_unmultiplied(20, 17, 13, 238),
    );
    painter.circle_filled(marker_pos, 12.2, fill);
    painter.circle_stroke(marker_pos, 16.0, egui::Stroke::new(1.7, ring));

    painter.text(
        icon_pos,
        egui::Align2::CENTER_CENTER,
        egui_phosphor::regular::FLAG_BANNER,
        egui::FontId::proportional(18.5),
        egui::Color32::WHITE,
    );

    let label_center = marker_pos - normal * 25.0;
    let label_width = (troops.chars().count() as f32 * 8.0 + 20.0).clamp(44.0, 76.0);
    let label_rect = egui::Rect::from_center_size(label_center, egui::vec2(label_width, 21.0));
    painter.rect(
        label_rect.translate(egui::vec2(1.4, 1.8)),
        4.0,
        shadow,
        egui::Stroke::NONE,
        egui::StrokeKind::Outside,
    );
    painter.rect(
        label_rect,
        4.0,
        label_fill,
        egui::Stroke::new(1.0, ring.gamma_multiply(0.9)),
        egui::StrokeKind::Outside,
    );
    painter.text(
        label_rect.center(),
        egui::Align2::CENTER_CENTER,
        troops,
        egui::FontId::proportional(12.5),
        war_text(),
    );
}

fn draw_siege_overlays(
    painter: &egui::Painter,
    game: &GameState,
    bounds: MapBounds,
    rect: egui::Rect,
    ui_state: &GameUiState,
    t: &Translator,
) {
    for siege in siege_summaries(game) {
        let Some(city) = game.cities.get(&siege.target_city_id) else {
            continue;
        };
        let pos = map_to_screen(city.position, bounds, rect, ui_state);
        draw_siege_badge(painter, pos, city, &siege, ui_state.map_zoom, t);
    }
}

fn draw_siege_badge(
    painter: &egui::Painter,
    pos: egui::Pos2,
    city: &City,
    siege: &SiegeSummary,
    map_zoom: f32,
    t: &Translator,
) {
    let scale = map_zoom.sqrt().clamp(0.85, 1.35);
    let marker_scale = scale * city_marker_rank_scale(city);
    let center = pos + egui::vec2(0.0, -5.0 * scale);
    let ring_radius = siege_ring_radius(marker_scale);

    let warning = egui::Color32::from_rgb(219, 87, 58);
    let badge_text = t.text_args(
        "map-siege-badge",
        &args([
            ("count", siege.attacker_count().to_string()),
            ("troops", compact_troops(siege.total_troops, t)),
        ]),
    );
    let badge_width = (badge_text.chars().count() as f32 * 8.5 + 22.0).clamp(64.0, 124.0) * scale;
    let badge_height = 23.0 * scale;
    let badge_rect = egui::Rect::from_center_size(
        center + egui::vec2(0.0, -(ring_radius + badge_height * 0.5)),
        egui::vec2(badge_width, badge_height),
    );
    let ring_bounds = egui::Rect::from_center_size(
        center,
        egui::vec2(
            (ring_radius + 4.0 * scale) * 2.0,
            (ring_radius + 4.0 * scale) * 2.0,
        ),
    );
    if !ring_bounds
        .union(badge_rect)
        .intersects(painter.clip_rect())
    {
        return;
    }

    painter.circle_stroke(
        center,
        ring_radius,
        egui::Stroke::new(
            4.5 * scale,
            egui::Color32::from_rgba_unmultiplied(74, 9, 6, 132),
        ),
    );
    painter.circle_stroke(center, ring_radius, egui::Stroke::new(2.0 * scale, warning));

    painter.rect(
        badge_rect.translate(egui::vec2(1.6 * scale, 2.0 * scale)),
        4.0 * scale,
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 128),
        egui::Stroke::NONE,
        egui::StrokeKind::Outside,
    );
    painter.rect(
        badge_rect,
        4.0 * scale,
        egui::Color32::from_rgba_unmultiplied(54, 18, 13, 238),
        egui::Stroke::new(1.0 * scale, warning),
        egui::StrokeKind::Outside,
    );
    painter.text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        badge_text,
        egui::FontId::proportional(12.0 * scale),
        egui::Color32::from_rgb(255, 223, 183),
    );
}

fn city_marker_icon_outer_radius(marker_scale: f32) -> f32 {
    24.0 * marker_scale
}

fn siege_ring_radius(marker_scale: f32) -> f32 {
    city_marker_icon_outer_radius(marker_scale) + 2.5 * marker_scale
}

fn active_expedition_movements(game: &GameState) -> impl Iterator<Item = &ArmyMovement> {
    game.army_movements
        .iter()
        .filter(|movement| movement.kind == ArmyMovementKind::Expedition)
        .filter(|movement| movement.siege_started_turn.is_none())
}

fn besieging_expedition_movements(game: &GameState) -> impl Iterator<Item = &ArmyMovement> {
    game.army_movements
        .iter()
        .filter(|movement| movement.kind == ArmyMovementKind::Expedition)
        .filter(|movement| movement.siege_started_turn.is_some())
}

fn siege_summaries(game: &GameState) -> Vec<SiegeSummary> {
    let mut summaries: BTreeMap<&str, SiegeSummary> = BTreeMap::new();
    for movement in besieging_expedition_movements(game) {
        let summary = summaries
            .entry(&movement.target_city_id)
            .or_insert_with(|| SiegeSummary {
                target_city_id: movement.target_city_id.clone(),
                attacker_faction_ids: BTreeSet::new(),
                total_troops: 0,
            });
        summary
            .attacker_faction_ids
            .insert(movement.issuer_faction_id.clone());
        summary.total_troops = summary.total_troops.saturating_add(movement.troops.total());
    }
    summaries.into_values().collect()
}

fn expedition_movement_line_progress(movement: &ArmyMovement, current_turn: u32) -> Option<f32> {
    if movement.kind != ArmyMovementKind::Expedition || movement.siege_started_turn.is_some() {
        return None;
    }

    let total = movement
        .arrival_turn
        .saturating_sub(movement.departure_turn);
    if total == 0 {
        return Some(0.5);
    }

    let elapsed = current_turn
        .saturating_sub(movement.departure_turn)
        .min(total);
    let waypoint = elapsed.max(1);
    Some((waypoint as f32 / (total + 1) as f32).clamp(0.08, 0.92))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SiegeSummary {
    target_city_id: CityId,
    attacker_faction_ids: BTreeSet<FactionId>,
    total_troops: u32,
}

impl SiegeSummary {
    fn attacker_count(&self) -> usize {
        self.attacker_faction_ids.len()
    }
}

fn draw_map_road(
    painter: &egui::Painter,
    game: &GameState,
    road: &Road,
    a: egui::Pos2,
    b: egui::Pos2,
    selected: bool,
    t: &Translator,
) {
    let bounds = egui::Rect::from_two_pos(a, b).expand(if selected { 10.0 } else { 7.0 });
    if !bounds.intersects(painter.clip_rect()) {
        return;
    }

    let shadow_width = if selected { 10.0 } else { 7.0 };
    let road_width = if selected { 4.2 } else { 3.0 };
    let road_color = if selected {
        egui::Color32::from_rgba_unmultiplied(216, 177, 95, 225)
    } else {
        egui::Color32::from_rgba_unmultiplied(166, 135, 83, 170)
    };
    painter.line_segment(
        [a, b],
        egui::Stroke::new(
            shadow_width,
            egui::Color32::from_rgba_unmultiplied(10, 12, 10, 108),
        ),
    );
    painter.line_segment([a, b], egui::Stroke::new(road_width, road_color));

    if selected {
        draw_road_distance_label(painter, game, road, a, b, t);
    }
}

fn draw_road_distance_label(
    painter: &egui::Painter,
    game: &GameState,
    road: &Road,
    a: egui::Pos2,
    b: egui::Pos2,
    t: &Translator,
) {
    let Some(distance_li) = game.road_distance_li(&road.from, &road.to) else {
        return;
    };
    let Some(travel_months) = game.travel_months_between(&road.from, &road.to) else {
        return;
    };
    let text = t.text_args(
        "map-road-distance",
        &args([
            ("distance", distance_li.to_string()),
            ("months", travel_months.to_string()),
        ]),
    );
    let width = (text.chars().count() as f32 * 8.5 + 18.0).max(74.0);
    let label_pos = a + (b - a) * 0.5;
    let rect =
        egui::Rect::from_center_size(label_pos + egui::vec2(0.0, -12.0), egui::vec2(width, 22.0));
    painter.rect(
        rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(18, 16, 12, 224),
        egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(216, 177, 95, 190),
        ),
        egui::StrokeKind::Outside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(12.0),
        war_gold(),
    );
}

pub(super) fn draw_map_boundaries(
    painter: &egui::Painter,
    game: &GameState,
    cells: &[TerritoryCell],
    province_segments: &[(MapPosition, MapPosition)],
    bounds: MapBounds,
    rect: egui::Rect,
    view: MapBoundaryView,
) {
    if cells.is_empty() {
        return;
    }

    for cell in cells {
        let fill = territory_cell_fill_color_for_selection(cell, game, view.selected_city_id());
        let points = territory_cell_screen_points(cell, bounds, rect, view.transform);
        paint_boundary_polygon(painter, points, fill, egui::Stroke::NONE);
    }

    for cell in cells {
        let selected = selected_city_in_cell_id(cell, game, view.selected_city_id());
        let points = territory_cell_screen_points(cell, bounds, rect, view.transform);
        let (stroke, dash, gap) = if selected {
            (
                egui::Stroke::new(
                    2.0,
                    egui::Color32::from_rgba_unmultiplied(174, 221, 210, 230),
                ),
                8.0,
                4.0,
            )
        } else {
            (
                egui::Stroke::new(
                    0.9,
                    egui::Color32::from_rgba_unmultiplied(112, 136, 124, 98),
                ),
                6.0,
                8.0,
            )
        };
        draw_dashed_closed_polyline(painter, &points, stroke, dash, gap);
    }

    for &(start, end) in province_segments {
        let start = map_to_screen_with_transform(start, bounds, rect, view.transform);
        let end = map_to_screen_with_transform(end, bounds, rect, view.transform);
        draw_dashed_segment(
            painter,
            start,
            end,
            egui::Stroke::new(3.5, egui::Color32::from_rgba_unmultiplied(6, 14, 15, 150)),
            15.0,
            7.0,
        );
        draw_dashed_segment(
            painter,
            start,
            end,
            egui::Stroke::new(
                1.8,
                egui::Color32::from_rgba_unmultiplied(116, 171, 170, 190),
            ),
            15.0,
            7.0,
        );
    }
}

pub(super) fn territory_cell_screen_points(
    cell: &TerritoryCell,
    bounds: MapBounds,
    rect: egui::Rect,
    transform: MapTransform,
) -> Vec<egui::Pos2> {
    cell.points
        .iter()
        .map(|point| map_to_screen_with_transform(*point, bounds, rect, transform))
        .collect()
}

pub(super) fn paint_boundary_polygon(
    painter: &egui::Painter,
    points: Vec<egui::Pos2>,
    fill: egui::Color32,
    stroke: egui::Stroke,
) {
    if points.len() < 3 {
        return;
    }
    if points_screen_bounds(&points).is_some_and(|bounds| !bounds.intersects(painter.clip_rect())) {
        return;
    }
    painter.add(egui::Shape::Path(egui::epaint::PathShape {
        points,
        closed: true,
        fill,
        stroke: stroke.into(),
    }));
}

pub(super) fn draw_dashed_closed_polyline(
    painter: &egui::Painter,
    points: &[egui::Pos2],
    stroke: egui::Stroke,
    dash: f32,
    gap: f32,
) {
    if points.len() < 2 {
        return;
    }

    let cycle = dash + gap;
    if cycle <= f32::EPSILON {
        return;
    }
    if points_screen_bounds(points).is_some_and(|bounds| !bounds.intersects(painter.clip_rect())) {
        return;
    }

    for index in 0..points.len() {
        draw_dashed_segment(
            painter,
            points[index],
            points[(index + 1) % points.len()],
            stroke,
            dash,
            gap,
        );
    }
}

pub(super) fn draw_dashed_segment(
    painter: &egui::Painter,
    start: egui::Pos2,
    end: egui::Pos2,
    stroke: egui::Stroke,
    dash: f32,
    gap: f32,
) {
    let cycle = dash + gap;
    if cycle <= f32::EPSILON {
        return;
    }

    let delta = end - start;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }
    let bounds = egui::Rect::from_two_pos(start, end).expand(stroke.width);
    if !bounds.intersects(painter.clip_rect()) {
        return;
    }

    let direction = delta / length;
    let mut offset = 0.0;
    while offset < length {
        let dash_end = (offset + dash).min(length);
        painter.line_segment(
            [start + direction * offset, start + direction * dash_end],
            stroke,
        );
        offset += cycle;
    }
}

pub(super) fn points_screen_bounds(points: &[egui::Pos2]) -> Option<egui::Rect> {
    let first = points.first().copied()?;
    let mut bounds = egui::Rect::from_min_max(first, first);
    for point in &points[1..] {
        bounds.min.x = bounds.min.x.min(point.x);
        bounds.min.y = bounds.min.y.min(point.y);
        bounds.max.x = bounds.max.x.max(point.x);
        bounds.max.y = bounds.max.y.max(point.y);
    }
    Some(bounds)
}

#[derive(Default)]
pub(super) struct MapBoundaryViewCache {
    year: Option<i32>,
    cells: Vec<TerritoryCell>,
    province_segments: Vec<(MapPosition, MapPosition)>,
}

impl MapBoundaryViewCache {
    pub(super) fn boundaries_for_year(
        &mut self,
        catalog: &MapBoundaryCatalog,
        year: i32,
    ) -> (&[TerritoryCell], &[(MapPosition, MapPosition)]) {
        if self.year != Some(year) {
            self.cells = catalog.territory_cells_for_year(year);
            self.province_segments = province_border_segments(&self.cells);
            self.year = Some(year);
        }

        (&self.cells, &self.province_segments)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TerritoryEdgeKey {
    start_x: i64,
    start_y: i64,
    end_x: i64,
    end_y: i64,
}

pub(super) struct TerritoryEdge {
    start: MapPosition,
    end: MapPosition,
    parent_id: Option<String>,
}

pub(super) fn province_border_segments(cells: &[TerritoryCell]) -> Vec<(MapPosition, MapPosition)> {
    let mut edges: BTreeMap<TerritoryEdgeKey, Vec<TerritoryEdge>> = BTreeMap::new();
    for cell in cells {
        for index in 0..cell.points.len() {
            let start = cell.points[index];
            let end = cell.points[(index + 1) % cell.points.len()];
            edges
                .entry(territory_edge_key(start, end))
                .or_default()
                .push(TerritoryEdge {
                    start,
                    end,
                    parent_id: cell.parent_id.clone(),
                });
        }
    }

    edges
        .into_values()
        .filter_map(|edge_group| {
            let first = edge_group.first()?;
            let crosses_parent = edge_group
                .iter()
                .any(|edge| edge.parent_id != first.parent_id);
            (edge_group.len() == 1 || crosses_parent).then_some((first.start, first.end))
        })
        .collect()
}

pub(super) fn territory_edge_key(start: MapPosition, end: MapPosition) -> TerritoryEdgeKey {
    let start = quantized_map_position(start);
    let end = quantized_map_position(end);
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    TerritoryEdgeKey {
        start_x: start.0,
        start_y: start.1,
        end_x: end.0,
        end_y: end.1,
    }
}

pub(super) fn quantized_map_position(position: MapPosition) -> (i64, i64) {
    (
        (position.x * 1_000.0).round() as i64,
        (position.y * 1_000.0).round() as i64,
    )
}

pub(super) fn territory_cell_fill_color_for_selection(
    cell: &TerritoryCell,
    game: &GameState,
    selected_city_id: Option<&str>,
) -> egui::Color32 {
    let selected = selected_city_in_cell_id(cell, game, selected_city_id);
    let alpha = if selected { 44 } else { 18 };

    dominant_cell_faction(cell, game)
        .map(faction_color)
        .map(|color| egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha))
        .unwrap_or_else(|| egui::Color32::from_rgba_unmultiplied(92, 96, 67, alpha))
}

pub(super) fn dominant_cell_faction<'a>(
    cell: &TerritoryCell,
    game: &'a GameState,
) -> Option<&'a Faction> {
    let mut counts: BTreeMap<&str, (usize, u32)> = BTreeMap::new();
    for city in game
        .cities
        .values()
        .filter(|city| city_matches_cell(cell, city))
    {
        let entry = counts.entry(city.faction_id.as_str()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 = entry.1.saturating_add(city.troops.total());
    }
    let faction_id = counts
        .iter()
        .max_by_key(|(_, (city_count, troops))| (*city_count, *troops))
        .map(|(faction_id, _)| *faction_id)?;
    game.factions.get(faction_id)
}

pub(super) fn selected_city_in_cell_id(
    cell: &TerritoryCell,
    game: &GameState,
    selected_city_id: Option<&str>,
) -> bool {
    let Some(selected_city_id) = selected_city_id else {
        return false;
    };
    game.cities
        .get(selected_city_id)
        .is_some_and(|city| city_matches_cell(cell, city))
}

pub(super) fn city_matches_cell(cell: &TerritoryCell, city: &City) -> bool {
    if !cell.city_ids.is_empty() {
        return cell.city_ids.iter().any(|city_id| city_id == &city.id);
    }

    let Some(profile) = &city.profile else {
        return false;
    };
    profile.commandery == cell.name
}

pub(super) struct CityMarkerStyle {
    color: egui::Color32,
    selected: bool,
    player_owned: bool,
    map_zoom: f32,
}

pub(super) fn draw_city_marker(
    painter: &egui::Painter,
    pos: egui::Pos2,
    city: &City,
    style: CityMarkerStyle,
    t: &Translator,
) {
    let CityMarkerStyle {
        color,
        selected,
        player_owned,
        map_zoom,
    } = style;
    let scale = map_zoom.sqrt().clamp(0.85, 1.35);
    let marker_scale = scale * city_marker_rank_scale(city);
    let marker_center = pos + egui::vec2(0.0, -5.0 * scale);
    let faction_fill = draw_city_marker_icon(
        painter,
        marker_center,
        marker_scale,
        scale,
        color,
        selected,
        player_owned,
    );

    let base = egui::Rect::from_center_size(
        pos + egui::vec2(0.0, 25.0 * scale),
        egui::vec2(48.0 * scale, 18.0 * scale),
    );
    painter.rect(
        base,
        4.0,
        egui::Color32::from_rgba_unmultiplied(20, 18, 14, 222),
        egui::Stroke::new(1.0, faction_fill),
        egui::StrokeKind::Outside,
    );
    painter.text(
        base.center(),
        egui::Align2::CENTER_CENTER,
        compact_troops(city.troops.total(), t),
        egui::FontId::proportional(12.0 * scale),
        war_text(),
    );

    let label_center = pos + egui::vec2(0.0, 54.0 * scale);
    let label_width = (city.name.chars().count() as f32 * 17.0 + 28.0).max(68.0);
    let label_rect =
        egui::Rect::from_center_size(label_center, egui::vec2(label_width, 25.0 * scale));
    painter.rect(
        label_rect,
        4.0,
        egui::Color32::from_rgba_unmultiplied(17, 16, 13, if selected { 238 } else { 204 }),
        egui::Stroke::new(
            if selected { 1.5 } else { 1.0 },
            if selected { war_gold() } else { war_border() },
        ),
        egui::StrokeKind::Outside,
    );
    painter.text(
        label_center,
        egui::Align2::CENTER_CENTER,
        &city.name,
        egui::FontId::proportional(15.0 * scale),
        if selected { war_gold() } else { war_text() },
    );
}

pub(super) fn draw_city_marker_icon(
    painter: &egui::Painter,
    marker_center: egui::Pos2,
    marker_scale: f32,
    shadow_scale: f32,
    color: egui::Color32,
    selected: bool,
    player_owned: bool,
) -> egui::Color32 {
    let radius = 20.0 * marker_scale;
    let faction_fill = if player_owned {
        color
    } else {
        color.gamma_multiply(0.74)
    };
    let shadow = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120);
    let ring = if selected { war_gold() } else { faction_fill };

    painter.circle_filled(
        marker_center + egui::vec2(2.5 * shadow_scale, 3.5 * shadow_scale),
        radius + 5.0 * marker_scale,
        shadow,
    );
    painter.circle_filled(
        marker_center,
        city_marker_icon_outer_radius(marker_scale),
        egui::Color32::from_rgba_unmultiplied(20, 17, 13, 238),
    );
    painter.circle_stroke(
        marker_center,
        radius + 3.0 * marker_scale,
        egui::Stroke::new(if selected { 3.0 } else { 1.8 } * marker_scale, ring),
    );
    painter.circle_filled(marker_center, radius, faction_fill.gamma_multiply(0.62));
    painter.circle_filled(
        marker_center,
        radius - 5.0 * marker_scale,
        egui::Color32::from_rgba_unmultiplied(36, 31, 23, 226),
    );

    let wall_fill = egui::Color32::from_rgb(106, 88, 58);
    let wall_light = egui::Color32::from_rgb(171, 139, 83);
    let wall_dark = egui::Color32::from_rgb(48, 38, 25);
    let wall = egui::Rect::from_center_size(
        marker_center + egui::vec2(0.0, 4.0 * marker_scale),
        egui::vec2(32.0 * marker_scale, 16.0 * marker_scale),
    );
    painter.rect(
        wall.translate(egui::vec2(1.8 * shadow_scale, 2.0 * shadow_scale)),
        2.0 * marker_scale,
        shadow,
        egui::Stroke::NONE,
        egui::StrokeKind::Outside,
    );
    painter.rect(
        wall,
        2.0 * marker_scale,
        wall_fill,
        egui::Stroke::new(1.2 * marker_scale, wall_dark),
        egui::StrokeKind::Outside,
    );
    painter.line_segment(
        [
            egui::pos2(wall.left(), wall.top() + 5.0 * marker_scale),
            egui::pos2(wall.right(), wall.top() + 5.0 * marker_scale),
        ],
        egui::Stroke::new(1.0 * marker_scale, wall_light),
    );

    let merlon_width = 5.5 * marker_scale;
    let merlon_gap = 2.0 * marker_scale;
    let merlon_height = 5.8 * marker_scale;
    let total_merlon_width = 4.0 * merlon_width + 3.0 * merlon_gap;
    let mut merlon_left = marker_center.x - total_merlon_width * 0.5;
    for _ in 0..4 {
        let merlon = egui::Rect::from_min_size(
            egui::pos2(merlon_left, wall.top() - merlon_height + 1.0 * marker_scale),
            egui::vec2(merlon_width, merlon_height),
        );
        painter.rect(
            merlon,
            1.0 * marker_scale,
            wall_light,
            egui::Stroke::new(0.8 * marker_scale, wall_dark),
            egui::StrokeKind::Outside,
        );
        merlon_left += merlon_width + merlon_gap;
    }

    let gate = egui::Rect::from_center_size(
        egui::pos2(marker_center.x, wall.bottom() - 4.4 * marker_scale),
        egui::vec2(8.5 * marker_scale, 10.2 * marker_scale),
    );
    painter.rect(
        gate,
        2.0 * marker_scale,
        egui::Color32::from_rgb(42, 31, 21),
        egui::Stroke::new(0.9 * marker_scale, egui::Color32::from_rgb(133, 105, 62)),
        egui::StrokeKind::Outside,
    );

    let crest = egui::Rect::from_center_size(
        marker_center + egui::vec2(0.0, -10.5 * marker_scale),
        egui::vec2(14.0 * marker_scale, 4.0 * marker_scale),
    );
    painter.rect(
        crest,
        1.0 * marker_scale,
        faction_fill,
        egui::Stroke::new(0.8 * marker_scale, wall_dark),
        egui::StrokeKind::Outside,
    );

    faction_fill
}

pub(super) fn city_marker_rank_scale(city: &City) -> f32 {
    match city.profile.as_ref().map(|profile| &profile.scale) {
        Some(CityScale::ImperialCapital) => 1.18,
        Some(CityScale::RegionalCapital) => 1.1,
        Some(CityScale::Commandery) => 1.02,
        Some(CityScale::County) => 0.94,
        None => (0.94 + f32::from(city.level) * 0.02).clamp(0.96, 1.14),
    }
}

pub(super) fn compact_troops(troops: u32, t: &Translator) -> String {
    if troops >= 10_000 {
        t.text_args(
            "map-troops-ten-thousand",
            &args([("value", format!("{:.1}", troops as f32 / 10_000.0))]),
        )
    } else {
        troops.to_string()
    }
}

#[derive(Clone, Copy)]
pub(super) struct MapBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

#[derive(Clone, Copy)]
pub(super) struct MapTransform {
    zoom: f32,
    pan: egui::Vec2,
}

impl MapTransform {
    fn from_ui_state(ui_state: &GameUiState) -> Self {
        Self {
            zoom: ui_state.map_zoom,
            pan: ui_state.map_pan,
        }
    }
}

#[derive(Clone)]
pub(super) struct MapBoundaryView {
    transform: MapTransform,
    selected_city_id: Option<CityId>,
}

impl MapBoundaryView {
    fn from_ui_state(ui_state: &GameUiState) -> Self {
        Self {
            transform: MapTransform::from_ui_state(ui_state),
            selected_city_id: ui_state.selected_city_id.clone(),
        }
    }

    fn selected_city_id(&self) -> Option<&str> {
        self.selected_city_id.as_deref()
    }
}

pub(super) fn map_bounds(game: &GameState) -> Option<MapBounds> {
    let mut cities = game.cities.values();
    let first = cities.next()?;
    let mut bounds = MapBounds {
        min_x: first.position.x,
        max_x: first.position.x,
        min_y: first.position.y,
        max_y: first.position.y,
    };
    for city in cities {
        bounds.min_x = bounds.min_x.min(city.position.x);
        bounds.max_x = bounds.max_x.max(city.position.x);
        bounds.min_y = bounds.min_y.min(city.position.y);
        bounds.max_y = bounds.max_y.max(city.position.y);
    }
    Some(bounds)
}

pub(super) fn map_to_screen(
    position: MapPosition,
    bounds: MapBounds,
    rect: egui::Rect,
    ui_state: &GameUiState,
) -> egui::Pos2 {
    map_to_screen_with_transform(
        position,
        bounds,
        rect,
        MapTransform::from_ui_state(ui_state),
    )
}

pub(super) fn map_to_screen_with_transform(
    position: MapPosition,
    bounds: MapBounds,
    rect: egui::Rect,
    transform: MapTransform,
) -> egui::Pos2 {
    let padding = map_padding(rect);
    let width = (bounds.max_x - bounds.min_x).max(1.0);
    let height = (bounds.max_y - bounds.min_y).max(1.0);
    let x = (position.x - bounds.min_x) / width;
    let y = (position.y - bounds.min_y) / height;
    let base = egui::pos2(
        rect.left() + padding + x * (rect.width() - padding * 2.0).max(1.0),
        rect.bottom() - padding - y * (rect.height() - padding * 2.0).max(1.0),
    );
    rect.center() + (base - rect.center()) * transform.zoom + transform.pan
}

pub(super) fn map_content_screen_bounds(
    game: &GameState,
    catalog: Option<&MapBoundaryCatalog>,
    bounds: MapBounds,
    rect: egui::Rect,
    zoom: f32,
) -> egui::Rect {
    let transform = MapTransform {
        zoom,
        pan: egui::Vec2::ZERO,
    };
    let mut screen_bounds = None;

    if let Some(catalog) = catalog {
        for boundary in catalog.boundaries_for_year(game.year) {
            for point in &boundary.points {
                extend_screen_bounds(
                    &mut screen_bounds,
                    map_to_screen_with_transform(*point, bounds, rect, transform),
                );
            }
        }
    }

    for city in game.cities.values() {
        let position = map_to_screen_with_transform(city.position, bounds, rect, transform);
        extend_screen_bounds_rect(
            &mut screen_bounds,
            city_marker_screen_bounds(position, city, zoom),
        );
    }

    screen_bounds.unwrap_or(rect)
}

pub(super) fn extend_screen_bounds(bounds: &mut Option<egui::Rect>, point: egui::Pos2) {
    if let Some(bounds) = bounds {
        bounds.min.x = bounds.min.x.min(point.x);
        bounds.min.y = bounds.min.y.min(point.y);
        bounds.max.x = bounds.max.x.max(point.x);
        bounds.max.y = bounds.max.y.max(point.y);
    } else {
        *bounds = Some(egui::Rect::from_min_max(point, point));
    }
}

pub(super) fn extend_screen_bounds_rect(bounds: &mut Option<egui::Rect>, rect: egui::Rect) {
    extend_screen_bounds(bounds, rect.min);
    extend_screen_bounds(bounds, rect.max);
}

pub(super) fn city_marker_screen_bounds(
    position: egui::Pos2,
    city: &City,
    zoom: f32,
) -> egui::Rect {
    let scale = zoom.sqrt().clamp(0.85, 1.35);
    let marker_scale = scale * city_marker_rank_scale(city);
    let radius = 25.0 * marker_scale;
    let label_half_width = (city.name.chars().count() as f32 * 8.5 + 14.0).max(34.0);
    let half_width = radius.max(label_half_width);
    egui::Rect::from_min_max(
        position + egui::vec2(-half_width, -32.0 * marker_scale),
        position + egui::vec2(half_width, 67.0 * scale),
    )
}

pub(super) fn city_at_position(
    game: &GameState,
    bounds: MapBounds,
    rect: egui::Rect,
    pointer_pos: egui::Pos2,
    ui_state: &GameUiState,
) -> Option<CityId> {
    game.cities
        .values()
        .filter_map(|city| {
            let pos = map_to_screen(city.position, bounds, rect, ui_state);
            let distance = pos.distance(pointer_pos);
            (distance <= city_pick_radius(ui_state)).then_some((distance, city.id.clone()))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, city_id)| city_id)
}

pub(super) fn city_pick_radius(ui_state: &GameUiState) -> f32 {
    (24.0 * ui_state.map_zoom.sqrt()).clamp(22.0, 38.0)
}

pub(super) fn zoom_map(
    ui_state: &mut GameUiState,
    factor: f32,
    anchor: Option<egui::Pos2>,
    center: Option<egui::Pos2>,
) {
    let old_zoom = ui_state.map_zoom;
    let new_zoom = (old_zoom * factor).clamp(MAP_MIN_ZOOM, MAP_MAX_ZOOM);
    if (new_zoom - old_zoom).abs() <= f32::EPSILON {
        return;
    }

    ui_state.map_zoom = new_zoom;
    if let (Some(anchor), Some(center)) = (anchor, center) {
        let content_from_center = (anchor - center - ui_state.map_pan) / old_zoom;
        ui_state.map_pan = anchor - center - content_from_center * new_zoom;
    } else if new_zoom <= 1.0 {
        ui_state.map_pan = egui::Vec2::ZERO;
    }
}

pub(super) fn clamp_map_pan(ui_state: &mut GameUiState, limits: MapPanLimits) {
    ui_state.map_pan.x = ui_state.map_pan.x.clamp(limits.min.x, limits.max.x);
    ui_state.map_pan.y = ui_state.map_pan.y.clamp(limits.min.y, limits.max.y);
}

pub(super) fn map_pan_limits_for_state(
    ui_state: &GameUiState,
    bounds: MapBounds,
    rect: egui::Rect,
) -> MapPanLimits {
    let catalog = ui_state
        .map_boundaries_enabled
        .then_some(ui_state.map_boundaries.as_ref())
        .flatten();
    let content_bounds = ui_state.game.as_ref().map_or(rect, |game| {
        map_content_screen_bounds(game, catalog, bounds, rect, ui_state.map_zoom)
    });
    map_pan_limits(content_bounds, rect)
}

#[derive(Clone, Copy)]
pub(super) struct MapPanLimits {
    min: egui::Vec2,
    max: egui::Vec2,
}

pub(super) fn map_pan_limits(content: egui::Rect, viewport: egui::Rect) -> MapPanLimits {
    let (min_x, max_x) =
        pan_axis_limits(content.min.x, content.max.x, viewport.min.x, viewport.max.x);
    let (min_y, max_y) =
        pan_axis_limits(content.min.y, content.max.y, viewport.min.y, viewport.max.y);
    MapPanLimits {
        min: egui::vec2(min_x, min_y),
        max: egui::vec2(max_x, max_y),
    }
}

pub(super) fn pan_axis_limits(
    content_min: f32,
    content_max: f32,
    viewport_min: f32,
    viewport_max: f32,
) -> (f32, f32) {
    let align_start = viewport_min - content_min;
    let align_end = viewport_max - content_max;
    if content_max - content_min > viewport_max - viewport_min {
        (align_end, align_start)
    } else {
        (align_start, align_end)
    }
}

pub(super) fn map_padding(rect: egui::Rect) -> f32 {
    (rect.width().min(rect.height()) * 0.09).clamp(72.0, 118.0)
}

pub(super) fn reset_map_view(ui_state: &mut GameUiState) {
    ui_state.map_zoom = 1.0;
    ui_state.map_pan = egui::Vec2::ZERO;
}

pub(super) fn faction_color(faction: &Faction) -> egui::Color32 {
    egui::Color32::from_rgb(
        (faction.color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (faction.color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (faction.color[2].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_rect() -> egui::Rect {
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 820.0))
    }

    #[test]
    fn pan_limits_allow_dragging_at_default_zoom() {
        let rect = map_rect();
        let padding = map_padding(rect);
        let content = egui::Rect::from_min_max(
            egui::pos2(rect.left() + padding, rect.top() + padding),
            egui::pos2(rect.right() - padding, rect.bottom() - padding),
        );
        let limits = map_pan_limits(content, rect);

        assert!(limits.min.x < 0.0);
        assert!(limits.max.x > 0.0);
        assert!(limits.min.y < 0.0);
        assert!(limits.max.y > 0.0);
    }

    #[test]
    fn pan_limits_reach_edges_of_wide_content() {
        let rect = map_rect();
        let content = egui::Rect::from_min_max(
            egui::pos2(rect.left() - 420.0, rect.top() - 160.0),
            egui::pos2(rect.right() + 310.0, rect.bottom() + 280.0),
        );
        let limits = map_pan_limits(content, rect);

        assert_eq!(limits.min.x, -310.0);
        assert_eq!(limits.max.x, 420.0);
        assert_eq!(limits.min.y, -280.0);
        assert_eq!(limits.max.y, 160.0);
    }

    #[test]
    fn pan_limits_allow_narrow_content_to_touch_each_edge() {
        let rect = map_rect();
        let content = egui::Rect::from_min_max(
            egui::pos2(rect.left() + 120.0, rect.top() + 80.0),
            egui::pos2(rect.right() - 90.0, rect.bottom() - 110.0),
        );
        let limits = map_pan_limits(content, rect);

        assert_eq!(limits.min.x, -120.0);
        assert_eq!(limits.max.x, 90.0);
        assert_eq!(limits.min.y, -80.0);
        assert_eq!(limits.max.y, 110.0);
    }

    #[test]
    fn active_expedition_progress_stays_inside_road_line() {
        let movement = test_movement(ArmyMovementKind::Expedition, 10, 14, None);

        let progress = expedition_movement_line_progress(&movement, 12).unwrap();

        assert!((0.0..=1.0).contains(&progress));
        assert!((progress - 0.4).abs() < 0.001);
    }

    #[test]
    fn two_month_expedition_first_visible_point_uses_first_third() {
        let movement = test_movement(ArmyMovementKind::Expedition, 10, 12, None);

        let progress = expedition_movement_line_progress(&movement, 11).unwrap();

        assert!((progress - (1.0 / 3.0)).abs() < 0.001);
    }

    #[test]
    fn arrived_expedition_without_siege_still_stays_between_cities() {
        let movement = test_movement(ArmyMovementKind::Expedition, 10, 12, None);

        let progress = expedition_movement_line_progress(&movement, 13).unwrap();

        assert!((progress - (2.0 / 3.0)).abs() < 0.001);
    }

    #[test]
    fn siege_ring_radius_hugs_city_marker_icon_outer_radius() {
        let marker_scale = 1.0;

        let city_outer = city_marker_icon_outer_radius(marker_scale);
        let siege_radius = siege_ring_radius(marker_scale);

        assert_eq!(city_outer, 24.0);
        assert_eq!(siege_radius, 26.5);
        assert!(siege_radius > city_outer);
        assert!(siege_radius < 30.0);
    }

    #[test]
    fn expedition_with_siege_started_is_grouped_as_siege() {
        let mut game = test_game_with_movements(vec![
            test_movement(ArmyMovementKind::Expedition, 10, 12, Some(12)),
            test_movement(ArmyMovementKind::Expedition, 10, 13, Some(13)),
        ]);
        game.army_movements[1].issuer_faction_id = "sun_quan".to_string();
        game.army_movements[1].troops = TroopPool::new(0, 400, 0);

        let active_count = active_expedition_movements(&game).count();
        let summaries = siege_summaries(&game);

        assert_eq!(active_count, 0);
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].target_city_id, "target");
        assert_eq!(summaries[0].attacker_count(), 2);
        assert_eq!(summaries[0].total_troops, 1400);
    }

    #[test]
    fn transfer_movement_is_not_map_expedition_overlay() {
        let game = test_game_with_movements(vec![test_movement(
            ArmyMovementKind::Transfer,
            10,
            12,
            None,
        )]);

        assert_eq!(active_expedition_movements(&game).count(), 0);
        assert!(siege_summaries(&game).is_empty());
        assert_eq!(
            expedition_movement_line_progress(&game.army_movements[0], 11),
            None
        );
    }

    #[test]
    fn explicit_cell_city_ids_do_not_match_other_cities_in_same_commandery() {
        let cell = test_cell("颍川郡", &["yingchuan"]);
        let xuchang = test_city("xuchang", "颍川郡");

        assert!(!city_matches_cell(&cell, &xuchang));
    }

    #[test]
    fn cells_without_city_ids_fall_back_to_commandery_name() {
        let cell = test_cell("颍川郡", &[]);
        let xuchang = test_city("xuchang", "颍川郡");

        assert!(city_matches_cell(&cell, &xuchang));
    }

    fn test_game_with_movements(army_movements: Vec<ArmyMovement>) -> GameState {
        let mut factions = BTreeMap::new();
        factions.insert(
            "liu_bei".to_string(),
            Faction {
                id: "liu_bei".to_string(),
                name: "刘备".to_string(),
                ruler_id: "liu_bei".to_string(),
                heir_id: None,
                color: [0.2, 0.6, 0.3],
                selectable: true,
                controlled_by: Controller::Player,
            },
        );
        factions.insert(
            "sun_quan".to_string(),
            Faction {
                id: "sun_quan".to_string(),
                name: "孙权".to_string(),
                ruler_id: "sun_quan".to_string(),
                heir_id: None,
                color: [0.7, 0.35, 0.2],
                selectable: true,
                controlled_by: Controller::RuleAi,
            },
        );

        let mut cities = BTreeMap::new();
        cities.insert("source".to_string(), test_city("source", "沛郡"));
        cities.insert("target".to_string(), test_city("target", "颍川郡"));

        GameState {
            version: SAVE_VERSION,
            scenario_id: "test".to_string(),
            scenario_name: "test".to_string(),
            scenario_era_name: String::new(),
            year: 200,
            month: 1,
            turn: 12,
            player_faction_id: "liu_bei".to_string(),
            factions,
            cities,
            officers: BTreeMap::new(),
            officer_tag_definitions: Vec::new(),
            officer_tag_aliases: BTreeMap::new(),
            roads: vec![Road {
                from: "source".to_string(),
                to: "target".to_string(),
            }],
            diplomacy: BTreeMap::new(),
            pending_diplomacy: Vec::new(),
            pending_commands: Vec::new(),
            army_movements,
            technologies: BTreeMap::new(),
            technology_catalog: TechnologyCatalog::default(),
            events: Vec::new(),
            next_event_sequence: 0,
            dynamic_event_cooldowns: BTreeMap::new(),
            marriages: Vec::new(),
            family_relationships: Vec::new(),
            next_generated_officer_sequence: 0,
            last_lifecycle_year: None,
            officer_recruitments: Vec::new(),
            next_officer_recruitment_sequence: 0,
            applied_event_ids: BTreeSet::new(),
            reports: Vec::new(),
            status: GameStatus::Running,
        }
    }

    fn test_movement(
        kind: ArmyMovementKind,
        departure_turn: u32,
        arrival_turn: u32,
        siege_started_turn: Option<u32>,
    ) -> ArmyMovement {
        ArmyMovement {
            kind,
            issuer_faction_id: "liu_bei".to_string(),
            source_city_id: "source".to_string(),
            target_city_id: "target".to_string(),
            commander_id: "zhang_fei".to_string(),
            officer_ids: vec!["zhang_fei".to_string()],
            troops: TroopPool::new(1000, 0, 0),
            food_supply: 100,
            wounded_troops: TroopPool::default(),
            assignments: Vec::new(),
            siege_started_turn,
            training: 50,
            distance_li: 500,
            departure_turn,
            arrival_turn,
        }
    }

    fn test_cell(name: &str, city_ids: &[&str]) -> TerritoryCell {
        TerritoryCell {
            boundary_id: format!("boundary_{name}"),
            name: name.to_string(),
            parent_id: Some("province_yuzhou".to_string()),
            city_ids: city_ids.iter().map(|city_id| city_id.to_string()).collect(),
            seed: MapPosition { x: 0.0, y: 0.0 },
            points: vec![
                MapPosition { x: 0.0, y: 0.0 },
                MapPosition { x: 1.0, y: 0.0 },
                MapPosition { x: 0.0, y: 1.0 },
            ],
        }
    }

    fn test_city(id: &str, commandery: &str) -> City {
        City {
            id: id.to_string(),
            name: id.to_string(),
            faction_id: "cao_cao".to_string(),
            position: MapPosition { x: 0.0, y: 0.0 },
            level: 1,
            population: 1,
            gold: 0,
            food: 0,
            materials: 0,
            troops: TroopPool::default(),
            wounded_troops: TroopPool::default(),
            training: 0,
            agriculture: 0,
            commerce: 0,
            defense: 0,
            order: 0,
            facilities: Vec::new(),
            governor_id: None,
            profile: Some(CityProfile {
                id: id.to_string(),
                name: id.to_string(),
                province: "豫州".to_string(),
                commandery: commandery.to_string(),
                position: MapPosition { x: 0.0, y: 0.0 },
                scale: CityScale::Commandery,
                strategic_rank: 1,
                agriculture_base: 1,
                commerce_base: 1,
                defense_base: 1,
                population_min: 1,
                population_max: 1,
                confidence: SourceConfidence::Medium,
                notes: String::new(),
            }),
        }
    }
}
