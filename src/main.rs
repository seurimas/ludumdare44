extern crate amethyst;
#[macro_use]
extern crate specs_derive;
extern crate rand;
mod utils;
mod basics;

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
use crate::utils::*;
use crate::basics::*;

const stage: (f32, f32) = (400.0, 300.0);

struct EmptySystem;
impl<'s> System<'s> for EmptySystem {
    type SystemData = (
    );
    fn run(&mut self, () : Self::SystemData) {
    }
}
struct PlayerMovementSystem;
impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Velocity>,
        Read<'s, InputHandler<String, String>>,
        Read<'s, Time>,
    );
    fn run(&mut self, (players, mut velocities, input, time) : Self::SystemData) {
        let deacc_factor = 3.0;
        for (player, mut velocity) in (&players, &mut velocities).join() {
            let x_tilt = input.axis_value("leftright");
            let y_tilt = input.axis_value("updown");
            if let (Some(x_tilt), Some(y_tilt)) = (x_tilt, y_tilt) {
                let mut x_accel = 0.0;
                if x_tilt < 0.0 {
                    x_accel = -player.walk_accel;
                } else if x_tilt > 0.0 {
                    x_accel = player.walk_accel;
                } else if velocity.vx != 0.0 {
                    let direction = velocity.vx / velocity.vx.abs();
                    x_accel = player.walk_accel * -direction;
                }
                let mut y_accel = 0.0;
                if y_tilt < 0.0 {
                    y_accel = -player.walk_accel;
                } else if y_tilt > 0.0 {
                    y_accel = player.walk_accel;
                } else if velocity.vy != 0.0 {
                    let direction = velocity.vy / velocity.vy.abs();
                    y_accel = player.walk_accel * -direction;
                }
                if (x_accel > 0.0 && velocity.vx <= 0.0)
                    || (x_accel < 0.0 && velocity.vx > 0.0){
                    x_accel *= deacc_factor;
                }
                if (y_accel > 0.0 && velocity.vy <= 0.0)
                    || (y_accel < 0.0 && velocity.vy > 0.0){
                    y_accel *= deacc_factor;
                }
                if x_tilt == 0.0 && x_accel.abs() * time.delta_seconds() > velocity.vx.abs() {
                    velocity.vx = 0.0;
                } else {
                    velocity.vx += x_accel * time.delta_seconds();
                }
                if y_tilt == 0.0 && y_accel.abs() * time.delta_seconds() > velocity.vy.abs() {
                    velocity.vy = 0.0;
                } else {
                    velocity.vy += y_accel * time.delta_seconds();
                }
                if velocity.vx.abs() > player.walk_speed {
                    velocity.vx = player.walk_speed * velocity.vx.signum();
                }
                if velocity.vy.abs() > player.walk_speed {
                    velocity.vy = player.walk_speed * velocity.vy.signum();
                }
            }
        }
    }
}
struct VelocitySystem;
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
struct RestitutionSystem;
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
                    println!("HERE");
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

struct Example;

impl SimpleState for Example {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        spawn_at_z(data.world, 0.0, 0.0, 1.0)
            .with(Camera::from(Projection::orthographic(0.0, stage.0, 0.0, stage.1)))
            .build();

        let sprite_sheet = load_spritesheet(data.world, get_resource("Sprites"));

        spawn_at(data.world, stage.0 / 2.0, stage.1 / 2.0)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0
            })
            .with(Player::new())
            .with(Velocity { vx: 0.0, vy: 0.0 })
            .with(Physical { is_static: false, size: 8.0 })
            .build();

        spawn_at(data.world, 0.0, 0.0)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0
            })
            .with(Physical { is_static: true, size: 8.0 })
            .build();

        spawn_at(data.world, stage.0, 0.0)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0
            })
            .with(Physical { is_static: false, size: 8.0 })
            .build();

        spawn_at(data.world, 0.0, stage.1)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0
            })
            .with(Physical { is_static: true, size: 8.0 })
            .build();

        spawn_at(data.world, stage.0, stage.1)
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 0
            })
            .with(Physical { is_static: false, size: 8.0 })
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
            .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None)),
    );

    let game_data =
        GameDataBuilder::default()
            .with_bundle(RenderBundle::new(pipe, Some(config))
                .with_sprite_sheet_processor())?
            .with_bundle(input_bundle)?
            .with_bundle(TransformBundle::new())?
            .with(PlayerMovementSystem, "player_move", &[])
            .with(VelocitySystem, "velocity", &[])
            .with(CameraFollow, "camera_follow", &[])
            .with(RestitutionSystem, "restitution", &["velocity"]);
    let mut game = Application::new("./resources", Example, game_data)?;

    game.run();

    Ok(())
}
