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
use crate::player::*;

#[derive(Debug, Clone)]
pub enum Upgrade {
    HeartBracelet,
    GoldenAegis,
    CursedRing,
}

#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct Chest {
    cost: i32,
    upgrade: Upgrade,
}

pub fn spawn_chest(world: &mut World, x: f32, y: f32, cost: i32, upgrade: Upgrade) {
    let mut hitboxes = HitState::new();
    hitboxes.set(PLAYER_HITTABLE_BOX, 16.0, 16.0, (0.0, 0.0));
    hitboxes.set(CHEST_BOX, 16.0, 16.0, (0.0, 0.0));
    let sprite_sheet = get_sprite_sheet(world);
    let chest = spawn_at(world, x, y)
        .with_sprite(sprite_sheet, CHEST_SPRITE)
        .with(hitboxes.clone())
        .with(AnimationController::new())
        .with(Health::new(3))
        .with(Chest { cost, upgrade })
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
pub struct PurchaseSystem;
impl<'s> System<'s> for PurchaseSystem {
    type SystemData = (
        ReadStorage<'s, HitState>,
        WriteStorage<'s, Transform>,
        Entities<'s>,
        <Self as HitboxCollisionSystem<'s>>::ExtraData,
    );
    fn run(&mut self, mut system_data: Self::SystemData) {
        let extra = &mut system_data.3;
        let input = &extra.3;
        if let Some(true) = input.action_is_down("interact") {
            self.check_collisions(system_data);
        }
    }
}
impl<'s> HitboxCollisionSystem<'s> for PurchaseSystem {
    type ExtraData = (
        WriteStorage<'s, Health>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, Chest>,
        Read<'s, InputHandler<String, String>>,
    );
    fn collide(&self, collision: HitboxCollision, entity_a: Entity, entity_b: Entity, transforms: &WriteStorage<'s, Transform>, extra: &mut Self::ExtraData) {
        let health = &mut extra.0;
        if let Some(chest_health) = health.get_mut(entity_a) {
            chest_health.hit_for(99, 0.0);
        }
        let player = &mut extra.1;
        let chest = &mut extra.2;
        if let (Some(health), Some(mut player), Some(chest)) =
            (health.get_mut(entity_b), player.get_mut(entity_b), chest.get(entity_a)) {
            let mut cost = chest.cost;
            if player.big_hearts {
                cost *= 2;
            }
            if health.left > cost && health.max > cost {
                match (&chest.upgrade, player.big_hearts, player.healthy) {
                    (Upgrade::GoldenAegis, false, _) => {
                        health.pay(cost);
                        health.embiggen();
                        player.big_hearts = true;
                    },
                    (Upgrade::HeartBracelet, _, false) => {
                        health.pay(cost);
                        player.healthy = true;
                    },
                    _ => {

                    }
                }
            }
        }
    }
    fn source() -> usize {
        CHEST_BOX
    }
    fn target() -> usize {
        PLAYER_INTERACT_BOX
    }
}
