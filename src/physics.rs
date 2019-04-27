use amethyst::{
    prelude::*,
    ecs::*,
    core::transform::*,
    core::*,
};
use crate::basics::*;

pub struct VelocitySystem;
impl<'s> System<'s> for VelocitySystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Velocity>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut transform, velocity, time) : Self::SystemData) {
        for (mut transform, velocity) in (&mut transform, &velocity).join() {
            transform.translate_x(velocity.vx * time.delta_seconds());
            transform.translate_y(velocity.vy * time.delta_seconds());
        }
    }
}
pub struct RestitutionSystem;
impl<'s> System<'s> for RestitutionSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Physical>,
        Entities<'s>,
    );
    fn run(&mut self, (mut transform, physical, entities) : Self::SystemData) {
        let mut obstacles = Vec::new();
        for (transform, physical, entity) in (&transform, &physical, &entities).join() {
            obstacles.push((transform.translation().clone(), physical, entity));
        }
        for (mut transform, physical, entity) in (&mut transform, &physical, &entities).join() {
            if physical.is_static {
                continue;
            }
            let mut restitution = (0.0, 0.0);
            let x = transform.translation().x;
            let y = transform.translation().y;
            for (translation, obstacle, obs_entity) in obstacles.iter() {
                if obs_entity.id() == entity.id() {
                } else if let Some((dx, dy)) = physical.depth(obstacle, (x, y), (translation.x, translation.y)) {
                    let dir_x = (x - translation.x).signum();
                    let dir_y = (y - translation.y).signum();
                    let factor = {
                        if obstacle.is_static {
                            1.0
                        } else {
                            2.0
                        }
                    };
                    if dy.abs() > dx.abs() {
                        restitution.0 += dx * dir_x / factor;
                    } else {
                        restitution.1 += dy * dir_y / factor;
                    }
                }
            }
            transform.translate_x(restitution.0);
            transform.translate_y(restitution.1);
        }
    }
}
