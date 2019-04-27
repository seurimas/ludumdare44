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
