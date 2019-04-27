use amethyst::{
    prelude::*,
    ecs::*,
    renderer::{SpriteRender, SpriteSheetHandle, Rgba},
    core::*,
    input::*,
};
use crate::basics::*;
use crate::combat::*;
use crate::player::*;
use crate::utils::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ChaseAndWanderEnemy {
    aware_of_player: bool,
    idle_animation: HitboxAnimation,
    walking_animation: HitboxAnimation,
    wander_speed: f32,
    chase_speed: f32,
    wander_direction: Option<(Rotation, f32, f32)>,
    wander_progress: f32,
}
impl ChaseAndWanderEnemy {
    pub fn new(idle_animation: HitboxAnimation, walking_animation: HitboxAnimation, wander_speed: f32, chase_speed: f32) -> ChaseAndWanderEnemy {
        ChaseAndWanderEnemy {
            aware_of_player: false,
            idle_animation,
            walking_animation,
            wander_speed,
            chase_speed,
            wander_direction: None,
            wander_progress: -1.0,
        }
    }
}
const wanders: [(Rotation, f32, f32); 4] = [
    (Rotation::East, 1.0, 0.0),
    (Rotation::West, -1.0, 0.0),
    (Rotation::North, 0.0, 1.0),
    (Rotation::South, 0.0, -1.0),
];
pub struct ChaseAndWanderSystem;
impl<'s> System<'s> for ChaseAndWanderSystem {
    type SystemData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, ChaseAndWanderEnemy>,
        WriteStorage<'s, Rotation>,
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
        Entities<'s>,
    );
    fn run(&mut self, (mut animation, mut enemy, mut rotations, mut velocity, player, time, entities) : Self::SystemData) {
        for (mut animation, mut enemy, mut velocity, entity) in (&mut animation, &mut enemy, &mut velocity, &entities).join() {
            if animation.state() == AnimationState::Idle || animation.state() == AnimationState::Walking {
                if enemy.aware_of_player {

                } else if let Some((rotation, wx, wy)) = enemy.wander_direction {
                    enemy.wander_progress -= time.delta_seconds();
                    if enemy.wander_progress <= 0.0 {
                        println!("Stopping");
                        animation.start(enemy.idle_animation.clone(), AnimationState::Idle);
                        velocity.vx = 0.0;
                        velocity.vy = 0.0;
                        enemy.wander_progress = random_between(2.0, 5.0);
                        enemy.wander_direction = None;
                    } else if animation.state() != AnimationState::Walking {
                        println!("stepping");
                        animation.start(enemy.walking_animation.clone(), AnimationState::Walking);
                        velocity.vx = wx * enemy.wander_speed;
                        velocity.vy = wy * enemy.wander_speed;
                        if let Ok(_) = rotations.insert(entity, rotation) {

                        }
                    }
                } else {
                    enemy.wander_progress -= time.delta_seconds();
                    if enemy.wander_progress <= 0.0 {
                        enemy.wander_progress = random_between(1.0, 3.0);
                        enemy.wander_direction = wanders.choose(&mut rng()).cloned()
                    }
                }
            }
        }
    }
}

pub fn spawn_goblin(world: &mut World, sprite_sheet: SpriteSheetHandle, x: f32, y: f32) -> EntityBuilder {
    let idle = idle_animation(7);
    let walking = walking_animation(7, 8, 9, 0.1);
    let mut hitstate = HitState::new();
    hitstate.set(ENEMY_SIGHT_BOX, 24.0, (12.0, 0.0));
    hitstate.set(PLAYER_HITTABLE_BOX, 6.0, (0.0, 0.0));

    spawn_at(world, x, y)
        .with_physics(6.0)
        .with(AnimationController::new())
        .with(hitstate)
        .with(Health { max: 2, left: 2 })
        .with(SpriteRender {
            sprite_sheet: sprite_sheet,
            sprite_number: 7,
        })
        .with(ChaseAndWanderEnemy::new(idle, walking, 50.0, 100.0))
}
