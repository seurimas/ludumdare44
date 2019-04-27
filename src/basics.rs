use amethyst::{
    prelude::*,
    ecs::*,
    core::transform::*,
    renderer::Rgba,
};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}



#[derive(Debug, Copy, Clone)]
pub struct Hitbox {
    pub size: f32,
    pub offset: (f32, f32),
    pub debug_color: Option<Rgba>,
}

impl Hitbox {
    pub fn new(size: f32) -> Hitbox {
        Hitbox { size, offset: (0.0, 0.0), debug_color: None }
    }
    pub fn new_at(size: f32, offset: (f32, f32)) -> Hitbox {
        Hitbox { size, offset, debug_color: None }
    }
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
        Physical { hitbox: Hitbox::new(size), is_static: false }
    }
    pub fn new_static(size: f32) -> Physical {
        Physical { hitbox: Hitbox::new(size), is_static: true }
    }
    pub fn depth(&self, other: &Physical, my_pos: (f32, f32), other_pos: (f32, f32)) -> Option<(f32, f32)> {
        self.hitbox.depth(&other.hitbox, my_pos, other_pos)
    }
}

pub const HITSTATE_SIZE: usize = 16;

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
        self.hitboxes[index] = Some(Hitbox::new_at(size, offset));
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
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub enum Rotation {
    East,
    South,
    West,
    North,
}
impl Rotation {
    pub fn rotate(&self, (x, y): (f32, f32)) -> (f32, f32) {
        match self {
            Rotation::East => (x, y),
            Rotation::South => (y, -x),
            Rotation::West => (-x, -y),
            Rotation::North => (-y, x),
        }
    }
    pub fn rotate_euler(&self, transform: &mut Transform) {
        use std::f32;
        transform.set_rotation_euler(0.0, 0.0, match self {
            Rotation::East => 0.0,
            Rotation::South => f32::consts::PI * 3.0 / 2.0,
            Rotation::West => f32::consts::PI,
            Rotation::North => f32::consts::PI / 2.0,
        });
    }
}
#[derive(Debug, Copy, Clone)]
pub struct AnimationFrame {
    pub hitboxes: [Option<Hitbox>; HITSTATE_SIZE],
    pub velocity: (f32, f32),
    pub sprite: Option<usize>,
    pub duration: f32,
}
#[derive(Debug, Clone)]
pub struct HitboxAnimation {
    frames: Vec<AnimationFrame>,
}
impl HitboxAnimation {
    pub fn new() -> HitboxAnimation {
        HitboxAnimation { frames: Vec::new() }
    }
    pub fn add_frame(&mut self, velocity: (f32, f32), duration: f32) -> usize {
        self.frames.push(AnimationFrame {
            hitboxes: Default::default(),
            velocity,
            sprite: None,
            duration,
        });
        self.frames.len() - 1
    }
    pub fn set_hitbox(&mut self, index: usize, hitbox_index: usize, hitbox: Hitbox) {
        self.frames[index].hitboxes[hitbox_index] = Some(hitbox);
    }
    pub fn set_sprite(&mut self, index: usize, sprite: usize) {
        self.frames[index].sprite = Some(sprite);
    }
    fn get_frame_at(&self, progress: f32) -> Option<AnimationFrame> {
        let mut progress_left = progress;
        let mut found_frame = None;
        for frame in self.frames.iter() {
            if progress_left < frame.duration {
                found_frame = Some(*frame);
                break;
            }
            progress_left -= frame.duration;
        }
        found_frame
    }
}
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct AnimationController {
    progress: f32,
    animation: Option<HitboxAnimation>,
}
impl AnimationController {
    pub fn new() -> AnimationController {
        AnimationController {
            progress: 0.0,
            animation: None,
        }
    }
    pub fn start(&mut self, animation: HitboxAnimation) {
        self.animation = Some(animation);
        self.progress = 0.0;
    }
    pub fn step(&mut self, delta_seconds: f32) -> Option<AnimationFrame> {
        self.progress += delta_seconds;
        if let Some(animation) = &self.animation {
            animation.get_frame_at(self.progress)
        } else {
            self.animation = None;
            None
        }
    }
    pub fn active(&self) -> bool {
        !self.animation.is_none()
    }
}
