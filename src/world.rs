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
