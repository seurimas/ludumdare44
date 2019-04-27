use amethyst::{
    prelude::*,
    ecs::*,
    renderer::Rgba,
    core::*,
    input::*,
};
use crate::basics::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Player {
    pub can_move: bool,
    pub walk_accel: f32,
    pub walk_speed: f32,
}
impl Player {
    pub fn new() -> Player {
        Player {
            can_move: true,
            walk_accel: 400.0,
            walk_speed: 100.0,
        }
    }
}

pub struct PlayerMovementSystem;
impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Velocity>,
        WriteStorage<'s, Rotation>,
        Read<'s, InputHandler<String, String>>,
        Read<'s, Time>,
        Entities<'s>,
    );
    fn run(&mut self, (players, mut velocities, mut rotations, input, time, entities) : Self::SystemData) {
        let deacc_factor = 3.0;
        for (player, mut velocity, entity) in (&players, &mut velocities, &entities).join() {
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
                if x_tilt != 0.0 || y_tilt != 0.0 {
                    let rotation;
                    if velocity.vx.abs() > velocity.vy.abs() {
                        if velocity.vx < 0.0 {
                            rotation = Some(Rotation::West);
                        } else {
                            rotation = Some(Rotation::East);
                        }
                    } else {
                        if velocity.vy < 0.0 {
                            rotation = Some(Rotation::South);
                        } else {
                            rotation = Some(Rotation::North);
                        }
                    }
                    if let Some(rotation) = rotation {
                        if let Ok(_) = rotations.insert(entity, rotation) {

                        }
                    }
                }
            }
        }
    }
}

const PLAYER_ATTACK: usize = 0;

pub struct PlayerAttackSystem {
    attacks: Vec<HitboxAnimation>,
}
impl PlayerAttackSystem {
    pub fn new() -> PlayerAttackSystem {
        let mut attacks = Vec::new();
        let mut base_attack = HitboxAnimation::new();
        let frame = base_attack.add_frame((0.0, 0.0), 0.25);
        base_attack.set_hitbox(frame, PLAYER_ATTACK, Hitbox::new_at(8.0, (8.0, 0.0)));
        let frame = base_attack.add_frame((0.0, 0.0), 0.25);
        base_attack.set_hitbox(frame, PLAYER_ATTACK, Hitbox::new_at(4.0, (10.0, 0.0)));
        attacks.push(base_attack);
        PlayerAttackSystem {
            attacks,
        }
    }
}
impl<'s> System<'s> for PlayerAttackSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, AnimationController>,
        Read<'s, InputHandler<String, String>>,
    );
    fn run(&mut self, (player, mut animation_controller, input) : Self::SystemData) {
        if let Some(true) = input.action_is_down("attack") {
            for (player, mut animation_controller) in (&player, &mut animation_controller).join() {
                animation_controller.start(self.attacks[0].clone());
                println!("Attacking");
            }
        }
    }
}
