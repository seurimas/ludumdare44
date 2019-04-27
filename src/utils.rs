use amethyst::{
    prelude::*,
    ecs::*,
    utils::application_root_dir,
    core::transform::*,
    assets::*,
    renderer::*,
};
use crate::basics::*;

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

fn at(x: f32, y: f32) -> Transform {
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
}

impl <'s> BuilderHelp for EntityBuilder<'s> {
    fn with_physics(self, size: f32) -> EntityBuilder<'s> {
        self
            .with(Velocity { vx: 0.0, vy: 0.0 })
            .with(Physical::new(size))
            .with(Rotation::East)
    }
}

pub fn load_texture(world: &mut World, path: String) -> TextureHandle {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    loader.load(
        path,
        PngFormat,
        TextureMetadata::srgb_scale(),
        (),
        &texture_storage,
    )
}
pub fn load_spritesheet(world: &mut World, path: String) -> SpriteSheetHandle {
    let texture_handle = load_texture(world, format!("{}.png", path));
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("{}.ron", path), // Here we load the associated ron file
        SpriteSheetFormat,
        texture_handle, // We pass it the handle of the texture we want it to use
        (),
        &sprite_sheet_store,
    )
}
