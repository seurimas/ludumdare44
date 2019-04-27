use amethyst::{
    prelude::*,
    ecs::*,
    renderer::Rgba,
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



#[derive(Debug)]
pub struct Hitbox {
    pub size: f32,
    pub offset: (f32, f32),
    pub debug_color: Option<Rgba>,
}

impl Hitbox {
    pub fn depth(&self, other: &Hitbox, (mx, my): (f32, f32), (ox, oy): (f32, f32)) -> Option<(f32, f32)> {
        let square_size = self.size + other.size;
        let dx = (ox + other.offset.0) - (mx + self.offset.0);
        let dy = (oy + other.offset.1) - (my + self.offset.1);
        if dx.abs() < square_size && dy.abs() < square_size {
            Some((square_size - dx.abs(), square_size - dy.abs()))
        } else {
            None
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Physical {
    pub hitbox: Hitbox,
    pub is_static: bool,
}

impl Physical {
    pub fn new(size: f32) -> Physical {
        Physical { hitbox: Hitbox { size, offset: (0.0, 0.0), debug_color: None }, is_static: false }
    }
    pub fn new_static(size: f32) -> Physical {
        Physical { hitbox: Hitbox { size, offset: (0.0, 0.0), debug_color: None }, is_static: true }
    }
    pub fn depth(&self, other: &Physical, my_pos: (f32, f32), other_pos: (f32, f32)) -> Option<(f32, f32)> {
        self.hitbox.depth(&other.hitbox, my_pos, other_pos)
    }
}

const HITSTATE_SIZE: usize = 16;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct HitState {
    hitboxes: [Option<Hitbox>; HITSTATE_SIZE],
    bounding: Hitbox,
}

impl HitState {
    pub fn new() -> HitState {
        HitState {
            hitboxes: Default::default(),
            bounding: Hitbox {
                size: 0.0,
                offset: (0.0, 0.0),
                debug_color: Some([0.0, 1.0, 1.0, 1.0].into()),
            }
        }
    }
    pub fn set(&mut self, index: usize, size: f32, offset: (f32, f32)) {
        self.hitboxes[index] = Some(Hitbox { size, offset, debug_color: None });
        self.recalculate_bounding();
    }
    pub fn get_all(&self) -> &[Option<Hitbox>; HITSTATE_SIZE] {
        &self.hitboxes
    }
    pub fn recalculate_bounding(&mut self) {
        if let Some(extent) = self.hitboxes
            .iter()
            .fold(None, |extent, next| match (extent, next) {
                (None, Some(next)) => Some(next.offset.0.abs().max(next.offset.1.abs())),
                (Some(y), Some(next)) => Some((next.offset.0.abs().max(next.offset.1.abs())).max(y)),
                _ => extent,
            }) {
            self.bounding.size = extent;
        }
    }
}
