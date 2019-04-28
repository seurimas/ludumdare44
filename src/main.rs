extern crate amethyst;
#[macro_use]
extern crate specs_derive;
extern crate rand;
mod utils;
mod basics;
mod physics;
mod player;
mod combat;
mod enemies;
mod sprites;
mod world;
mod ui;

use std::path::Path;
use amethyst::{
    prelude::*,
    ecs::*,
    input::*,
    core::transform::*,
    core::*,
    renderer::*,
    assets::*,
    utils::application_root_dir,
};
use nalgebra::{ Vector3, Point3};
use crate::utils::*;
use crate::basics::*;
use crate::physics::*;
use crate::player::*;
use crate::combat::*;
use crate::enemies::*;
use crate::sprites::*;
use crate::world::*;
use crate::ui::*;

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
        let x = hitbox.offset.0 - hitbox.width / 2.0;
        let y = hitbox.offset.1 - hitbox.height / 2.0;
        let width = hitbox.width;
        let height = hitbox.height;
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
        Entities<'s>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut animation, mut hitstate, mut velocity, mut sprite, rotation, entities, time) : Self::SystemData) {
        for (animation, mut sprite, entity) in (&mut animation, &mut sprite, &entities).join() {
            let mut hitstate = hitstate.get_mut(entity);
            if let Some(hitstate) = hitstate.as_mut() {
                hitstate.clear(ENEMY_ATTACK_BOX);
                hitstate.clear(PLAYER_ATTACK_BOX);
            }
            if animation.active() {
                let frame = animation.step(time.delta_seconds());
                if let Some(frame) = frame {
                    for i in 0..HITSTATE_SIZE {
                        if let Some(hitstate) = hitstate.as_mut() {
                            if let Some(hitbox) = frame.hitboxes[i] {
                                hitstate.set(i, hitbox.width, hitbox.height, hitbox.offset);
                            }
                        }
                    }
                    if let (Some(frame_velocity), Some(velocity)) = (frame.velocity, velocity.get_mut(entity)) {
                        let vx;
                        let vy;
                        if let Some(rotation) = rotation.get(entity) {
                            let (rx, ry) = rotation.rotate(frame_velocity);
                            vx = rx;
                            vy = ry;
                        } else {
                            vx = frame_velocity.0;
                            vy = frame_velocity.1;
                        }
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
        WriteStorage<'s, HitState>,
    );
    fn run(&mut self, (rotation, mut transform, mut hitstate) : Self::SystemData) {
        for (rotation, transform) in (&rotation, &mut transform).join() {
            rotation.rotate_euler(transform);
        }
        for (rotation, hitstate) in (&rotation, &mut hitstate).join() {
            hitstate.rotate(*rotation);
        }
    }
}

struct InitializingGameState {
    progress: ProgressCounter,
    sprite_sheet: Option<SpriteSheetHandle>,
}
impl InitializingGameState {
    fn new() -> InitializingGameState {
        InitializingGameState {
            progress: ProgressCounter::new(),
            sprite_sheet: None,
        }
    }
}
impl SimpleState for InitializingGameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.add_resource(DebugLines::new().with_capacity(100));
        data.world.add_resource(DebugLinesParams {
            line_width: 100.0,
        });
        data.world.register::<StaggerAnimation>();
        let sprite_sheet = load_spritesheet(data.world, get_resource("Sprites"), &mut self.progress);
        let main_sprite = MainSprite(sprite_sheet.clone());
        data.world.add_resource(main_sprite);
        self.sprite_sheet = Some(sprite_sheet);
    }
    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        match self.progress.complete() {
            Completion::Complete => {
                Trans::Switch(Box::new(MainGameState {
                    sprite_sheet: self.sprite_sheet.clone().unwrap(),
                }))
            },
            _ => Trans::None
        }
    }
}

struct GameOverState {
    sprite_sheet: SpriteSheetHandle,
}
impl SimpleState for GameOverState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_all();
        spawn_at_z(data.world, 0.0, 0.0, 1.0)
            .with(Camera::from(Projection::orthographic(0.0, stage.0, 0.0, stage.1)))
            .build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_G, Anchor::Middle, (-24.0, 8.0)).build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_A, Anchor::Middle, (-8.0, 8.0)).build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_M, Anchor::Middle, (8.0, 8.0)).build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_E_0, Anchor::Middle, (24.0, 8.0)).build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_O, Anchor::Middle, (-24.0, -8.0)).build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_V, Anchor::Middle, (-8.0, -8.0)).build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_E_1, Anchor::Middle, (8.0, -8.0)).build();
        draw_sprite(data.world, self.sprite_sheet.clone(), GAME_OVER_R, Anchor::Middle, (24.0, -8.0)).build();
    }
}

