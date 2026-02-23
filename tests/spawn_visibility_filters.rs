use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use enemy_pathfinder::map::actor_spawn_from_tiled_json;

fn temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock went backwards")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("enemy_pathfinder_spawn_filters_{nanos}"));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn write_map(dir: &PathBuf, visible_layer: bool, visible_object: bool) -> PathBuf {
    let map_path = dir.join("map.json");
    let layer_visible = if visible_layer { "true" } else { "false" };
    let object_visible = if visible_object { "true" } else { "false" };

    let json = format!(
        r#"{{
  "width": 1,
  "height": 1,
  "tilewidth": 32,
  "tileheight": 32,
  "layers": [
    {{
      "type": "objectgroup",
      "name": "Actors_Layer",
      "visible": {layer_visible},
      "objects": [
        {{
          "type": "Enemy",
          "point": true,
          "visible": {object_visible},
          "x": 10.0,
          "y": 20.0
        }}
      ]
    }}
  ]
}}"#
    );

    fs::write(&map_path, json).expect("failed to write map fixture");
    map_path
}

#[test]
fn hidden_spawn_object_is_ignored_by_json_helper() {
    let dir = temp_dir();
    let map_path = write_map(&dir, true, false);

    let spawn =
        actor_spawn_from_tiled_json(&map_path, "Enemy").expect("json helper should parse fixture");

    assert_eq!(spawn, None);
}

#[test]
fn hidden_actors_layer_is_ignored_by_json_helper() {
    let dir = temp_dir();
    let map_path = write_map(&dir, false, true);

    let spawn =
        actor_spawn_from_tiled_json(&map_path, "Enemy").expect("json helper should parse fixture");

    assert_eq!(spawn, None);
}
