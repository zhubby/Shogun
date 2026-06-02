use shogun::game::*;
use std::collections::BTreeSet;

#[test]
fn boundary_asset_loads_with_non_empty_provinces_and_commanderies() {
    let catalog = MapBoundaryCatalog::from_path(MAP_BOUNDARY_ASSET_PATH).unwrap();
    let boundaries: Vec<_> = catalog.boundaries_for_year(180).collect();

    let province_count = boundaries
        .iter()
        .filter(|boundary| boundary.level == MapBoundaryLevel::Province)
        .count();
    let commandery_count = boundaries
        .iter()
        .filter(|boundary| boundary.level == MapBoundaryLevel::Commandery)
        .count();

    assert_eq!(province_count, 13);
    assert_eq!(commandery_count, 70);
    assert!(boundaries.iter().all(|boundary| boundary.points.len() >= 3));

    let cells = catalog.territory_cells_for_year(180);
    assert_eq!(cells.len(), commandery_count);
    assert!(cells.iter().all(|cell| cell.points.len() >= 3));
}

#[test]
fn paired_cities_use_separate_map_boundaries() {
    let catalog = MapBoundaryCatalog::from_path(MAP_BOUNDARY_ASSET_PATH).unwrap();
    let boundaries: Vec<_> = catalog
        .boundaries_for_year(190)
        .filter(|boundary| boundary.level == MapBoundaryLevel::Commandery)
        .collect();

    for (first_city_id, second_city_id) in [
        ("xuchang", "yingchuan"),
        ("xiangyang", "jiangling"),
        ("jianye", "danyang"),
        ("hefei", "lujiang"),
    ] {
        let first = boundary_for_city(&boundaries, first_city_id);
        let second = boundary_for_city(&boundaries, second_city_id);

        assert_ne!(first.id, second.id);
        assert_eq!(first.parent_id, second.parent_id);
    }
}

#[test]
fn reported_city_markers_are_inside_assigned_map_cells() {
    let catalog = MapBoundaryCatalog::from_path(MAP_BOUNDARY_ASSET_PATH).unwrap();
    let historical = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();
    let game = historical.build_game("ad180", "yellow_turban").unwrap();
    let cells = catalog.territory_cells_for_year(game.year);

    for city_id in [
        "xiangyang",
        "jiangling",
        "jianye",
        "danyang",
        "hefei",
        "lujiang",
        "xiapi",
        "donghai",
    ] {
        let city = &game.cities[city_id];
        let cell = cells
            .iter()
            .find(|cell| cell.city_ids.iter().any(|id| id == city_id))
            .unwrap_or_else(|| panic!("missing territory cell for {city_id}"));

        assert!(
            polygon_contains_position(city.position, &cell.points),
            "{} marker at ({}, {}) is outside {}",
            city.id,
            city.position.x,
            city.position.y,
            cell.boundary_id
        );
    }
}

#[test]
fn historical_scenario_cities_have_boundary_coverage() {
    let catalog = MapBoundaryCatalog::from_path(MAP_BOUNDARY_ASSET_PATH).unwrap();
    let historical = SqliteHistoricalCatalog::in_memory_from_seed().unwrap();
    let game = historical.build_game("ad180", "yellow_turban").unwrap();

    let provinces: BTreeSet<_> = catalog
        .boundaries_for_year(game.year)
        .filter(|boundary| boundary.level == MapBoundaryLevel::Province)
        .map(|boundary| boundary.name.as_str())
        .collect();
    let commandery_city_ids: BTreeSet<_> = catalog
        .boundaries_for_year(game.year)
        .filter(|boundary| boundary.level == MapBoundaryLevel::Commandery)
        .flat_map(|boundary| boundary.city_ids.iter().map(String::as_str))
        .collect();

    for city in game.cities.values() {
        let profile = city.profile.as_ref().unwrap();
        assert!(
            provinces.contains(profile.province.as_str()),
            "missing province boundary for {}",
            profile.province
        );
        assert!(
            commandery_city_ids.contains(city.id.as_str()),
            "missing commandery boundary coverage for {} ({})",
            city.id,
            profile.commandery
        );
    }
}

fn boundary_for_city<'a>(boundaries: &'a [&MapBoundary], city_id: &str) -> &'a MapBoundary {
    boundaries
        .iter()
        .copied()
        .find(|boundary| boundary.city_ids.len() == 1 && boundary.city_ids[0] == city_id)
        .unwrap_or_else(|| panic!("{city_id} boundary"))
}

fn polygon_contains_position(position: MapPosition, polygon: &[MapPosition]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    for index in 0..polygon.len() {
        if position_on_segment(
            position,
            polygon[index],
            polygon[(index + 1) % polygon.len()],
        ) {
            return true;
        }
    }

    let mut inside = false;
    for index in 0..polygon.len() {
        let current = polygon[index];
        let previous = polygon[(index + polygon.len() - 1) % polygon.len()];
        if (current.y > position.y) != (previous.y > position.y) {
            let intersection_x = (previous.x - current.x) * (position.y - current.y)
                / (previous.y - current.y)
                + current.x;
            if position.x < intersection_x {
                inside = !inside;
            }
        }
    }
    inside
}

fn position_on_segment(position: MapPosition, start: MapPosition, end: MapPosition) -> bool {
    const EPSILON: f32 = 0.001;

    let cross =
        (position.y - start.y) * (end.x - start.x) - (position.x - start.x) * (end.y - start.y);
    if cross.abs() > EPSILON {
        return false;
    }

    position.x >= start.x.min(end.x) - EPSILON
        && position.x <= start.x.max(end.x) + EPSILON
        && position.y >= start.y.min(end.y) - EPSILON
        && position.y <= start.y.max(end.y) + EPSILON
}
