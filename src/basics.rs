use amethyst::{
    prelude::*,
    ecs::*,
    core::transform::*,
    renderer::*,
};

pub const stage: (f32, f32) = (200.0, 150.0);

pub struct MainSprite(pub SpriteSheetHandle);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

pub type HitboxCollision = (f32, f32, f32, f32);

#[derive(Debug, Copy, Clone)]
pub struct Hitbox {
    pub width: f32,
    pub height: f32,
    pub offset: (f32, f32),
    pub debug_color: Option<Rgba>,
}

impl Hitbox {
    pub fn new(size: f32) -> Hitbox {
        Hitbox { width: size * 2.0, height: size * 2.0, offset: (0.0, 0.0), debug_color: None }
    }
    pub fn new_at(size: f32, offset: (f32, f32)) -> Hitbox {
        Hitbox { width: size * 2.0, height: size * 2.0, offset, debug_color: None }
    }
    pub fn new_at_rect(width: f32, height: f32, offset: (f32, f32)) -> Hitbox {
        Hitbox { width, height, offset, debug_color: None }
    }
    pub fn depth(&self, other: &Hitbox, (mx, my): (f32, f32), (ox, oy): (f32, f32)) -> Option<HitboxCollision> {
        let dx = (ox + other.offset.0) - (mx + self.offset.0);
        let dy = (oy + other.offset.1) - (my + self.offset.1);
        let sw = (self.width + other.width) / 2.0;
        let sh = (self.height + other.height) / 2.0;
        if dx.abs() < sw && dy.abs() < sh {
            Some((dx, dy, sw - dx.abs(), sh - dy.abs()))
        } else {
            None
        }
    }
}

#[derive(Component, Debug)]
#[storage(HashMapStorage)]
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
    pub fn new_wall(width: f32, height: f32) -> Physical {
        Physical { hitbox: Hitbox::new_at_rect(width, height, (0.0, 0.0)), is_static: true }
    }
    pub fn depth(&self, other: &Physical, my_pos: (f32, f32), other_pos: (f32, f32)) -> Option<HitboxCollision> {
        self.hitbox.depth(&other.hitbox, my_pos, other_pos)
    }
}

pub const HITSTATE_SIZE: usize = 16;

pub const ENEMY_ATTACK_BOX: usize = 0;
pub const ENEMY_HITTABLE_BOX: usize = 1;
pub const PLAYER_ATTACK_BOX: usize = 2;
pub const PLAYER_HITTABLE_BOX: usize = 3;
pub const ENEMY_SIGHT_BOX: usize = 4;
pub const ENEMY_AIMING_BOX: usize = 5;
pub const PORTAL_BOX: usize = 6;
pub const PLAYER_INTERACT_BOX: usize = 7;
pub const PLAYER_INTERACTABLE_BOX: usize = 8;

#[derive(Component, Debug, Clone)]
#[storage(HashMapStorage)]
pub struct HitState {
    hitboxes: [Option<Hitbox>; HITSTATE_SIZE],
    rotation: Rotation,
}

