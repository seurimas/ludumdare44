use amethyst::{
    prelude::*,
    ecs::*,
    renderer::Rgba,
    core::*,
    input::*,
};
use crate::basics::*;
use crate::player::*;
use crate::enemies::*;


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Health {
    pub max: i32,
    pub left: i32,
    pub invuln: f32,
}
impl Health {
    pub fn new(max: i32) -> Health {
        Health {
            max,
            left: max,
            invuln: 0.0,
        }
    }
    pub fn hit_for(&mut self, amount: i32, invuln: f32) {
        self.left -= amount;
        self.invuln = invuln;
    }
    pub fn pay(&mut self, amount: i32) {
        self.left -= amount;
        self.max -= amount;
    }
    pub fn embiggen(&mut self) {
        self.left *= 2;
        self.max *= 2;
    }
}

pub struct DeathSystem;
impl<'s> System<'s> for DeathSystem {
    type SystemData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, Health>,
        Entities<'s>,
        Read<'s, Time>,
    );
    fn run(&mut self, (animation, mut health, mut entities, time) : Self::SystemData) {
        for (health, entity) in (&mut health, &entities).join() {
            if health.left <= 0 {
                if !is_staggered(entity, &animation) {
                    entities.delete(entity);
                }
            }
            health.invuln -= time.delta_seconds();
        }
    }
}

#[derive(Component, Debug)]
#[storage(HashMapStorage)]
pub struct StaggerAnimation {
    stagger: HitboxAnimation,
}
impl StaggerAnimation {
    pub fn new(stagger: HitboxAnimation) -> StaggerAnimation {
        StaggerAnimation { stagger }
    }
}

pub struct PlayerDamageSystem;
fn knockback(distance: f32) -> HitboxAnimation {
    let speed = 100.0;
    let mut animation = HitboxAnimation::new();
    animation.add_frame_with_velocity((-speed, 0.0), distance / speed);
    animation.add_frame_with_velocity((0.0, 0.0), 0.2);
    animation
}
fn stagger_entity<'s>(entity: Entity, animations: &mut WriteStorage<'s, AnimationController>, stagger_animation: HitboxAnimation) {
    if let Some(animation) = animations.get_mut(entity) {
        animation.start(stagger_animation, AnimationState::Staggered);
    }
}
pub fn is_staggered<'s>(entity: Entity, animations: &WriteStorage<'s, AnimationController>) -> bool {
    if let Some(animation) = animations.get(entity) {
        animation.state() == AnimationState::Staggered
    } else {
        false
    }
}
fn knockback_entity<'s>(collision: HitboxCollision, entity: Entity, animations: &mut WriteStorage<'s, AnimationController>, rotation: &mut WriteStorage<'s, Rotation>) {
    stagger_entity(entity, animations, knockback(15.0));
    if let Some(current_rotation) = rotation.get(entity) {
        let (dx, dy, _depthx, _depthy) = collision;
        rotation.insert(entity, {
            if dx.abs() > dy.abs() {
                if dx < 0.0 {
                    Rotation::East
                } else {
                    Rotation::West
                }
            } else {
                if dy < 0.0 {
                    Rotation::North
                } else {
                    Rotation::South
                }
            }
        });
    }
}
impl<'s> System<'s> for PlayerDamageSystem {
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
impl<'s> HitboxCollisionSystem<'s> for PlayerDamageSystem {
    type ExtraData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, Rotation>,
        WriteStorage<'s, Health>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Velocity>,
        ReadStorage<'s, StaggerAnimation>,
    );
    fn collide(&self, collision: HitboxCollision, entity_a: Entity, entity_b: Entity, transforms: &WriteStorage<'s, Transform>, extra: &mut Self::ExtraData) {
        let animations = &mut extra.0;
        let rotation = &mut extra.1;
        let velocity = &mut extra.4;
        let stagger_animation = &mut extra.5;
        if !is_staggered(entity_b, animations) {
            if let Some(stagger_animation) = stagger_animation.get(entity_b) {
                stagger_entity(entity_b, animations, stagger_animation.stagger.clone());
            } else if let Some(velocity) = velocity.get(entity_b) {
                knockback_entity(collision, entity_b, animations, rotation);
            }
            let health = &mut extra.2;
            let player = &extra.3;
            if let (Some(player), Some(health)) = (player.get(entity_a), health.get_mut(entity_b)) {
                health.left -= 1;
            }
        }
    }
    fn source() -> usize {
        PLAYER_ATTACK_BOX
    }
    fn target() -> usize {
        PLAYER_HITTABLE_BOX
    }
}
pub struct EnemyDamageSystem;
impl<'s> System<'s> for EnemyDamageSystem {
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
impl<'s> HitboxCollisionSystem<'s> for EnemyDamageSystem {
    type ExtraData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, Rotation>,
        WriteStorage<'s, Health>,
        ReadStorage<'s, MeleeEnemy>,
        ReadStorage<'s, Velocity>,
        ReadStorage<'s, StaggerAnimation>,
    );
    fn collide(&self, collision: HitboxCollision, entity_a: Entity, entity_b: Entity, transforms: &WriteStorage<'s, Transform>, extra: &mut Self::ExtraData) {
        let animations = &mut extra.0;
        let rotation = &mut extra.1;
        let velocity = &mut extra.4;
        let stagger_animation = &mut extra.5;
        if !is_staggered(entity_b, animations) {
            if let Some(stagger_animation) = stagger_animation.get(entity_b) {
                stagger_entity(entity_b, animations, stagger_animation.stagger.clone());
            } else if let Some(velocity) = velocity.get(entity_b) {
                knockback_entity(collision, entity_b, animations, rotation);
            }
            let health = &mut extra.2;
            let enemy = &extra.3;
            if let (Some(enemy), Some(health)) = (enemy.get(entity_a), health.get_mut(entity_b)) {
                health.hit_for(enemy.damage, 1.0);
            }
        }
    }
    fn source() -> usize {
        ENEMY_ATTACK_BOX
    }
    fn target() -> usize {
        ENEMY_HITTABLE_BOX
    }
}
