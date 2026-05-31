use crate::game::*;
use bevy_egui::egui;
use std::collections::BTreeMap;

use super::actions::open_city;
use super::state::GameUiState;
use super::style::{draw_strategy_map_background, war_border, war_gold, war_text};
use super::{MAP_MAX_ZOOM, MAP_MIN_ZOOM};

pub(super) fn map_panel(ui: &mut egui::Ui, ui_state: &mut GameUiState) {
    if ui_state.game.is_none() {
        ui.centered_and_justified(|ui| {
            ui.label("尚未开始游戏");
        });
        return;
    }
    let desired = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    draw_strategy_map_background(&painter, rect);

    let scroll_delta = ui.input(|input| input.raw_scroll_delta.y);
    if response.hovered() && scroll_delta.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + scroll_delta * 0.0015).clamp(0.8, 1.25);
        zoom_map(
            ui_state,
            zoom_factor,
            response.hover_pos(),
            Some(rect.center()),
        );
    }

    if response.dragged_by(egui::PointerButton::Primary) {
        ui_state.map_pan += response.drag_delta();
        clamp_map_pan(ui_state, rect);
    }

    let Some(game) = &ui_state.game else {
        return;
    };
    let Some(bounds) = map_bounds(game) else {
        return;
    };

    if ui_state.map_boundaries_enabled
        && let Some(catalog) = &ui_state.map_boundaries
    {
        draw_map_boundaries(&painter, game, catalog, bounds, rect, ui_state);
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
        painter.line_segment(
            [a, b],
            egui::Stroke::new(7.0, egui::Color32::from_rgba_unmultiplied(10, 12, 10, 110)),
        );
        painter.line_segment(
            [a, b],
            egui::Stroke::new(
                3.0,
                egui::Color32::from_rgba_unmultiplied(160, 128, 77, 185),
            ),
        );
    }

    for city in game.cities.values() {
        let pos = map_to_screen(city.position, bounds, rect, ui_state);
        let faction = &game.factions[&city.faction_id];
        let color = faction_color(faction);
        let selected = ui_state.selected_city_id.as_deref() == Some(city.id.as_str());
        let player_owned = city.faction_id == game.player_faction_id;
        draw_city_marker(&painter, pos, city, color, selected, player_owned, ui_state);
    }

    let picked_city = response
        .interact_pointer_pos()
        .and_then(|pointer_pos| city_at_position(game, bounds, rect, pointer_pos, ui_state));

    if (response.clicked() || response.secondary_clicked())
        && let Some(city_id) = picked_city.clone()
    {
        ui_state.selected_city_id = Some(city_id);
        ui_state.city_drawer_open = true;
    }
    if response.double_clicked()
        && let Some(city_id) = picked_city.clone()
    {
        open_city(ui_state, city_id);
    }

    let context_city_id = ui_state.selected_city_id.clone();
    response.context_menu(|ui| {
        if let Some(city_id) = context_city_id.clone()
            && ui.button("打开军令").clicked()
        {
            open_city(ui_state, city_id);
            ui.close();
        }
    });
}

