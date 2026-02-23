use crate::actor::Actor;
use crate::map::{RuntimeMapAdapter, is_wall, map_height, map_tile_size, map_width, world_to_grid};

pub fn resolve_circle_map(actor: &mut Actor) {
    let (cx, cy) = world_to_grid(actor.pos.0, actor.pos.1);
    let map_w = map_width();
    let map_h = map_height();
    let tile = map_tile_size();
    resolve_circle_map_impl(actor, cx, cy, map_w, map_h, tile, |x, y| is_wall(x, y));
}

pub fn resolve_circle_map_with_map(map: &RuntimeMapAdapter, actor: &mut Actor) {
    let (cx, cy) = map.world_to_grid(actor.pos.0, actor.pos.1);
    resolve_circle_map_impl(
        actor,
        cx,
        cy,
        map.width,
        map.height,
        map.tile_size,
        |x, y| map.is_wall(x, y),
    );
}

fn resolve_circle_map_impl<FIsWall>(
    actor: &mut Actor,
    cx: usize,
    cy: usize,
    map_w: usize,
    map_h: usize,
    tile: f32,
    is_wall_fn: FIsWall,
) where
    FIsWall: Fn(usize, usize) -> bool,
{
    if map_w == 0 || map_h == 0 {
        return;
    }

    let y0 = cy.saturating_sub(1);
    let x0 = cx.saturating_sub(1);
    let y1 = (cy + 1).min(map_h - 1);
    let x1 = (cx + 1).min(map_w - 1);

    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if is_wall_fn(tx, ty) {
                let tile_min = (tx as f32 * tile, ty as f32 * tile);
                let tile_max = (tile_min.0 + tile, tile_min.1 + tile);

                let closest = (
                    actor.pos.0.clamp(tile_min.0, tile_max.0),
                    actor.pos.1.clamp(tile_min.1, tile_max.1),
                );

                let delta = (actor.pos.0 - closest.0, actor.pos.1 - closest.1);
                let dist = (delta.0 * delta.0 + delta.1 * delta.1).sqrt();

                if dist < actor.radius && dist > 0.0 {
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

pub fn move_with_slide(actor: &mut Actor, vx: f32, vy: f32) {
    move_with_slide_impl(actor, vx, vy, |actor| resolve_circle_map(actor));
}

pub fn move_with_slide_with_map(map: &RuntimeMapAdapter, actor: &mut Actor, vx: f32, vy: f32) {
    move_with_slide_impl(actor, vx, vy, |actor| {
        resolve_circle_map_with_map(map, actor)
    });
}

fn move_with_slide_impl<FResolve>(actor: &mut Actor, vx: f32, vy: f32, mut resolve: FResolve)
where
    FResolve: FnMut(&mut Actor),
{
    let old = actor.pos;

    actor.pos = (actor.pos.0 + vx, actor.pos.1);
    resolve(actor);
    let after_x = actor.pos;

    actor.pos = (actor.pos.0, actor.pos.1 + vy);
    resolve(actor);

    let moved = (actor.pos.0 - old.0).abs() + (actor.pos.1 - old.1).abs();
    if moved < 0.001 {
        actor.pos = old;

        actor.pos = (actor.pos.0, actor.pos.1 + vy);
        resolve(actor);

        actor.pos = (actor.pos.0 + vx, actor.pos.1);
        resolve(actor);

        let _ = after_x;
    }
}
