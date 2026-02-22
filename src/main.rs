#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use macroquad::prelude::*;

const TILE: f32 = 32.0;

const MAP_W: usize = 20;
const MAP_H: usize = 15;

const MAP: [[u8; MAP_W]; MAP_H] = [
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

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Node {
    f: i32, // total estimated cost
    g: i32, // cost so far
    x: usize,
    y: usize,
}

// BinaryHeap is max-heap, so reverse ordering (smaller f = "bigger priority")
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.g.cmp(&self.g))
            .then_with(|| other.y.cmp(&self.y))
            .then_with(|| other.x.cmp(&self.x))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan(a: (usize, usize), b: (usize, usize)) -> i32 {
    let dx = if a.0 > b.0 { a.0 - b.0 } else { b.0 - a.0 };
    let dy = if a.1 > b.1 { a.1 - b.1 } else { b.1 - a.1 };
    (dx + dy) as i32
}

fn neighbors4(x: usize, y: usize) -> [(i32, i32); 4] {
    [
        (x as i32 + 1, y as i32),
        (x as i32 - 1, y as i32),
        (x as i32, y as i32 + 1),
        (x as i32, y as i32 - 1),
    ]
}

fn blocked_for_agent(x: usize, y: usize) -> bool {
    // must be inside map and not a wall
    if is_wall(x, y) {
        return true;
    }

    // If you're within 1 tile of a wall, treat it as blocked.
    // This gives your circle "breathing room" so it can turn corners.
    // You can tune this radius: 1 is usually enough for TILE=32 and radius=12.
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

fn astar(start: (usize, usize), goal: (usize, usize)) -> Vec<(usize, usize)> {
    if is_wall(start.0, start.1) || is_wall(goal.0, goal.1) {
        return vec![];
    }
    if start == goal {
        return vec![start];
    }

    let mut open = BinaryHeap::<Node>::new();
    let inf: i32 = i32::MAX / 4;

    let mut g_score = [[inf; MAP_W]; MAP_H];
    let mut came_from = [[None::<(usize, usize)>; MAP_W]; MAP_H];
    let mut closed = [[false; MAP_W]; MAP_H];

    g_score[start.1][start.0] = 0;

    open.push(Node {
        x: start.0,
        y: start.1,
        g: 0,
        f: manhattan(start, goal),
    });

    while let Some(current) = open.pop() {
        let cx = current.x;
        let cy = current.y;

        if closed[cy][cx] {
            continue;
        }
        closed[cy][cx] = true;

        if (cx, cy) == goal {
            let mut path = vec![(cx, cy)];
            let mut cur = (cx, cy);
            while cur != start {
                if let Some(prev) = came_from[cur.1][cur.0] {
                    cur = prev;
                    path.push(cur);
                } else {
                    return vec![];
                }
            }
            path.reverse();
            return path;
        }

        let current_g = g_score[cy][cx];

        for (nx, ny) in neighbors4(cx, cy) {
            if nx < 0 || ny < 0 {
                continue;
            }
            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= MAP_W || ny >= MAP_H {
                continue;
            }

            // IMPORTANT: use inflated blocking for traversal
            if blocked_for_agent(nx, ny) {
                continue;
            }
            if closed[ny][nx] {
                continue;
            }

            let tentative_g = current_g + 1;
            if tentative_g < g_score[ny][nx] {
                came_from[ny][nx] = Some((cx, cy));
                g_score[ny][nx] = tentative_g;

                let h = manhattan((nx, ny), goal);
                open.push(Node {
                    x: nx,
                    y: ny,
                    g: tentative_g,
                    f: tentative_g + h,
                });
            }
        }
    }

    vec![]
}
#[derive(Clone, Copy, Debug)]
struct Actor {
    pos: (f32, f32),
    radius: f32,
    speed: f32,
}

impl Actor {
    pub fn new(x: usize, y: usize, speed: f32) -> Self {
        Self {
            pos: grid_to_world(x, y),
            radius: 12.0,
            speed: speed,
        }
    }
}

fn world_to_grid(x: f32, y: f32) -> (usize, usize) {
    ((x / TILE) as usize, (y / TILE) as usize)
}

fn grid_to_world(x: usize, y: usize) -> (f32, f32) {
    ((x as f32) * TILE + TILE / 2., (y as f32) * TILE + TILE / 2.)
}

fn draw_map() {
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let world = (x as f32 * TILE, y as f32 * TILE);

            if MAP[y][x] == 1 {
                // WALL: solid fill + bright border
                draw_rectangle(world.0, world.1, TILE, TILE, GRAY);
                draw_rectangle_lines(world.0, world.1, TILE, TILE, 2.0, WHITE);
            } else {
                // FLOOR: dark fill + subtle grid line
                draw_rectangle(world.0, world.1, TILE, TILE, DARKGRAY);
                draw_rectangle_lines(world.0, world.1, TILE, TILE, 1.0, BLACK);
            }
        }
    }
}

