use amethyst::{
    prelude::*,
    ecs::*,
};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Player {
    pub can_move: bool,
    pub walk_accel: f32,
    pub walk_speed: f32,
}
impl Player {
    pub fn new() -> Player {
        Player {
            can_move: true,
            walk_accel: 400.0,
            walk_speed: 100.0,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Physical {
    pub size: f32,
    pub is_static: bool,
}

impl Physical {
    pub fn depth(&self, other: &Physical, (mx, my): (f32, f32), (ox, oy): (f32, f32)) -> Option<(f32, f32)> {
        let square_size = self.size + other.size;
        let dx = ox - mx;
        let dy = oy - my;
        if dx.abs() < square_size && dy.abs() < square_size {
            Some((square_size - dx.abs(), square_size - dy.abs()))
        } else {
            None
        }
    }
}
