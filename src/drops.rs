use amethyst::{
    prelude::*,
    ecs::*,
    renderer::{SpriteRender, SpriteSheetHandle, Rgba},
    core::*,
    input::*,
};
use crate::basics::*;
use crate::combat::*;
use crate::world::*;
use crate::sprites::*;
use crate::utils::*;

pub fn spawn_chest(world: &mut World, x: f32, y: f32, cost: u32) {
    let mut hitboxes = HitState::new();
    hitboxes.set(PLAYER_HITTABLE_BOX, 16.0, 16.0, (0.0, 0.0));
    let sprite_sheet = get_sprite_sheet(world);
    let chest = spawn_at(world, x, y)
        .with_sprite(sprite_sheet, CHEST_SPRITE)
        .with(hitboxes.clone())
        .with(AnimationController::new())
        .with(Health::new(2))
        .with_physics(8.0)
        .build();
    if cost == 2 {
        heart_spin(world, -4.0, 8.0)
            .with(Parent { entity: chest })
            .build();
        heart_spin(world, 4.0, 8.0)
            .with(Parent { entity: chest })
            .build();
    } else if cost == 1 {
        heart_spin(world, 0.0, 8.0)
            .with(Parent { entity: chest })
            .build();
    }
}