fn los_grid(a: (usize, usize), b: (usize, usize)) -> bool {
    let aw = grid_to_world(a.0, a.1);
    let bw = grid_to_world(b.0, b.1);
    has_line_of_sight(aw, bw)
}

fn furthest_visible_waypoint(
    enemy_pos: (f32, f32),
    path: &[(usize, usize)],
    from_idx: usize,
) -> usize {
    if path.is_empty() {
        return from_idx;
    }

    let enemy_cell = world_to_grid(enemy_pos.0, enemy_pos.1);
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

fn resolve_circle_map(actor: &mut Actor) {
    let (cx, cy) = world_to_grid(actor.pos.0, actor.pos.1);

    let y0 = cy.saturating_sub(1);
    let x0 = cx.saturating_sub(1);
    let y1 = (cy + 1).min(MAP_H - 1);
    let x1 = (cx + 1).min(MAP_W - 1);

    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if is_wall(tx, ty) {
                let tile_min = (tx as f32 * TILE, ty as f32 * TILE);
                let tile_max = (tile_min.0 + TILE, tile_min.1 + TILE);

                let closest = (
                    actor.pos.0.clamp(tile_min.0, tile_max.0),
                    actor.pos.1.clamp(tile_min.1, tile_max.1),
                );

                let delta = (actor.pos.0 - closest.0, actor.pos.1 - closest.1);
                let dist = (delta.0 * delta.0 + delta.1 * delta.1).sqrt();

                if dist < actor.radius && dist > 0. {
                    let push = (
                        (actor.radius - dist) * delta.0 / dist,
                        (actor.radius - dist) * delta.1 / dist,
                    );
                    actor.pos = (actor.pos.0 + push.0, actor.pos.1 + push.1);
                } else if dist <= 0.0001 {
                    actor.pos = (actor.pos.0 + 0.5, actor.pos.1 + 0.5);
                }
            }
        }
    }
}

fn is_wall(x: usize, y: usize) -> bool {
    if x >= MAP_W || y >= MAP_H {
        return true;
    }
    MAP[y][x] == 1
}

