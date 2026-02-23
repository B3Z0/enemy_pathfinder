pub const TILE: f32 = 32.0;

pub const MAP_W: usize = 20;
pub const MAP_H: usize = 15;

pub const MAP: [[u8; MAP_W]; MAP_H] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1],
    [1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1],
    [1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1],
    [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
    [1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1],
    [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1],
    [1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
    [1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1],
    [1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];

pub fn world_to_grid(x: f32, y: f32) -> (usize, usize) {
    ((x / TILE) as usize, (y / TILE) as usize)
}

pub fn grid_to_world(x: usize, y: usize) -> (f32, f32) {
    (
        (x as f32) * TILE + TILE / 2.0,
        (y as f32) * TILE + TILE / 2.0,
    )
}

pub fn is_wall(x: usize, y: usize) -> bool {
    if x >= MAP_W || y >= MAP_H {
        return true;
    }
    MAP[y][x] == 1
}

pub fn blocked_for_agent(x: usize, y: usize) -> bool {
    if is_wall(x, y) {
        return true;
    }

    let r: isize = 0;
    let xi = x as isize;
    let yi = y as isize;

    for dy in -r..=r {
        for dx in -r..=r {
            let nx = xi + dx;
            let ny = yi + dy;
            if nx < 0 || ny < 0 {
                return true;
            }

            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= MAP_W || ny >= MAP_H {
                return true;
            }
            if MAP[ny][nx] == 1 {
                return true;
            }
        }
    }

    false
}
