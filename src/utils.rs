use amethyst::{
    prelude::*,
    ecs::*,
    utils::application_root_dir,
    core::transform::*,
    assets::*,
    renderer::*,
};
use crate::basics::*;
use rand::prelude::*;
use rand::distributions::Standard;
pub use rand::seq::SliceRandom;

pub fn length(x: f32, y: f32) -> f32 {
    f32::sqrt(x * x + y * y)
}

pub fn normalize(x: f32, y: f32) -> (f32, f32) {
    if x == 0.0 && y == 0.0 {
        (0.0, 0.0)
    } else {
        let length = length(x, y);
        (x / length, y / length)
    }
}

pub fn get_resource(str: &str) -> String {
     format!(
        "{}/resources/{}",
        application_root_dir(),
        str
    )
}

pub fn at(x: f32, y: f32) -> Transform {
    let mut transform = Transform::default();
    transform.set_x(x);
    transform.set_y(y);
    transform
}

pub fn spawn_at(world: &mut World, x: f32, y: f32) -> EntityBuilder {
    world.create_entity()
        .with(at(x, y))
        .with(GlobalTransform::default())
}

pub fn spawn_at_z(world: &mut World, x: f32, y: f32, z: f32) -> EntityBuilder {
    let mut transform = at(x, y);
    transform.set_z(z);
    world.create_entity()
        .with(transform)
        .with(GlobalTransform::default())
}

pub trait BuilderHelp {
    fn with_physics(self, size: f32) -> Self;
    fn with_static(self, size: f32) -> Self;
    fn with_sprite(self, sprite_sheet: SpriteSheetHandle, sprite_number: usize) -> Self;
}

impl <'s> BuilderHelp for EntityBuilder<'s> {
    fn with_physics(self, size: f32) -> EntityBuilder<'s> {
        self
            .with(Velocity { vx: 0.0, vy: 0.0 })
            .with(Physical::new(size))
            .with(Rotation::East)
    }
    fn with_static(self, size: f32) -> EntityBuilder<'s> {
        self
            .with(Physical::new_static(size))
    }
    fn with_sprite(self, sprite_sheet: SpriteSheetHandle, sprite_number: usize) -> EntityBuilder<'s> {
        self
            .with(SpriteRender {
                sprite_sheet: sprite_sheet,
                sprite_number
            })
            .with(Transparent)
    }
}

pub fn load_texture<'a>(world: &mut World, path: String, progress: &'a mut ProgressCounter) -> TextureHandle {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        path,
        PngFormat,
        TextureMetadata::srgb_scale(),
        progress,
        &texture_storage,
    )
}
pub fn load_spritesheet<'a>(world: &mut World, path: String, progress: &'a mut ProgressCounter) -> SpriteSheetHandle {
    let texture_handle = load_texture(world, format!("{}.png", path), progress);
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("{}.ron", path), // Here we load the associated ron file
        SpriteSheetFormat,
        texture_handle, // We pass it the handle of the texture we want it to use
        progress,
        &sprite_sheet_store,
    )
}
pub fn get_sprite_sheet(world: &World) -> SpriteSheetHandle {
    let main_sprite = world.read_resource::<MainSprite>();
    main_sprite.0.clone()
}
pub fn idle_animation(stand: usize) -> HitboxAnimation {
    let mut idle = HitboxAnimation::new();
    let frame = idle.add_frame(1.0);
    idle.set_sprite(frame, stand);
    idle
}
pub fn walking_animation(stand: usize, left: usize, right: usize, duration: f32) -> HitboxAnimation {
    let mut walking = HitboxAnimation::new();
    let frame = walking.add_frame(duration);
    walking.set_sprite(frame, stand);
    let frame = walking.add_frame(duration);
    walking.set_sprite(frame, left);
    let frame = walking.add_frame(duration);
    walking.set_sprite(frame, stand);
    let frame = walking.add_frame(duration);
    walking.set_sprite(frame, right);
    walking
}
pub fn random_between(low: f32, high: f32) -> f32 {
    low + (high - low) * thread_rng().gen::<f32>()
}
pub fn rng() -> ThreadRng {
    thread_rng()
}