pub(super) fn draw_map_boundaries(
    painter: &egui::Painter,
    game: &GameState,
    catalog: &MapBoundaryCatalog,
    bounds: MapBounds,
    rect: egui::Rect,
    ui_state: &GameUiState,
) {
    let cells = catalog.territory_cells_for_year(game.year);
    if cells.is_empty() {
        return;
    }

    for cell in &cells {
        let fill = territory_cell_fill_color(cell, game, ui_state);
        let points = territory_cell_screen_points(cell, bounds, rect, ui_state);
        paint_boundary_polygon(painter, points, fill, egui::Stroke::NONE);
    }

    for cell in &cells {
        let selected = selected_city_in_cell(cell, game, ui_state);
        let points = territory_cell_screen_points(cell, bounds, rect, ui_state);
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

    for (start, end) in province_border_segments(&cells) {
        let start = map_to_screen(start, bounds, rect, ui_state);
        let end = map_to_screen(end, bounds, rect, ui_state);
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
    ui_state: &GameUiState,
) -> Vec<egui::Pos2> {
    cell.points
        .iter()
        .map(|point| map_to_screen(*point, bounds, rect, ui_state))
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

pub(super) fn territory_cell_fill_color(
    cell: &TerritoryCell,
    game: &GameState,
    ui_state: &GameUiState,
) -> egui::Color32 {
    let selected = selected_city_in_cell(cell, game, ui_state);
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
        entry.1 = entry.1.saturating_add(city.troops);
    }
    let faction_id = counts
        .iter()
        .max_by_key(|(_, (city_count, troops))| (*city_count, *troops))
        .map(|(faction_id, _)| *faction_id)?;
    game.factions.get(faction_id)
}

pub(super) fn selected_city_in_cell(
    cell: &TerritoryCell,
    game: &GameState,
    ui_state: &GameUiState,
) -> bool {
    let Some(selected_city_id) = ui_state.selected_city_id.as_deref() else {
        return false;
    };
    game.cities
        .get(selected_city_id)
        .is_some_and(|city| city_matches_cell(cell, city))
}

pub(super) fn city_matches_cell(cell: &TerritoryCell, city: &City) -> bool {
    if cell.city_ids.iter().any(|city_id| city_id == &city.id) {
        return true;
    }

    let Some(profile) = &city.profile else {
        return false;
    };
    profile.commandery == cell.name
}

pub(super) fn draw_city_marker(
    painter: &egui::Painter,
    pos: egui::Pos2,
    city: &City,
    color: egui::Color32,
    selected: bool,
    player_owned: bool,
    ui_state: &GameUiState,
) {
    let scale = ui_state.map_zoom.sqrt().clamp(0.85, 1.35);
    let pole_top = pos + egui::vec2(-13.0 * scale, -31.0 * scale);
    let pole_bottom = pos + egui::vec2(-13.0 * scale, 18.0 * scale);
    let flag_fill = if player_owned {
        color
    } else {
        color.gamma_multiply(0.82)
    };
    let shadow = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120);

    painter.line_segment(
        [
            pole_top + egui::vec2(2.0, 3.0),
            pole_bottom + egui::vec2(2.0, 3.0),
        ],
        egui::Stroke::new(5.0, shadow),
    );
    painter.line_segment(
        [pole_top, pole_bottom],
        egui::Stroke::new(3.0, egui::Color32::from_rgb(34, 26, 18)),
    );

    let banner = egui::Rect::from_min_size(
        pole_top + egui::vec2(3.0 * scale, 1.0 * scale),
        egui::vec2(43.0 * scale, 20.0 * scale),
    );
    painter.rect(
        banner.translate(egui::vec2(2.0, 2.0)),
        2.0,
        shadow,
        egui::Stroke::NONE,
        egui::StrokeKind::Outside,
    );
    painter.rect(
        banner,
        2.0,
        flag_fill,
        egui::Stroke::new(
            if selected { 2.5 } else { 1.5 },
            if selected {
                war_gold()
            } else {
                egui::Color32::from_rgb(35, 28, 20)
            },
        ),
        egui::StrokeKind::Outside,
    );

    let base = egui::Rect::from_center_size(
        pos + egui::vec2(0.0, 12.0 * scale),
        egui::vec2(42.0 * scale, 17.0 * scale),
    );
    painter.rect(
        base,
        4.0,
        egui::Color32::from_rgba_unmultiplied(20, 18, 14, 222),
        egui::Stroke::new(1.0, flag_fill),
        egui::StrokeKind::Outside,
    );
    painter.text(
        base.center(),
        egui::Align2::CENTER_CENTER,
        compact_troops(city.troops),
        egui::FontId::proportional(12.0 * scale),
        war_text(),
    );

    let label_center = pos + egui::vec2(0.0, 42.0 * scale);
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

pub(super) fn compact_troops(troops: u32) -> String {
    if troops >= 10_000 {
        format!("{:.1}万", troops as f32 / 10_000.0)
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
    let padding = (rect.width().min(rect.height()) * 0.09).clamp(72.0, 118.0);
    let width = (bounds.max_x - bounds.min_x).max(1.0);
    let height = (bounds.max_y - bounds.min_y).max(1.0);
    let x = (position.x - bounds.min_x) / width;
    let y = (position.y - bounds.min_y) / height;
    let base = egui::pos2(
        rect.left() + padding + x * (rect.width() - padding * 2.0).max(1.0),
        rect.bottom() - padding - y * (rect.height() - padding * 2.0).max(1.0),
    );
    rect.center() + (base - rect.center()) * ui_state.map_zoom + ui_state.map_pan
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

pub(super) fn clamp_map_pan(ui_state: &mut GameUiState, rect: egui::Rect) {
    if ui_state.map_zoom <= 1.0 {
        ui_state.map_pan = egui::Vec2::ZERO;
        return;
    }
    let max_x = rect.width() * ui_state.map_zoom;
    let max_y = rect.height() * ui_state.map_zoom;
    ui_state.map_pan.x = ui_state.map_pan.x.clamp(-max_x, max_x);
    ui_state.map_pan.y = ui_state.map_pan.y.clamp(-max_y, max_y);
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