struct MainGameState {
    sprite_sheet: SpriteSheetHandle,
}
impl SimpleState for MainGameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_all();
        spawn_at_z(data.world, 0.0, 0.0, 1.0)
            .with(Camera::from(Projection::orthographic(0.0, stage.0, 0.0, stage.1)))
            .build();

        let mut hitboxes = HitState::new();
        hitboxes.set(ENEMY_HITTABLE_BOX, 16.0, 16.0, (0.0, 0.0));
        let hearts = [
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (8.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (24.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (40.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (56.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (72.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (88.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (104.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (120.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (136.0, 0.0)).build(),
            draw_sprite(data.world, self.sprite_sheet.clone(), FULL_HEART, Anchor::TopLeft, (152.0, 0.0)).build(),
        ];

        spawn_at(data.world, stage.0 / 2.0, stage.1 / 2.0)
            .with_sprite(self.sprite_sheet.clone(), PLAYER_IDLE)
            .with(Player::new(hearts))
            .with(hitboxes)
            .with(AnimationController::new())
            .with(Health { left: 8, max: 8 })
            .with_physics(4.0)
            .build();


        fill_world(data.world, &self.sprite_sheet, 32, 32, FLOOR_EMPTY);
        draw_wall(data.world, &self.sprite_sheet, (0, 0), (32, 1), WALL);
        draw_wall(data.world, &self.sprite_sheet, (0, 0), (1, 32), WALL);
        draw_wall(data.world, &self.sprite_sheet, (31, 0), (1, 32), WALL);
        draw_wall(data.world, &self.sprite_sheet, (0, 31), (32, 1), WALL);

        let mut hitboxes = HitState::new();
        hitboxes.set(PLAYER_HITTABLE_BOX, 16.0, 16.0, (0.0, 0.0));

        spawn_goblin(data.world, 0.0, 0.0)
            .build();

        spawn_goblin(data.world, stage.0, 0.0)
            .build();

        let chest = spawn_at(data.world, 0.0, stage.1)
            .with_sprite(self.sprite_sheet.clone(), 0)
            .with(hitboxes.clone())
            .with(AnimationController::new())
            .with(Health { max: 2, left: 2 })
            .with_physics(8.0)
            .build();
        heart_spin(data.world, self.sprite_sheet.clone(), -4.0, 8.0)
            .with(Parent { entity: chest })
            .build();
        heart_spin(data.world, self.sprite_sheet.clone(), 4.0, 8.0)
            .with(Parent { entity: chest })
            .build();

        let chest = spawn_at(data.world, stage.0, stage.1)
            .with_sprite(self.sprite_sheet.clone(), 0)
            .with(hitboxes.clone())
            .with(AnimationController::new())
            .with(Health { max: 2, left: 2 })
            .with_physics(8.0)
            .build();
        spend_heart_spin(data.world, self.sprite_sheet.clone(), -4.0, 8.0)
            .with(Parent { entity: chest })
            .build();
        spend_heart_spin(data.world, self.sprite_sheet.clone(), 4.0, 8.0)
            .with(Parent { entity: chest })
            .build();
    }
    fn update(&mut self, mut data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let player = data.world.exec(|player: ReadStorage< Player>| {
            let mut alive = false;
            for (player) in (&player).join() {
                alive = true;
            }
            alive
        });
        if player {
            Trans::None
        } else {
            Trans::Switch(Box::new(GameOverState {
                sprite_sheet: self.sprite_sheet.clone(),
            }))
        }
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
            .with_bundle(input_bundle)?
            .with_bundle(TransformBundle::new())?
            .with(PlayerMovementSystem::new(), "player_move", &[])
            .with(ChaseAndWanderSystem, "chase_and_wander", &[])
            .with(PlayerAttackSystem::new(), "player_attack", &["player_move"])
            .with(CameraFollow, "camera_follow", &[])
            .with(UiSpriteSystem, "ui_sprite", &["camera_follow", "transform_system"])
            .with(RotationSystem, "rotation", &[])
            .with(AnimationSystem, "animation", &[])
            .with(VelocitySystem, "velocity", &["animation"])
            .with(RestitutionSystem, "restitution", &["velocity"])
            .with(DebugDrawHitboxes, "debug_hitboxes", &[])
            .with(PlayerHeartSystem, "hearts", &[])
            .with(PlayerDamageSystem, "player_damage", &["animation"])
            .with(EnemyDamageSystem, "enemy_damage", &["animation"])
            .with(SightSystem, "sight", &["animation"])
            .with(AimingSystem, "aim", &["sight"])
            .with(DeathSystem, "death", &["player_damage", "enemy_damage"])
            .with_barrier()
            .with_bundle(RenderBundle::new(pipe, Some(config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&[]))?;
    let mut game = Application::new("./resources", InitializingGameState::new(), game_data)?;

    game.run();

    Ok(())
}
