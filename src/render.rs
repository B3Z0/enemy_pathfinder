use macroquad::prelude::*;

use crate::map::{MAP, MAP_H, MAP_W, TILE};

pub fn draw_map() {
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let world = (x as f32 * TILE, y as f32 * TILE);

            if MAP[y][x] == 1 {
                draw_rectangle(world.0, world.1, TILE, TILE, GRAY);
                draw_rectangle_lines(world.0, world.1, TILE, TILE, 2.0, WHITE);
            } else {
                draw_rectangle(world.0, world.1, TILE, TILE, DARKGRAY);
                draw_rectangle_lines(world.0, world.1, TILE, TILE, 1.0, BLACK);
            }
        }
    }
}
