use macroquad::prelude::*;
use macroquad_tiled_clone::Map as TiledMap;

use crate::actor::Actor;
use crate::map::{RuntimeMapAdapter, actor_spawn_from_tiled_map, end_zone_from_tiled_map};
use crate::pathfinding::astar_with_map;
use crate::physics::{move_with_slide_with_map, resolve_circle_map_with_map};
use crate::render::draw_map;
use crate::visibility::has_line_of_sight_with_map;

#[derive(PartialEq)]
enum GameState {
    StartScreen,
    Playing,
    Win,
    GameOver,
}

pub async fn run() {
    let mut game = Game::new().await;

    loop {
        game.frame().await;
    }
}

struct Game {
    collision_map: RuntimeMapAdapter,
    _tiled_map: TiledMap,
    state: GameState,
    enemy_spawn_pos: (f32, f32),
    player_spawn_pos: (f32, f32),
    end_zone: Option<Rect>,
    enemy: Actor,
    player: Actor,
    path: Vec<(usize, usize)>,
    last_start: (usize, usize),
    last_goal: (usize, usize),
    path_index: usize,
}

impl Game {
    async fn new() -> Self {
        let tiled_map = TiledMap::load("assets/map.json")
            .await
            .expect("Failed to load Tiled map: assets/map.json");
        let collision_map = RuntimeMapAdapter::from_tiled_json_wall_layer("assets/map.json")
            .expect("Failed to build collision grid from Wall_Layer in assets/map.json");

        let enemy_spawn_pos = actor_spawn_from_tiled_map(&tiled_map, "Enemy")
            .unwrap_or_else(|| collision_map.grid_to_world(1, 2));
        let player_spawn_pos = actor_spawn_from_tiled_map(&tiled_map, "Player")
            .unwrap_or_else(|| collision_map.grid_to_world(17, 12));
        let end_zone =
            end_zone_from_tiled_map(&tiled_map).map(|(x, y, w, h)| Rect::new(x, y, w, h));

        let enemy = Actor::new_world(enemy_spawn_pos, 180.0);
        let player = Actor::new_world(player_spawn_pos, 140.0);

        Self {
            collision_map,
            _tiled_map: tiled_map,
            state: GameState::StartScreen,
            enemy_spawn_pos,
            player_spawn_pos,
            end_zone,
            enemy,
            player,
            path: vec![],
            last_start: (0, 0),
            last_goal: (0, 0),
            path_index: 0,
        }
    }

    async fn frame(&mut self) {
        match self.state {
            GameState::StartScreen => self.frame_start_screen().await,
            GameState::Playing => self.frame_playing().await,
            GameState::Win => self.frame_win_screen().await,
            GameState::GameOver => self.frame_game_over().await,
        }
    }

    async fn frame_start_screen(&mut self) {
        clear_background(macroquad::color::BLACK);

        let text = "Press SPACE to Start";
        let font_size = 42.0;
        let text_dim = measure_text(text, None, font_size as u16, 1.0);

        draw_text(
            text,
            screen_width() / 2.0 - text_dim.width / 2.0,
            screen_height() / 2.0,
            font_size,
            WHITE,
        );

        if is_key_pressed(KeyCode::Space) {
            self.state = GameState::Playing;
        }

        next_frame().await;
    }

    async fn frame_playing(&mut self) {
        let dt = get_frame_time();

        self.update_player(dt);

        let los = has_line_of_sight_with_map(&self.collision_map, self.enemy.pos, self.player.pos);
        let start_cell = self
            .collision_map
            .world_to_grid(self.enemy.pos.0, self.enemy.pos.1);
        let goal_cell = self
            .collision_map
            .world_to_grid(self.player.pos.0, self.player.pos.1);

        self.update_path_cache(los, start_cell, goal_cell);

        if los {
            self.direct_chase(dt);
        } else {
            self.follow_path(dt);
        }

        if self.player_reached_end_zone() {
            self.state = GameState::Win;
        } else {
            self.update_game_over_collision();
        }

        self.draw_playing(los);
        next_frame().await;
    }