fn has_line_of_sight(a: (f32, f32), b: (f32, f32)) -> bool {
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

    let t_delta_x = if dx != 0. { 1. / dx } else { f32::INFINITY };
    let t_delta_y = if dy != 0. { 1. / dy } else { f32::INFINITY };

    let frac_x = x0 - cx as f32;
    let frac_y = y0 - cy as f32;

    let mut t_max_x = if step_x > 0 {
        (1. - frac_x) * t_delta_x
    } else {
        frac_x * t_delta_x
    };
    let mut t_max_y = if step_y > 0 {
        (1. - frac_y) * t_delta_y
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

fn move_with_slide(actor: &mut Actor, vx: f32, vy: f32) {
    // Try X then Y (slide along walls)
    let old = actor.pos;

    actor.pos = (actor.pos.0 + vx, actor.pos.1);
    resolve_circle_map(actor);
    let after_x = actor.pos;

    actor.pos = (actor.pos.0, actor.pos.1 + vy);
    resolve_circle_map(actor);

    // If X-then-Y resulted in basically no movement, try Y-then-X
    let moved = (actor.pos.0 - old.0).abs() + (actor.pos.1 - old.1).abs();
    if moved < 0.001 {
        actor.pos = old;

        actor.pos = (actor.pos.0, actor.pos.1 + vy);
        resolve_circle_map(actor);

        actor.pos = (actor.pos.0 + vx, actor.pos.1);
        resolve_circle_map(actor);

        // If still stuck, leave it (next step will replan / advance)
        let _ = after_x;
    }
}

#[derive(PartialEq)]
enum GameState {
    Playing,
    GameOver,
}

#[macroquad::main("Enemy Pathfinder")]
async fn main() {
    let mut state = GameState::Playing;

    let mut enemy = Actor::new(1, 2, 180.0);
    let mut player = Actor::new(17, 12, 140.0);

    let mut path: Vec<(usize, usize)> = vec![];
    let mut last_start: (usize, usize) = (0, 0);
    let mut last_goal: (usize, usize) = (0, 0);

    let mut path_index: usize = 0;

    loop {
        if state == GameState::Playing {
            let dt = get_frame_time();

            //Player input
            let mut dir = (0., 0.);
            if is_key_down(KeyCode::W) {
                dir.1 -= 1.;
            }
            if is_key_down(KeyCode::S) {
                dir.1 += 1.;
            }
            if is_key_down(KeyCode::A) {
                dir.0 -= 1.;
            }
            if is_key_down(KeyCode::D) {
                dir.0 += 1.;
            }

            if (dir.0 * dir.0) + (dir.1 * dir.1) > 0. {
                let norm = ((dir.0 * dir.0 + dir.1 * dir.1) as f32).sqrt();
                dir.0 /= norm;
                dir.1 /= norm;

                // move X then collide
                player.pos = (player.pos.0 + dir.0 * player.speed * dt, player.pos.1);
                resolve_circle_map(&mut player);

                // move Y then collide
                player.pos = (player.pos.0, player.pos.1 + dir.1 * player.speed * dt);
                resolve_circle_map(&mut player);
            }

            let los = has_line_of_sight(enemy.pos, player.pos);
            let start_cell = world_to_grid(enemy.pos.0, enemy.pos.1);
            let goal_cell = world_to_grid(player.pos.0, player.pos.1);

            if !los {
                if start_cell != last_start || goal_cell != last_goal || path.is_empty() {
                    path = astar(start_cell, goal_cell);
                    last_start = start_cell;
                    last_goal = goal_cell;
                    path_index = 0;
                }
            } else {
                path.clear();
                path_index = 0;
            }

            if los {
                // direct chase
                let to_player = (player.pos.0 - enemy.pos.0, player.pos.1 - enemy.pos.1);
                let dist = (to_player.0 * to_player.0 + to_player.1 * to_player.1).sqrt();
                if dist > 0.001 {
                    let dir = (to_player.0 / dist, to_player.1 / dist);
                    enemy.pos = (
                        enemy.pos.0 + dir.0 * enemy.speed * dt,
                        enemy.pos.1 + dir.1 * enemy.speed * dt,
                    );
                    resolve_circle_map(&mut enemy);
                }
            } else {
                if path.len() >= 2 {
                    // Keep path_index consistent with where we actually are
                    let enemy_cell = world_to_grid(enemy.pos.0, enemy.pos.1);

                    // If we somehow drifted, snap path_index to the closest matching cell on path (cheap + works)
                    if path_index < path.len() && path[path_index] != enemy_cell {
                        for i in 0..path.len() {
                            if path[i] == enemy_cell {
                                path_index = i;
                                break;
                            }
                        }
                    }

                    // Advance while we're already in the next cell
                    while path_index + 1 < path.len()
                        && world_to_grid(enemy.pos.0, enemy.pos.1) == path[path_index + 1]
                    {
                        path_index += 1;
                    }

                    // If we reached the end, done
                    if path_index + 1 >= path.len() {
                        // we're at goal cell
                    } else {
                        let target_cell = path[path_index + 1];
                        let tgt = grid_to_world(target_cell.0, target_cell.1);

                        let dx = tgt.0 - enemy.pos.0;
                        let dy = tgt.1 - enemy.pos.1;
                        let dist = (dx * dx + dy * dy).sqrt();

                        if dist > 0.001 {
                            let dirx = dx / dist;
                            let diry = dy / dist;

                            let vx = dirx * enemy.speed * dt;
                            let vy = diry * enemy.speed * dt;

                            move_with_slide(&mut enemy, vx, vy);

                            // If after moving we entered the target cell, advance immediately
                            let new_cell = world_to_grid(enemy.pos.0, enemy.pos.1);
                            if new_cell == target_cell && path_index + 1 < path.len() {
                                path_index += 1;
                            }
                        }
                    }

                    // Optional: if we got shoved into a new cell, just replan next frame
                    // (your replan condition already includes start_cell change)
                }
            }

            // Check enemy-player collision
            let dx = enemy.pos.0 - player.pos.0;
            let dy = enemy.pos.1 - player.pos.1;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < enemy.radius + player.radius {
                state = GameState::GameOver;
            }

            clear_background(macroquad::color::BLACK);

            draw_map();

            if path.len() >= 2 {
                for w in path.windows(2) {
                    let a = grid_to_world(w[0].0, w[0].1);
                    let b = grid_to_world(w[1].0, w[1].1);
                    draw_line(a.0, a.1, b.0, b.1, 3.0, SKYBLUE);
                }
            }

            draw_line(
                enemy.pos.0,
                enemy.pos.1,
                player.pos.0,
                player.pos.1,
                1.,
                if los {
                    macroquad::color::GREEN
                } else {
                    macroquad::color::RED
                },
            );

            draw_circle(
                enemy.pos.0,
                enemy.pos.1,
                enemy.radius,
                macroquad::color::BLUE,
            );
            draw_circle(
                player.pos.0,
                player.pos.1,
                player.radius,
                macroquad::color::YELLOW,
            );

            let (pcx, pcy) = world_to_grid(player.pos.0, player.pos.1);
            draw_text(
                &format!("Player grid: ({}, {})", pcx, pcy),
                10.,
                20.,
                30.,
                RED,
            );
            draw_text(
                &format!("path len: {}", path.len()),
                10.0,
                60.0,
                30.0,
                GREEN,
            );

            next_frame().await
        } else if state == GameState::GameOver {
            clear_background(macroquad::color::BLACK);

            if is_key_pressed(KeyCode::R) {
                enemy = Actor::new(1, 2, 180.0);
                player = Actor::new(17, 12, 140.0);
                path.clear();
                path_index = 0;
                last_start = (0, 0);
                last_goal = (0, 0);
                state = GameState::Playing;
            }

            let text = "GAME OVER";
            let sub = "Press R to Restart";

            let font_size = 60.0;
            let text_dim = measure_text(text, None, font_size as u16, 1.0);

            draw_text(
                text,
                screen_width() / 2.0 - text_dim.width / 2.0,
                screen_height() / 2.0,
                font_size,
                RED,
            );

            let sub_size = 30.0;
            let sub_dim = measure_text(sub, None, sub_size as u16, 1.0);

            draw_text(
                sub,
                screen_width() / 2.0 - sub_dim.width / 2.0,
                screen_height() / 2.0 + 50.0,
                sub_size,
                WHITE,
            );

            next_frame().await;
        }
    }
}
