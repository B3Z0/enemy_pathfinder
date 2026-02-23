use crate::actor::Actor;
use crate::map::{MAP_H, MAP_W, TILE, is_wall, world_to_grid};

pub fn resolve_circle_map(actor: &mut Actor) {
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
    let old = actor.pos;

    actor.pos = (actor.pos.0 + vx, actor.pos.1);
    resolve_circle_map(actor);
    let after_x = actor.pos;

    actor.pos = (actor.pos.0, actor.pos.1 + vy);
    resolve_circle_map(actor);

    let moved = (actor.pos.0 - old.0).abs() + (actor.pos.1 - old.1).abs();
    if moved < 0.001 {
        actor.pos = old;

        actor.pos = (actor.pos.0, actor.pos.1 + vy);
        resolve_circle_map(actor);

        actor.pos = (actor.pos.0 + vx, actor.pos.1);
        resolve_circle_map(actor);

        let _ = after_x;
    }
}
