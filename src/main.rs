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
mod drops;
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
    utils::fps_counter::*,
    audio::output::*,
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
use crate::drops::*;
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
        Option<Read<'s, FPSCounter>>,
    );
    fn run(&mut self, (mut lines, hitboxes, transform, physical, fps) : Self::SystemData) {
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
        if let Some(fps) = fps {
            println!("{}", fps.frame_fps());
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
        data.world.add_resource(AssetStorage::<amethyst::audio::Source>::new());
        data.world.add_resource(DebugLines::new().with_capacity(100));
        data.world.add_resource(DebugLinesParams {
            line_width: 100.0,
        });
        init_output(&mut data.world.res);
        data.world.add_resource::<Option<ContinueTimer>>(None);
        data.world.register::<StaggerAnimation>();
        let sprite_sheet = load_spritesheet(data.world, get_resource("Sprites"), &mut self.progress);
        let swing_sound = load_sound(data.world, get_resource("swing.wav"), &mut self.progress);
        let player_hit_sound = load_sound(data.world, get_resource("hit_enemy.wav"), &mut self.progress);
        let enemy_hit_sound = load_sound(data.world, get_resource("hit_player.wav"), &mut self.progress);
        let purchase_sound = load_sound(data.world, get_resource("purchase.wav"), &mut self.progress);
        let main_sprite = MainAssets {
            sprite_sheet: sprite_sheet.clone(),
            swing_sound: swing_sound.clone(),
            player_hit_sound: player_hit_sound.clone(),
            enemy_hit_sound: enemy_hit_sound.clone(),
            purchase_sound: purchase_sound.clone(),
        };
        data.world.add_resource(main_sprite);
        self.sprite_sheet = Some(sprite_sheet);
    }
    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        println!("Errors: {:?} {} {}", self.progress.errors(), self.progress.num_loading(), self.progress.num_failed());
        match self.progress.complete() {
            Completion::Complete => {
                Trans::Switch(Box::new(TutorialState {
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
        init_continue(data.world, 3.0, 2.0);
        spawn_at_z(data.world, 0.0, 0.0, 1.0)
            .with(Camera::from(Projection::orthographic(0.0, stage.0, 0.0, stage.1)))
            .build();
        draw_sprite(data.world, GAME_OVER_G, Anchor::Middle, (-24.0, 8.0)).build();
        draw_sprite(data.world, GAME_OVER_A, Anchor::Middle, (-8.0, 8.0)).build();
        draw_sprite(data.world, GAME_OVER_M, Anchor::Middle, (8.0, 8.0)).build();
        draw_sprite(data.world, GAME_OVER_E_0, Anchor::Middle, (24.0, 8.0)).build();
        draw_sprite(data.world, GAME_OVER_O, Anchor::Middle, (-24.0, -8.0)).build();
        draw_sprite(data.world, GAME_OVER_V, Anchor::Middle, (-8.0, -8.0)).build();
        draw_sprite(data.world, GAME_OVER_E_1, Anchor::Middle, (8.0, -8.0)).build();
        draw_sprite(data.world, GAME_OVER_R, Anchor::Middle, (24.0, -8.0)).build();
    }
    fn on_stop(&mut self, data: StateData<GameData>) {
        data.world.add_resource::<Option<ContinueTimer>>(None);
    }
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if want_continue(&event) && data.world.exec(can_continue) {
            Trans::Switch(Box::new(MainGameState {
                sprite_sheet: self.sprite_sheet.clone(),
                player_state: PlayerState::new(),
            }))
        } else {
            Trans::None
        }
    }
}

struct TutorialState {
    sprite_sheet: SpriteSheetHandle,
}
impl SimpleState for TutorialState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_all();
        data.world.read_resource::<Output>().name();
        init_continue(data.world, 0.0, 2.0);
        spawn_at_z(data.world, 0.0, 0.0, 1.0)
            .with(Camera::from(Projection::orthographic(0.0, stage.0, 0.0, stage.1)))
            .build();
        draw_sprite(data.world, MOVEMENT, Anchor::TopLeft, (0.0, 8.0)).build();
        draw_sprite(data.world, WASD_UI, Anchor::TopLeft, (80.0, 0.0)).build();
        draw_sprite(data.world, ATTACK, Anchor::TopLeft, (0.0, 32.0)).build();
        draw_sprite(data.world, SPACE_UI, Anchor::TopLeft, (80.0, 32.0)).build();
        draw_sprite(data.world, INTERACT, Anchor::TopLeft, (0.0, 48.0)).build();
        draw_sprite(data.world, E_UI, Anchor::TopLeft, (80.0, 48.0)).build();

        draw_sprite(data.world, BUY_UPGRADES, Anchor::TopRight, (0.0, 0.0)).build();
        draw_sprite(data.world, SPEND_HEART_SPIN[0], Anchor::TopRight, (12.0, 26.0)).build();
        draw_sprite(data.world, CHEST_SPRITE, Anchor::TopRight, (8.0, 34.0)).build();
        draw_sprite(data.world, SPEND_HEART_SPIN[0], Anchor::TopRight, (44.0, 26.0)).build();
        draw_sprite(data.world, SPEND_HEART_SPIN[0], Anchor::TopRight, (52.0, 26.0)).build();
        draw_sprite(data.world, CHEST_SPRITE, Anchor::TopRight, (44.0, 34.0)).build();
    }
    fn on_stop(&mut self, data: StateData<GameData>) {
        data.world.add_resource::<Option<ContinueTimer>>(None);
    }
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if want_continue(&event) && data.world.exec(can_continue) {
            Trans::Switch(Box::new(MainGameState {
                sprite_sheet: self.sprite_sheet.clone(),
                player_state: PlayerState::new(),
            }))
        } else {
            Trans::None
        }
    }
}

