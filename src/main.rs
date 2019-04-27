extern crate amethyst;
#[macro_use]
extern crate specs_derive;
extern crate rand;
mod utils;
mod basics;
mod physics;
mod player;
mod combat;

use std::path::Path;
use amethyst::{
    prelude::*,
    ecs::*,
    input::*,
    core::transform::*,
    core::*,
    renderer::*,
    utils::application_root_dir,
};
use nalgebra::{ Vector3, Point3};
use crate::utils::*;
use crate::basics::*;
use crate::physics::*;
use crate::player::*;
use crate::combat::*;

const stage: (f32, f32) = (200.0, 150.0);

struct EmptySystem;
impl<'s> System<'s> for EmptySystem {
    type SystemData = (
    );
    fn run(&mut self, () : Self::SystemData) {
    }
}
struct CameraFollow;
impl<'s> System<'s> for CameraFollow {
    type SystemData = (
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut transform, camera, player, time) : Self::SystemData) {
        let mut target = None;
        let max_slack = 100.0;
        let track_speed = 100.0;
        for (transform, player) in (&transform, &player).join() {
            let translation = transform.translation();
            target = Some((translation.x - stage.0 / 2.0, translation.y - stage.1 / 2.0));
        }
        if let Some((x, y)) = target {
            for (mut transform, camera) in (&mut transform, &camera).join() {
                let translation = transform.translation();
                let dx = x - translation.x;
                let dy = y - translation.y;
                if dx.abs() > max_slack {
                    transform.translate_x(dx - (dx.signum() * max_slack));
                } else {
                    let movement = track_speed * time.delta_seconds() * dx.signum();
                    if movement.abs() < dx.abs() {
                        transform.translate_x(movement);
                    } else {
                        transform.translate_x(dx);
                    }
                }
                if dy.abs() > max_slack {
                    transform.translate_y(dy - (dy.signum() * max_slack));
                } else {
                    let movement = track_speed * time.delta_seconds() * dy.signum();
                    if movement.abs() < dy.abs() {
                        transform.translate_y(movement);
                    } else {
                        transform.translate_y(dy);
                    }
                }
            }
        }
    }
}
struct DebugDrawHitboxes;
impl DebugDrawHitboxes {
    fn draw(&self, hitbox: &Hitbox, lines: &mut DebugLines, offset: &Vector3<f32>) {
        let x = hitbox.offset.0 - hitbox.size;
        let y = hitbox.offset.1 - hitbox.size;
        let width = hitbox.size * 2.0;
        let height = hitbox.size * 2.0;
        let color = hitbox.debug_color.unwrap_or([1., 1., 1., 1.].into());
        let bottom_left: Point3<f32> = [x, y, 0.1].into();
        let bottom_left = bottom_left + offset;
        let bottom_right: Point3<f32> = [x + width, y, 0.1].into();
        let bottom_right = bottom_right + offset;
        let top_left: Point3<f32> = [x, y + height, 0.1].into();
        let top_left = top_left + offset;
        let top_right: Point3<f32> = [x + width, y + height, 0.1].into();
        let top_right = top_right + offset;
        lines.draw_line(bottom_left, bottom_right, color);
        lines.draw_line(bottom_left, top_left, color);
        lines.draw_line(top_right, bottom_right, color);
        lines.draw_line(top_right, top_left, color);
    }
}
impl<'s> System<'s> for DebugDrawHitboxes {
    type SystemData = (
        Write<'s, DebugLines>,
        ReadStorage<'s, HitState>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, Physical>,
    );
    fn run(&mut self, (mut lines, hitboxes, transform, physical) : Self::SystemData) {
        for (hitboxes, transform) in (&hitboxes, &transform).join() {
            for m_hitbox in hitboxes.get_all().iter() {
                if let Some(hitbox) = m_hitbox {
                    self.draw(&hitbox, &mut lines, &transform.translation())
                }
            }
            self.draw(&hitboxes.bounding, &mut lines, &transform.translation())
        }
        for (physical, transform) in (&physical, &transform).join() {
            self.draw(&physical.hitbox, &mut lines, &transform.translation())
        }
    }
}
struct AnimationSystem;
impl<'s> System<'s> for AnimationSystem {
    type SystemData = (
        WriteStorage<'s, AnimationController>,
        WriteStorage<'s, HitState>,
        WriteStorage<'s, Velocity>,
        WriteStorage<'s, SpriteRender>,
        ReadStorage<'s, Rotation>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut animation, mut hitstate, mut velocity, mut sprite, rotation, time) : Self::SystemData) {
        for (animation, hitstate, mut velocity, mut sprite, rotation) in (&mut animation, &mut hitstate, &mut velocity, &mut sprite, &rotation).join() {
            if animation.active() {
                let frame = animation.step(time.delta_seconds());
                if let Some(frame) = frame {
                    for i in 0..HITSTATE_SIZE {
                        if let Some(hitbox) = frame.hitboxes[i] {
                            hitstate.set(i, hitbox.size, rotation.rotate(hitbox.offset));
                        }
                    }
                    if let Some(frame_velocity) = frame.velocity {
                        let (vx, vy) = rotation.rotate(frame_velocity);
                        velocity.vx = vx;
                        velocity.vy = vy;
                    }
                        if let Some(sprite_id) = frame.sprite {
                        sprite.sprite_number = sprite_id;
                    }
                }
            }
        }
    }
}
struct RotationSystem;
impl<'s> System<'s> for RotationSystem {
    type SystemData = (
        ReadStorage<'s, Rotation>,
        WriteStorage<'s, Transform>,
    );
    fn run(&mut self, (rotation, mut transform) : Self::SystemData) {
        for (rotation, transform) in (&rotation, &mut transform).join() {
            rotation.rotate_euler(transform);
        }
    }
}

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.add_resource(DebugLines::new().with_capacity(100));
        data.world.add_resource(DebugLinesParams {
            line_width: 100.0,
        });
        data.world.register::<StaggerAnimation>();

        spawn_at_z(data.world, 0.0, 0.0, 1.0)
            .with(Camera::from(Projection::orthographic(0.0, stage.0, 0.0, stage.1)))
            .build();

        let sprite_sheet = load_spritesheet(data.world, get_resource("Sprites"));

        let mut hitboxes = HitState::new();
        hitboxes.set(ENEMY_HITTABLE_BOX, 8.0, (0.0, 0.0));

        spawn_at(data.world, stage.0 / 2.0, stage.1 / 2.0)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 1
            })
            .with(Player::new())
            .with(hitboxes)
            .with(AnimationController::new())
            .with_physics(4.0)
            .build();

        let mut hitboxes = HitState::new();
        hitboxes.set(PLAYER_HITTABLE_BOX, 8.0, (0.0, 0.0));

        spawn_at(data.world, 0.0, 0.0)
            // .with(SpriteRender {
            //     sprite_sheet: sprite_sheet.clone(),
            //     sprite_number: 0
            // })
            .with(hitboxes.clone())
            .with(AnimationController::new())
            .with(Health { max: 2, left: 2 })
            .with_physics(8.0)
            .build();

        spawn_at(data.world, stage.0, 0.0)
            // .with(SpriteRender {
            //     sprite_sheet: sprite_sheet.clone(),
            //     sprite_number: 0
            // })
            .with(hitboxes.clone())
            .with(AnimationController::new())
            .with(Health { max: 2, left: 2 })
            .with_physics(8.0)
            .build();

        spawn_at(data.world, 0.0, stage.1)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0
            })
            .with(hitboxes.clone())
            .with(AnimationController::new())
            .with(Health { max: 2, left: 2 })
            .with_physics(8.0)
            .build();

        spawn_at(data.world, stage.0, stage.1)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0
            })
            .with(hitboxes.clone())
            .with(AnimationController::new())
            .with(Health { max: 2, left: 2 })
            .with_physics(8.0)
            .build();
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let display_path = get_resource("display_config.ron");
    let config = DisplayConfig::load(&display_path);

    let input_path = get_resource("input.ron");
    let input_bundle: InputBundle<String, String> = InputBundle::new().with_bindings_from_file(input_path)?;

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None))
            .with_pass(DrawDebugLines::<PosColorNorm>::new()),
    );

    let game_data =
        GameDataBuilder::default()
            .with_bundle(RenderBundle::new(pipe, Some(config))
                .with_sprite_sheet_processor())?
            .with_bundle(input_bundle)?
            .with_bundle(TransformBundle::new())?
            .with(PlayerMovementSystem::new(), "player_move", &[])
            .with(PlayerAttackSystem::new(), "player_attack", &[])
            .with(VelocitySystem, "velocity", &[])
            .with(CameraFollow, "camera_follow", &[])
            .with(RestitutionSystem, "restitution", &["velocity"])
            .with(RotationSystem, "rotation", &[])
            .with(AnimationSystem, "animation", &[])
            .with(DebugDrawHitboxes, "debug_hitboxes", &[])
            .with(DamageSystem, "damage", &["animation"])
            .with(DeathSystem, "death", &["damage"]);
    let mut game = Application::new("./resources", Example, game_data)?;

    game.run();

    Ok(())
}