impl HitState {
    pub fn new() -> HitState {
        HitState {
            hitboxes: Default::default(),
            rotation: Rotation::East,
        }
    }
    pub fn set(&mut self, index: usize, width: f32, height: f32, offset: (f32, f32)) {
        self.hitboxes[index] = Some(Hitbox::new_at_rect(width, height, offset));
    }
    pub fn clear(&mut self, index: usize) {
        self.hitboxes[index] = None;
    }
    pub fn rotate(&mut self, rotation: Rotation) {
        self.rotation = rotation;
    }
    pub fn get_all(&self) -> [Option<Hitbox>; HITSTATE_SIZE] {
        let mut rotated = self.hitboxes.clone();
        for i in 0..rotated.len() {
            if let Some(mut hitbox) = rotated[i].as_mut() {
                hitbox.offset = self.rotation.rotate(hitbox.offset);
                let (width, height) = match self.rotation {
                    Rotation::North | Rotation::South => {
                        (hitbox.height, hitbox.width)
                    },
                    Rotation::East | Rotation::West => {
                        (hitbox.width, hitbox.height)
                    }
                };
                hitbox.width = width;
                hitbox.height = height;
            }
        }
        rotated
    }
    pub fn get(&self, index: usize) -> Option<Hitbox> {
        if let Some(hitbox) = self.hitboxes[index] {
            let mut rotated = hitbox.clone();
            rotated.offset = self.rotation.rotate(hitbox.offset);
            let (width, height) = match self.rotation {
                Rotation::North | Rotation::South => {
                    (rotated.height, rotated.width)
                },
                Rotation::East | Rotation::West => {
                    (rotated.width, rotated.height)
                }
            };
            rotated.width = width;
            rotated.height = height;
            Some(rotated)
        } else {
            None
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationState {
    Idle,
    Walking,
    Attacking,
    Staggered,
}
#[derive(Component, Debug, Clone, Copy)]
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
    pub velocity: Option<(f32, f32)>,
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
    pub fn add_frame(&mut self, duration: f32) -> usize {
        self.frames.push(AnimationFrame {
            hitboxes: Default::default(),
            velocity: None,
            sprite: None,
            duration,
        });
        self.frames.len() - 1
    }
    pub fn add_frame_with_velocity(&mut self, velocity: (f32, f32), duration: f32) -> usize {
        self.frames.push(AnimationFrame {
            hitboxes: Default::default(),
            velocity: Some(velocity),
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
pub trait HitboxCollisionSystem<'s>: System<'s> {
    type ExtraData: SystemData<'s>;
    fn source() -> usize;
    fn target() -> usize;
    fn collide(&self, collision: HitboxCollision, entity_a: Entity, entity_b: Entity, transforms: &WriteStorage<'s, Transform>, extra: &mut Self::ExtraData);
    fn check_collisions(&mut self,
        (hitboxes, transforms, entities, mut extra) : (ReadStorage<'s, HitState>, WriteStorage<'s, Transform>, Entities<'s>, Self::ExtraData)
    ) {
        for (hitbox_a, transform_a, entity_a) in (&hitboxes, &transforms, &entities).join() {
            if hitbox_a.get(Self::source()).is_none() {
                continue;
            }
            for (hitbox_b, transform_b, entity_b) in (&hitboxes, &transforms, &entities).join() {
                if entity_a.id() == entity_b.id() {
                } else if hitbox_b.get(Self::target()).is_none() {
                } else if let (Some(attack), Some(hit)) =
                    (hitbox_a.get(Self::source()), hitbox_b.get(Self::target())) {
                    let mx = transform_a.translation().x;
                    let my = transform_a.translation().y;
                    let ox = transform_b.translation().x;
                    let oy = transform_b.translation().y;
                    if let Some(collision) = attack.depth(&hit, (mx, my), (ox, oy)) {
                        self.collide(collision, entity_a, entity_b, &transforms, &mut extra);
                    }
                }
            }
        }
    }
}
#[derive(Component, Debug)]
#[storage(HashMapStorage)]
pub struct AnimationController {
    progress: f32,
    animation: Option<HitboxAnimation>,
    state: AnimationState,
    looping: bool,
}
impl AnimationController {
    pub fn new() -> AnimationController {
        AnimationController {
            progress: 0.0,
            animation: None,
            state: AnimationState::Idle,
            looping: false,
        }
    }
    pub fn start(&mut self, animation: HitboxAnimation, state: AnimationState) {
        self.animation = Some(animation);
        self.progress = 0.0;
        self.state = state;
    }
    pub fn start_loop(&mut self, animation: HitboxAnimation) {
        self.animation = Some(animation);
        self.progress = 0.0;
        self.looping = true;
    }
    pub fn step(&mut self, delta_seconds: f32) -> Option<AnimationFrame> {
        self.progress += delta_seconds;
        if let Some(animation) = &self.animation {
            let mut frame = animation.get_frame_at(self.progress);
            if self.looping && frame.is_none() {
                self.progress = 0.0;
                frame = animation.get_frame_at(self.progress);
            }
            if let Some(frame) = frame {
                Some(frame)
            } else {
                self.animation = None;
                self.state = AnimationState::Idle;
                None
            }
        } else {
            self.animation = None;
            self.state = AnimationState::Idle;
            None
        }
    }
    pub fn active(&self) -> bool {
        !self.animation.is_none()
    }
    pub fn state(&self) -> AnimationState {
        self.state
    }
}
