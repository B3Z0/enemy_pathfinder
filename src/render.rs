use macroquad::prelude::*;
use macroquad_tiled_clone::Map as TiledMap;

pub fn draw_map(tiled_map: &mut TiledMap) {
    let view_min = vec2(0.0, 0.0);
    let view_max = vec2(screen_width(), screen_height());
    tiled_map.draw(view_min, view_max);
}
