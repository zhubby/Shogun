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
    assert_eq!(commandery_count, 67);
    assert!(boundaries.iter().all(|boundary| boundary.points.len() >= 3));

    let cells = catalog.territory_cells_for_year(180);
    assert_eq!(cells.len(), commandery_count);
    assert!(cells.iter().all(|cell| cell.points.len() >= 3));
}

#[test]
fn xuchang_and_yingchuan_use_separate_map_boundaries() {
    let catalog = MapBoundaryCatalog::from_path(MAP_BOUNDARY_ASSET_PATH).unwrap();
    let boundaries: Vec<_> = catalog
        .boundaries_for_year(190)
        .filter(|boundary| boundary.level == MapBoundaryLevel::Commandery)
        .collect();

    let xuchang = boundaries
        .iter()
        .find(|boundary| boundary.city_ids == ["xuchang"])
        .expect("xuchang boundary");
    let yingchuan = boundaries
        .iter()
        .find(|boundary| boundary.city_ids == ["yingchuan"])
        .expect("yingchuan boundary");

    assert_ne!(xuchang.id, yingchuan.id);
    assert_eq!(xuchang.parent_id, yingchuan.parent_id);
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
