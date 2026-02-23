use crate::map::{TILE, grid_to_world, is_wall};

pub fn los_grid(a: (usize, usize), b: (usize, usize)) -> bool {
    let aw = grid_to_world(a.0, a.1);
    let bw = grid_to_world(b.0, b.1);
    has_line_of_sight(aw, bw)
}

pub fn has_line_of_sight(a: (f32, f32), b: (f32, f32)) -> bool {
    let x0 = a.0 / TILE;
    let y0 = a.1 / TILE;
    let x1 = b.0 / TILE;
    let y1 = b.1 / TILE;

    let mut cx = x0 as usize;
    let mut cy = y0 as usize;
    let end_cx = x1 as usize;
    let end_cy = y1 as usize;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let t_delta_x = if dx != 0.0 { 1.0 / dx } else { f32::INFINITY };
    let t_delta_y = if dy != 0.0 { 1.0 / dy } else { f32::INFINITY };

    let frac_x = x0 - cx as f32;
    let frac_y = y0 - cy as f32;

    let mut t_max_x = if step_x > 0 {
        (1.0 - frac_x) * t_delta_x
    } else {
        frac_x * t_delta_x
    };
    let mut t_max_y = if step_y > 0 {
        (1.0 - frac_y) * t_delta_y
    } else {
        frac_y * t_delta_y
    };

    if is_wall(cx, cy) {
        return false;
    }

    while cx != end_cx || cy != end_cy {
        if t_max_x < t_max_y {
            t_max_x += t_delta_x;
            let nx = cx as isize + step_x;
            if nx < 0 {
                return false;
            }
            cx = nx as usize;
        } else {
            t_max_y += t_delta_y;
            let ny = cy as isize + step_y;
            if ny < 0 {
                return false;
            }
            cy = ny as usize;
        }

        if is_wall(cx, cy) {
            return false;
        }
    }
    true
}

pub fn furthest_visible_waypoint(
    enemy_pos: (f32, f32),
    path: &[(usize, usize)],
    from_idx: usize,
) -> usize {
    if path.is_empty() {
        return from_idx;
    }

    let enemy_cell = crate::map::world_to_grid(enemy_pos.0, enemy_pos.1);
    let mut best = from_idx;

    for i in (from_idx + 1)..path.len() {
        if los_grid(enemy_cell, path[i]) {
            best = i;
        } else {
            break;
        }
    }

    best
}
