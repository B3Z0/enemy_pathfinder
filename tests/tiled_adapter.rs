use std::path::PathBuf;

use enemy_pathfinder::map::{RuntimeMapAdapter, actor_spawn_from_tiled_json};

fn assets_map_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("map.json")
}

#[test]
fn wall_layer_builds_runtime_solid_grid_and_preserves_oob_wall_behavior() {
    let adapter = RuntimeMapAdapter::from_tiled_json_wall_layer(assets_map_path())
        .expect("Wall_Layer should parse into a runtime adapter");

    assert_eq!(adapter.width, 30);
    assert_eq!(adapter.height, 20);
    assert_eq!(adapter.tile_size, 32.0);

    assert!(adapter.is_wall(0, 0));
    assert!(adapter.is_wall(29, 19));
    assert!(!adapter.is_wall(1, 1));
    assert!(adapter.is_wall(30, 0));
    assert!(adapter.is_wall(0, 20));
}

#[test]
fn actor_spawn_lookup_reads_enemy_and_player_from_actors_layer() {
    let map_path = assets_map_path();

    let enemy = actor_spawn_from_tiled_json(&map_path, "Enemy")
        .expect("enemy spawn lookup should parse")
        .expect("enemy spawn should exist");
    let player = actor_spawn_from_tiled_json(&map_path, "Player")
        .expect("player spawn lookup should parse")
        .expect("player spawn should exist");

    assert!((enemy.0 - 79.9962).abs() < 0.01);
    assert!((enemy.1 - 583.8801).abs() < 0.01);
    assert!((player.0 - 837.6610).abs() < 0.01);
    assert!((player.1 - 343.8916).abs() < 0.01);
}

#[test]
fn runtime_grid_world_conversion_uses_runtime_tile_size() {
    let adapter = RuntimeMapAdapter::from_tiled_json_wall_layer(assets_map_path())
        .expect("Wall_Layer should parse into a runtime adapter");

    let world = adapter.grid_to_world(4, 3);
    assert_eq!(world, (144.0, 112.0));
    assert_eq!(adapter.world_to_grid(world.0, world.1), (4, 3));
}
