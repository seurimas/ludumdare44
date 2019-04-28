use amethyst::{
    prelude::*,
    ecs::*,
    renderer::*,
    core::*,
    input::*,
    core::transform::*,
};
use crate::sprites::*;
use crate::basics::*;
use crate::combat::*;
use crate::utils::*;

const WORLD_BASE: (i32, i32) = (-256, -256);
const TILE_SIZE: (i32, i32) = (16, 16);

pub fn spawn_world_tile(world: &mut World, sprite_sheet: SpriteSheetHandle, x: i32, y: i32, sprite_number: usize) -> EntityBuilder {
    let (tx, ty) = (WORLD_BASE.0 + x * TILE_SIZE.0, WORLD_BASE.1 + y * TILE_SIZE.1);
    spawn_at_z(world, tx as f32, ty as f32, -1.0)
        .with_sprite(sprite_sheet.clone(), sprite_number)
}

pub fn fill_world(world: &mut World, sprite_sheet: &SpriteSheetHandle, width: i32, height: i32, sprite_number: usize) {
    for x in 0..width {
        for y in 0..height {
            spawn_world_tile(world, sprite_sheet.clone(), x, y, sprite_number).build();
        }
    }
}

pub fn draw_wall(world: &mut World, sprite_sheet: &SpriteSheetHandle, (x, y): (i32, i32), (width, height): (i32, i32), sprite_number: usize) {
    for x in x..(x + width) {
        for y in y..(y + height) {
            spawn_world_tile(world, sprite_sheet.clone(), x, y, sprite_number)
                .build();
        }
    }
    let (tx, ty) = (WORLD_BASE.0 + x * TILE_SIZE.0, WORLD_BASE.1 + y * TILE_SIZE.1);
    let (w, h) = (TILE_SIZE.0 * width, TILE_SIZE.1 * height);
    spawn_at_z(world, (tx + w / 2 - TILE_SIZE.0 / 2) as f32, (ty + h / 2 - TILE_SIZE.1 / 2) as f32, -1.0)
        .with(Physical::new_wall(w as f32, h as f32))
        .build();
}

fn heart_spin_animation(rotation_speed: f32, pause_duration: f32, heart_spin: [usize; 8]) -> HitboxAnimation {
    let mut animation = HitboxAnimation::new();
    let frame = animation.add_frame(pause_duration);
    animation.set_sprite(frame, heart_spin[0]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[1]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[2]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[3]);
    let frame = animation.add_frame(pause_duration);
    animation.set_sprite(frame, heart_spin[4]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[5]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[6]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[7]);
    animation
}

pub fn heart_spin(world: &mut World, sprite_sheet: SpriteSheetHandle, x: f32, y: f32) -> EntityBuilder {
    let mut animation_controller = AnimationController::new();
    let rotation_speed = 0.15;
    let pause_duration = 0.5;
    animation_controller.start_loop(heart_spin_animation(rotation_speed, pause_duration, HEART_SPIN));
    spawn_at(world, x, y)
        .with_sprite(sprite_sheet, HEART_SPIN[0])
        .with(animation_controller)
}

pub fn spend_heart_spin(world: &mut World, sprite_sheet: SpriteSheetHandle, x: f32, y: f32) -> EntityBuilder {
    let mut animation_controller = AnimationController::new();
    let rotation_speed = 0.25;
    let pause_duration = 0.25;
    animation_controller.start_loop(heart_spin_animation(rotation_speed, pause_duration, SPEND_HEART_SPIN));
    spawn_at(world, x, y)
        .with_sprite(sprite_sheet, SPEND_HEART_SPIN[0])
        .with(animation_controller)
}