    async fn frame_win_screen(&mut self) {
        clear_background(macroquad::color::BLACK);

        if is_key_pressed(KeyCode::R) {
            self.reset();
        }

        let text = "YOU WIN";
        let sub = "Press R to Restart";

        let font_size = 60.0;
        let text_dim = measure_text(text, None, font_size as u16, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - text_dim.width / 2.0,
            screen_height() / 2.0,
            font_size,
            GREEN,
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

    async fn frame_game_over(&mut self) {
        clear_background(macroquad::color::BLACK);

        if is_key_pressed(KeyCode::R) {
            self.reset();
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

    fn reset(&mut self) {
        self.enemy = Actor::new_world(self.enemy_spawn_pos, 180.0);
        self.player = Actor::new_world(self.player_spawn_pos, 140.0);
        self.path.clear();
        self.path_index = 0;
        self.last_start = (0, 0);
        self.last_goal = (0, 0);
        self.state = GameState::Playing;
    }

    fn update_player(&mut self, dt: f32) {
        let mut dir: (f32, f32) = (0.0, 0.0);
        if is_key_down(KeyCode::W) {
            dir.1 -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            dir.1 += 1.0;
        }
        if is_key_down(KeyCode::A) {
            dir.0 -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            dir.0 += 1.0;
        }

        if (dir.0 * dir.0) + (dir.1 * dir.1) <= 0.0 {
            return;
        }

        let norm: f32 = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
        dir.0 /= norm;
        dir.1 /= norm;

        self.player.pos = (
            self.player.pos.0 + dir.0 * self.player.speed * dt,
            self.player.pos.1,
        );
        resolve_circle_map_with_map(&self.collision_map, &mut self.player);

        self.player.pos = (
            self.player.pos.0,
            self.player.pos.1 + dir.1 * self.player.speed * dt,
        );
        resolve_circle_map_with_map(&self.collision_map, &mut self.player);
    }

    fn update_path_cache(
        &mut self,
        los: bool,
        start_cell: (usize, usize),
        goal_cell: (usize, usize),
    ) {
        if !los {
            if start_cell != self.last_start || goal_cell != self.last_goal || self.path.is_empty()
            {
                self.path = astar_with_map(&self.collision_map, start_cell, goal_cell);
                self.last_start = start_cell;
                self.last_goal = goal_cell;
                self.path_index = 0;
            }
        } else {
            self.path.clear();
            self.path_index = 0;
        }
    }

    fn direct_chase(&mut self, dt: f32) {
        let to_player = (
            self.player.pos.0 - self.enemy.pos.0,
            self.player.pos.1 - self.enemy.pos.1,
        );
        let dist = (to_player.0 * to_player.0 + to_player.1 * to_player.1).sqrt();
        if dist <= 0.001 {
            return;
        }

        let dir = (to_player.0 / dist, to_player.1 / dist);
        self.enemy.pos = (
            self.enemy.pos.0 + dir.0 * self.enemy.speed * dt,
            self.enemy.pos.1 + dir.1 * self.enemy.speed * dt,
        );
        resolve_circle_map_with_map(&self.collision_map, &mut self.enemy);
    }

    fn follow_path(&mut self, dt: f32) {
        if self.path.len() < 2 {
            return;
        }

        let enemy_cell = self
            .collision_map
            .world_to_grid(self.enemy.pos.0, self.enemy.pos.1);
        self.sync_path_index(enemy_cell);

        while self.path_index + 1 < self.path.len()
            && self
                .collision_map
                .world_to_grid(self.enemy.pos.0, self.enemy.pos.1)
                == self.path[self.path_index + 1]
        {
            self.path_index += 1;
        }

        if self.path_index + 1 >= self.path.len() {
            return;
        }

        let target_cell = self.path[self.path_index + 1];
        let tgt = self
            .collision_map
            .grid_to_world(target_cell.0, target_cell.1);

        let dx = tgt.0 - self.enemy.pos.0;
        let dy = tgt.1 - self.enemy.pos.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= 0.001 {
            return;
        }

        let dirx = dx / dist;
        let diry = dy / dist;

        let vx = dirx * self.enemy.speed * dt;
        let vy = diry * self.enemy.speed * dt;

        move_with_slide_with_map(&self.collision_map, &mut self.enemy, vx, vy);

        let new_cell = self
            .collision_map
            .world_to_grid(self.enemy.pos.0, self.enemy.pos.1);
        if new_cell == target_cell && self.path_index + 1 < self.path.len() {
            self.path_index += 1;
        }
    }

    fn sync_path_index(&mut self, enemy_cell: (usize, usize)) {
        if self.path_index < self.path.len() && self.path[self.path_index] != enemy_cell {
            for i in 0..self.path.len() {
                if self.path[i] == enemy_cell {
                    self.path_index = i;
                    break;
                }
            }
        }
    }

    fn update_game_over_collision(&mut self) {
        let dx = self.enemy.pos.0 - self.player.pos.0;
        let dy = self.enemy.pos.1 - self.player.pos.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < self.enemy.radius + self.player.radius {
            self.state = GameState::GameOver;
        }
    }

    fn player_reached_end_zone(&self) -> bool {
        let Some(zone) = self.end_zone else {
            return false;
        };
        zone.contains(vec2(self.player.pos.0, self.player.pos.1))
    }

    fn draw_playing(&mut self, los: bool) {
        clear_background(macroquad::color::BLACK);

        draw_map(&mut self._tiled_map);

        if self.path.len() >= 2 {
            for w in self.path.windows(2) {
                let a = self.collision_map.grid_to_world(w[0].0, w[0].1);
                let b = self.collision_map.grid_to_world(w[1].0, w[1].1);
                draw_line(a.0, a.1, b.0, b.1, 3.0, SKYBLUE);
            }
        }

        draw_line(
            self.enemy.pos.0,
            self.enemy.pos.1,
            self.player.pos.0,
            self.player.pos.1,
            1.0,
            if los {
                macroquad::color::GREEN
            } else {
                macroquad::color::RED
            },
        );

        draw_circle(
            self.enemy.pos.0,
            self.enemy.pos.1,
            self.enemy.radius,
            macroquad::color::BLUE,
        );
        draw_circle(
            self.player.pos.0,
            self.player.pos.1,
            self.player.radius,
            macroquad::color::YELLOW,
        );

        let (pcx, pcy) = self
            .collision_map
            .world_to_grid(self.player.pos.0, self.player.pos.1);
        draw_text(
            &format!("Player grid: ({}, {})", pcx, pcy),
            10.0,
            20.0,
            30.0,
            RED,
        );
        draw_text(
            &format!("path len: {}", self.path.len()),
            10.0,
            60.0,
            30.0,
            GREEN,
        );
    }
}
