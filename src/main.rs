#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macroquad::main("Enemy Pathfinder")]
async fn main() {
    enemy_pathfinder::game::run().await;
}
