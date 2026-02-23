use enemy_pathfinder::map::{grid_to_world, is_wall, world_to_grid};
use enemy_pathfinder::pathfinding::{astar, manhattan};
use enemy_pathfinder::visibility::los_grid;

#[test]
fn manhattan_distance_matches_expected() {
    assert_eq!(manhattan((1, 2), (4, 6)), 7);
    assert_eq!(manhattan((4, 6), (1, 2)), 7);
    assert_eq!(manhattan((3, 3), (3, 3)), 0);
}

#[test]
fn grid_world_conversion_round_trips_for_tile_centers() {
    let world = grid_to_world(17, 12);
    assert_eq!(world_to_grid(world.0, world.1), (17, 12));
}

#[test]
fn line_of_sight_detects_open_and_blocked_segments() {
    assert!(los_grid((1, 1), (5, 1)));
    assert!(!los_grid((1, 1), (18, 2)));
}

#[test]
fn astar_returns_empty_for_wall_start_or_goal() {
    assert!(is_wall(0, 0));
    assert!(astar((0, 0), (1, 1)).is_empty());
    assert!(astar((1, 1), (0, 0)).is_empty());
}

#[test]
fn astar_returns_path_with_expected_endpoints_for_known_open_cells() {
    let path = astar((1, 2), (18, 12));

    assert!(!path.is_empty());
    assert_eq!(path.first().copied(), Some((1, 2)));
    assert_eq!(path.last().copied(), Some((18, 12)));
}
