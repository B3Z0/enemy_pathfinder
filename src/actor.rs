use crate::map::grid_to_world;

#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub pos: (f32, f32),
    pub radius: f32,
    pub speed: f32,
}

impl Actor {
    pub fn new(x: usize, y: usize, speed: f32) -> Self {
        Self {
            pos: grid_to_world(x, y),
            radius: 12.0,
            speed,
        }
    }

    pub fn new_world(pos: (f32, f32), speed: f32) -> Self {
        Self {
            pos,
            radius: 12.0,
            speed,
        }
    }
}