struct MainGameState {
    sprite_sheet: SpriteSheetHandle,
    player_state: PlayerState,
}
impl SimpleState for MainGameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        data.world.delete_all();
        init_world(data.world);
        spawn_at_z(data.world, 0.0, 0.0, 1.0)
            .with(Camera::from(Projection::orthographic(0.0, stage.0, 0.0, stage.1)))
            .build();

        spawn_player(data.world, &self.player_state);

        fill_world(data.world, &self.sprite_sheet, 32, 32, FLOOR_EMPTY);
        draw_wall(data.world, &self.sprite_sheet, (0, 0), (32, 1), WALL);
        draw_wall(data.world, &self.sprite_sheet, (0, 0), (1, 32), WALL);
        draw_wall(data.world, &self.sprite_sheet, (31, 0), (1, 32), WALL);
        draw_wall(data.world, &self.sprite_sheet, (0, 31), (32, 1), WALL);

        spawn_goblin(data.world, 0.0, 0.0)
            .build();

        spawn_goblin(data.world, stage.0, 0.0)
            .build();

        portal(data.world, stage.0 / 2.0, stage.1 / 2.0).build();

        spawn_chest(data.world, 0.0, stage.1, 1, Upgrade::HeartBracelet);
        spawn_chest(data.world, stage.0, stage.1, 2, Upgrade::GoldenAegis);
    }
    fn update(&mut self, mut data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let player = data.world.exec(|player: ReadStorage< Player>| {
            let mut alive = false;
            for (player) in (&player).join() {
                alive = true;
            }
            alive
        });
        if data.world.exec(want_advance) {
            Trans::Switch(Box::new(MainGameState {
                sprite_sheet: self.sprite_sheet.clone(),
                player_state: self.player_state.advance(data.world.exec(get_health)),
            }))
        } else if player {
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
            .with_bundle(FPSCounterBundle)?
            .with(Processor::<amethyst::audio::Source>::new(), "source_processor", &[])
            .with(ContinueSystem, "continue", &[])
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
            .with(PortalSystem, "portal", &[])
            .with(PurchaseSystem, "purchase", &[])
            .with(ExitSystem, "exit", &["portal"])
            .with_barrier()
            .with_bundle(RenderBundle::new(pipe, Some(config))
                .with_sprite_sheet_processor()
                .with_sprite_visibility_sorting(&[]))?;
    let mut game = Application::new("./resources", InitializingGameState::new(), game_data)?;

    game.run();

    Ok(())
}
