#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use macroquad::prelude::Conf;

fn window_conf() -> Conf {
    let mut conf = Conf {
        window_title: "Enemy Pathfinder".to_owned(),
        ..Default::default()
    };

    if let Ok(map) =
        enemy_pathfinder::map::RuntimeMapAdapter::from_tiled_json_wall_layer("assets/map.json")
    {
        conf.window_width = (map.width as f32 * map.tile_size).round() as i32;
        conf.window_height = (map.height as f32 * map.tile_size).round() as i32;
    }

    conf
}

#[macroquad::main(window_conf)]
async fn main() {
    enemy_pathfinder::game::run().await;
}
