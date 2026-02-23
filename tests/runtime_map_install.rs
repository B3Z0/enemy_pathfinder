use std::path::PathBuf;

use enemy_pathfinder::map::{RuntimeMapAdapter, install_runtime_map_compatible};

fn assets_map_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("map.json")
}

#[test]
fn repeated_runtime_map_install_is_idempotent_for_same_map() {
    let adapter = RuntimeMapAdapter::from_tiled_json_wall_layer(assets_map_path())
        .expect("assets/map.json Wall_Layer should parse");

    install_runtime_map_compatible(adapter.clone()).expect("first install should succeed");
    install_runtime_map_compatible(adapter).expect("second install with same map should be ok");
}
