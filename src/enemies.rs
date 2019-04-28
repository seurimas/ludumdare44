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
pub struct MeleeEnemy {
    in_melee: bool,
    attack_animation: HitboxAnimation,
}
impl MeleeEnemy {
    pub fn new(attack_animation: HitboxAnimation) -> MeleeEnemy {
        MeleeEnemy {
            in_melee: false,
            attack_animation,
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ChaseAndWanderEnemy {
    aware_of_player: Option<(f32, f32)>,
    idle_animation: HitboxAnimation,
    walking_animation: HitboxAnimation,
    wander_speed: f32,
    chase_speed: f32,
    wander_direction: Option<(f32, f32)>,
    wander_progress: f32,
}
impl ChaseAndWanderEnemy {
    pub fn new(idle_animation: HitboxAnimation, walking_animation: HitboxAnimation, wander_speed: f32, chase_speed: f32) -> ChaseAndWanderEnemy {
        ChaseAndWanderEnemy {
            aware_of_player: None,
            idle_animation,
            walking_animation,
            wander_speed,
            chase_speed,
            wander_direction: None,
            wander_progress: -1.0,
        }
    }
}
const wanders: [(f32, f32); 4] = [
    ( 1.0,  0.0),
    (-1.0,  0.0),
    ( 0.0,  1.0),
    ( 0.0, -1.0),
];
pub struct ChaseAndWanderSystem;
impl ChaseAndWanderSystem {
    fn walk<'s>(
        rotations: &mut WriteStorage<'s, Rotation>,
        animation_controller: &mut AnimationController,
        animation: HitboxAnimation,
        velocity: &mut Velocity,
        entity: Entity,
        (wx, wy): (f32, f32)) {
        animation_controller.start(animation, AnimationState::Walking);
        velocity.vx = wx;
        velocity.vy = wy;
        let rotation = {
            if wx.abs() > wy.abs() {
                if wx > 0.0 {
                    Rotation::East
                } else {
                    Rotation::West
                }
            } else {
                if wy > 0.0 {
                    Rotation::North
                } else {
                    Rotation::South
                }
            }
        };
        if let Ok(_) = rotations.insert(entity, rotation) {
        }
    }
    fn wander<'s>(
        rotations: &mut WriteStorage<'s, Rotation>,
        animation_controller: &mut AnimationController,
        velocity: &mut Velocity,
        enemy: &ChaseAndWanderEnemy,
        entity: Entity,
        (wx, wy): (f32, f32)
    ) {
        ChaseAndWanderSystem::walk(
            rotations,
            animation_controller,
            enemy.walking_animation.clone(),
            velocity,
            entity,
            (wx * enemy.wander_speed, wy * enemy.wander_speed),
        );
    }
}
impl<'s> System<'s> for ChaseAndWanderSystem {
    type SystemData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, ChaseAndWanderEnemy>,
        WriteStorage<'s, MeleeEnemy>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, Rotation>,
        WriteStorage<'s, Velocity>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
        Entities<'s>,
    );
    fn run(&mut self, (mut animation, mut enemy, mut melee, transform, mut rotations, mut velocity, player, time, entities) : Self::SystemData) {
        for (mut animation, mut enemy, transform, mut velocity, entity) in (&mut animation, &mut enemy, &transform, &mut velocity, &entities).join() {
            if animation.state() == AnimationState::Idle || animation.state() == AnimationState::Walking {
                if let Some(player_position) = enemy.aware_of_player {
                    let mut chase = true;
                    if let Some(melee) = melee.get(entity) {
                        if melee.in_melee {
                            println!("Attacking");
                            animation.start(melee.attack_animation.clone(), AnimationState::Attacking);
                            chase = false;
                        }
                    }
                    if chase {
                        let dx = player_position.0 - transform.translation().x;
                        let dy = player_position.1 - transform.translation().y;
                        if length(dx, dy) > 12.0 {
                            let dir = normalize(dx, dy);
                            let (wx, wy) = (dir.0 * enemy.chase_speed, dir.1 * enemy.chase_speed);
                            ChaseAndWanderSystem::walk(&mut rotations, animation, enemy.walking_animation.clone(), velocity, entity, (wx, wy));
                        } else {
                            animation.start(enemy.idle_animation.clone(), AnimationState::Idle);
                            velocity.vx = 0.0;
                            velocity.vy = 0.0;
                        }
                    }
                } else if let Some((wx, wy)) = enemy.wander_direction {
                    enemy.wander_progress -= time.delta_seconds();
                    if enemy.wander_progress <= 0.0 {
                        println!("Stopping");
                        animation.start(enemy.idle_animation.clone(), AnimationState::Idle);
                        velocity.vx = 0.0;
                        velocity.vy = 0.0;
                        enemy.wander_progress = random_between(2.0, 5.0);
                        enemy.wander_direction = None;
                    } else if animation.state() != AnimationState::Walking {
                        ChaseAndWanderSystem::wander(&mut rotations, animation, velocity, enemy, entity, (wx, wy));
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
pub struct SightSystem;
impl<'s> System<'s> for SightSystem {
    type SystemData = (
        ReadStorage<'s, HitState>,
        WriteStorage<'s, Transform>,
        Entities<'s>,
        <Self as HitboxCollisionSystem<'s>>::ExtraData,
    );
    fn run(&mut self, system_data: Self::SystemData) {
        self.check_collisions(system_data);
    }
}
impl<'s> HitboxCollisionSystem<'s> for SightSystem {
    type ExtraData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, ChaseAndWanderEnemy>,
    );
    fn collide(&self, collision: HitboxCollision, entity_a: Entity, entity_b: Entity, transforms: &WriteStorage<'s, Transform>, extra: &mut Self::ExtraData) {
        let animations = &mut extra.0;
        let enemy = &mut extra.1;
        if let (Some(animation), Some(enemy), Some(player_loc)) =
            (animations.get_mut(entity_a), enemy.get_mut(entity_a), transforms.get(entity_b)) {
            enemy.aware_of_player = Some((player_loc.translation().x, player_loc.translation().y))
        }
    }
    fn source() -> usize {
        ENEMY_SIGHT_BOX
    }
    fn target() -> usize {
        ENEMY_HITTABLE_BOX
    }
}
pub struct AimingSystem;
impl<'s> System<'s> for AimingSystem {
    type SystemData = (
        ReadStorage<'s, HitState>,
        WriteStorage<'s, Transform>,
        Entities<'s>,
        <Self as HitboxCollisionSystem<'s>>::ExtraData,
    );
    fn run(&mut self, mut system_data: Self::SystemData) {
        let extra = &mut system_data.3;
        for (mut enemy) in (&mut (extra.1)).join() {
            enemy.in_melee = false;
        }
        self.check_collisions(system_data);
    }
}
impl<'s> HitboxCollisionSystem<'s> for AimingSystem {
    type ExtraData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, MeleeEnemy>,
    );
    fn collide(&self, collision: HitboxCollision, entity_a: Entity, entity_b: Entity, transforms: &WriteStorage<'s, Transform>, extra: &mut Self::ExtraData) {
        let animations = &mut extra.0;
        let enemy = &mut extra.1;
        if let (Some(animation), Some(enemy), Some(player_loc)) =
            (animations.get_mut(entity_a), enemy.get_mut(entity_a), transforms.get(entity_b)) {
            enemy.in_melee = true;
        }
    }
    fn source() -> usize {
        ENEMY_AIMING_BOX
    }
    fn target() -> usize {
        ENEMY_HITTABLE_BOX
    }
}

pub fn spawn_goblin(world: &mut World, sprite_sheet: SpriteSheetHandle, x: f32, y: f32) -> EntityBuilder {
    let idle = idle_animation(7);
    let walking = walking_animation(7, 8, 9, 0.1);
    let mut attack_animation = HitboxAnimation::new();
    let frame = attack_animation.add_frame_with_velocity((0.0, 0.0), 0.125);
    attack_animation.set_sprite(frame, 10);
    let frame = attack_animation.add_frame_with_velocity((50.0, 0.0), 0.125);
    attack_animation.set_sprite(frame, 11);
    attack_animation.set_hitbox(frame, ENEMY_ATTACK_BOX, Hitbox::new_at(4.0, (8.0, 2.0)));
    let frame = attack_animation.add_frame_with_velocity((50.0, 0.0), 0.375);
    attack_animation.set_sprite(frame, 13);
    let frame = attack_animation.add_frame_with_velocity((50.0, 0.0), 0.125);
    attack_animation.set_sprite(frame, 12);
    attack_animation.set_hitbox(frame, ENEMY_ATTACK_BOX, Hitbox::new_at(4.0, (8.0, -2.0)));
    let frame = attack_animation.add_frame_with_velocity((50.0, 0.0), 0.375);
    attack_animation.set_sprite(frame, 14);
    let frame = attack_animation.add_frame_with_velocity((0.0, 0.0), 0.5);
    attack_animation.set_sprite(frame, 7);
    let mut hitstate = HitState::new();
    hitstate.set(ENEMY_AIMING_BOX, 6.0, (12.0, 0.0));
    hitstate.set(ENEMY_SIGHT_BOX, 32.0, (20.0, 0.0));
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
        .with(MeleeEnemy::new(attack_animation))
        .with(ChaseAndWanderEnemy::new(idle, walking, 50.0, 100.0))
}
