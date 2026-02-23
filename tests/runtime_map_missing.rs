use enemy_pathfinder::map::{
    try_blocked_for_agent, try_grid_to_world, try_is_wall, try_map_height, try_map_tile_size,
    try_map_width, try_world_to_grid,
};

#[test]
fn try_map_queries_return_none_when_runtime_map_not_installed() {
    assert_eq!(try_map_width(), None);
    assert_eq!(try_map_height(), None);
    assert_eq!(try_map_tile_size(), None);
    assert_eq!(try_world_to_grid(0.0, 0.0), None);
    assert_eq!(try_grid_to_world(0, 0), None);
    assert_eq!(try_is_wall(0, 0), None);
    assert_eq!(try_blocked_for_agent(0, 0), None);
}
