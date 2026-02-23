use std::path::PathBuf;

use enemy_pathfinder::map::RuntimeMapAdapter;
use enemy_pathfinder::pathfinding::{astar_with_map, manhattan};
use enemy_pathfinder::visibility::los_grid_with_map;

fn assets_map_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("map.json")
}

fn load_runtime_map() -> RuntimeMapAdapter {
    RuntimeMapAdapter::from_tiled_json_wall_layer(assets_map_path())
        .expect("assets/map.json Wall_Layer should parse")
}

#[test]
fn manhattan_distance_matches_expected() {
    assert_eq!(manhattan((1, 2), (4, 6)), 7);
    assert_eq!(manhattan((4, 6), (1, 2)), 7);
    assert_eq!(manhattan((3, 3), (3, 3)), 0);
}

#[test]
fn grid_world_conversion_round_trips_for_tile_centers() {
    let map = load_runtime_map();
    let world = map.grid_to_world(26, 10);
    assert_eq!(map.world_to_grid(world.0, world.1), (26, 10));
}

#[test]
fn line_of_sight_detects_open_and_blocked_segments() {
    let map = load_runtime_map();
    assert!(los_grid_with_map(&map, (1, 1), (5, 1)));
    assert!(!los_grid_with_map(&map, (1, 3), (12, 3)));
}

#[test]
fn astar_returns_empty_for_wall_start_or_goal() {
    let map = load_runtime_map();
    assert!(map.is_wall(0, 0));
    assert!(astar_with_map(&map, (0, 0), (1, 1)).is_empty());
    assert!(astar_with_map(&map, (1, 1), (0, 0)).is_empty());
}

#[test]
fn astar_returns_path_with_expected_endpoints_for_known_open_cells() {
    let map = load_runtime_map();
    let path = astar_with_map(&map, (1, 3), (12, 3));

    assert!(!path.is_empty());
    assert_eq!(path.first().copied(), Some((1, 3)));
    assert_eq!(path.last().copied(), Some((12, 3)));
}
