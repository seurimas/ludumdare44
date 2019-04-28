use amethyst::{
    prelude::*,
    ecs::*,
    renderer::*,
    core::*,
    input::*,
    core::transform::*,
};
use crate::sprites::*;
use crate::basics::*;
use crate::enemies::*;
use crate::utils::*;

pub struct WorldState {
    enemies_alive: usize,
    on_portal: bool,
    opened_portal: bool,
}
impl WorldState {
    pub fn new() -> WorldState {
        WorldState {
            enemies_alive: 1,
            on_portal: false,
            opened_portal: false,
        }
    }
}

const WORLD_BASE: (i32, i32) = (-256, -256);
const TILE_SIZE: (i32, i32) = (16, 16);

pub fn spawn_world_tile(world: &mut World, sprite_sheet: SpriteSheetHandle, x: i32, y: i32, sprite_number: usize) -> EntityBuilder {
    let (tx, ty) = (WORLD_BASE.0 + x * TILE_SIZE.0, WORLD_BASE.1 + y * TILE_SIZE.1);
    spawn_at_z(world, tx as f32, ty as f32, -1.0)
        .with_sprite(sprite_sheet.clone(), sprite_number)
}

pub fn fill_world(world: &mut World, sprite_sheet: &SpriteSheetHandle, width: i32, height: i32, sprite_number: usize) {
    for x in 0..width {
        for y in 0..height {
            spawn_world_tile(world, sprite_sheet.clone(), x, y, sprite_number).build();
        }
    }
}

pub fn draw_wall(world: &mut World, sprite_sheet: &SpriteSheetHandle, (x, y): (i32, i32), (width, height): (i32, i32), sprite_number: usize) {
    for x in x..(x + width) {
        for y in y..(y + height) {
            spawn_world_tile(world, sprite_sheet.clone(), x, y, sprite_number)
                .build();
        }
    }
    let (tx, ty) = (WORLD_BASE.0 + x * TILE_SIZE.0, WORLD_BASE.1 + y * TILE_SIZE.1);
    let (w, h) = (TILE_SIZE.0 * width, TILE_SIZE.1 * height);
    spawn_at_z(world, (tx + w / 2 - TILE_SIZE.0 / 2) as f32, (ty + h / 2 - TILE_SIZE.1 / 2) as f32, -1.0)
        .with(Physical::new_wall(w as f32, h as f32))
        .build();
}
#[derive(Debug, Component, Default)]
#[storage(NullStorage)]
pub struct Portal;

pub struct PortalSystem;
impl<'s> System<'s> for PortalSystem {
    type SystemData = (
        Write<'s, Option<WorldState>>,
        WriteStorage<'s, AnimationController>,
        ReadStorage<'s, Portal>,
        ReadStorage<'s, Enemy>,
    );
    fn run(&mut self, (mut world_state, mut animation, portal, enemies) : Self::SystemData) {
        if let Some(world_state) = world_state.as_mut() {
            world_state.enemies_alive = 0;
            for (enemy) in (enemies).join() {
                world_state.enemies_alive += 1;
            }
            println!("{}", world_state.enemies_alive);
            if world_state.enemies_alive == 0 && !world_state.opened_portal {
                for (animation_controller, portal) in (&mut animation, &portal).join() {
                    animation_controller.start_loop(portal_animation(0.15));
                    world_state.opened_portal = true;
                }
            }
        }
    }
}
pub struct ExitSystem;
impl<'s> System<'s> for ExitSystem {
    type SystemData = (
        ReadStorage<'s, HitState>,
        WriteStorage<'s, Transform>,
        Entities<'s>,
        <Self as HitboxCollisionSystem<'s>>::ExtraData,
    );
    fn run(&mut self, mut system_data: Self::SystemData) {
        let extra = &mut system_data.3;
        let world_state = &mut extra.0;
        if let Some(mut world_state) = world_state.as_mut() {
            world_state.on_portal = false;
        }
        self.check_collisions(system_data);
    }
}
impl<'s> HitboxCollisionSystem<'s> for ExitSystem {
    type ExtraData = (
        Write<'s, Option<WorldState>>,
    );
    fn collide(&self, collision: HitboxCollision, entity_a: Entity, entity_b: Entity, transforms: &WriteStorage<'s, Transform>, extra: &mut Self::ExtraData) {
        let world_state = &mut extra.0;
        if let Some(mut world_state) = world_state.as_mut() {
            world_state.on_portal = true;
        }
    }
    fn source() -> usize {
        PORTAL_BOX
    }
    fn target() -> usize {
        PLAYER_INTERACT_BOX
    }
}

pub fn want_advance<'s>((world_state, input): (Read<'s, Option<WorldState>>, Read<'s, InputHandler<String, String>>)) -> bool {
    if let Some(world_state) = world_state.as_ref() {
        if let Some(true) = input.action_is_down("interact") {
            world_state.enemies_alive == 0 && world_state.on_portal
        } else {
            false
        }
    } else {
        false
    }
}

pub fn init_world(world: &mut World) {
    world.add_resource(Some(WorldState::new()));
}


fn portal_animation(speed: f32) -> HitboxAnimation {
    let mut animation = HitboxAnimation::new();
    let frame = animation.add_frame(speed);
    animation.set_sprite(frame, PORTAL_SPIN[0]);
    let frame = animation.add_frame(speed);
    animation.set_sprite(frame, PORTAL_SPIN[1]);
    let frame = animation.add_frame(speed);
    animation.set_sprite(frame, PORTAL_SPIN[2]);
    let frame = animation.add_frame(speed);
    animation.set_sprite(frame, PORTAL_SPIN[3]);
    let frame = animation.add_frame(speed);
    animation.set_sprite(frame, PORTAL_SPIN[4]);
    let frame = animation.add_frame(speed);
    animation.set_sprite(frame, PORTAL_SPIN[5]);
    let frame = animation.add_frame(speed);
    animation.set_sprite(frame, PORTAL_SPIN[6]);
    animation
}

fn heart_spin_animation(rotation_speed: f32, pause_duration: f32, heart_spin: [usize; 8]) -> HitboxAnimation {
    let mut animation = HitboxAnimation::new();
    let frame = animation.add_frame(pause_duration);
    animation.set_sprite(frame, heart_spin[0]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[1]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[2]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[3]);
    let frame = animation.add_frame(pause_duration);
    animation.set_sprite(frame, heart_spin[4]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[5]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[6]);
    let frame = animation.add_frame(rotation_speed);
    animation.set_sprite(frame, heart_spin[7]);
    animation
}

pub fn heart_spin(world: &mut World, x: f32, y: f32) -> EntityBuilder {
    let sprite_sheet = get_sprite_sheet(world);
    let mut animation_controller = AnimationController::new();
    let rotation_speed = 0.15;
    let pause_duration = 0.5;
    animation_controller.start_loop(heart_spin_animation(rotation_speed, pause_duration, HEART_SPIN));
    spawn_at(world, x, y)
        .with_sprite(sprite_sheet, HEART_SPIN[0])
        .with(animation_controller)
}

pub fn spend_heart_spin(world: &mut World, x: f32, y: f32) -> EntityBuilder {
    let sprite_sheet = get_sprite_sheet(world);
    let mut animation_controller = AnimationController::new();
    let rotation_speed = 0.25;
    let pause_duration = 0.25;
    animation_controller.start_loop(heart_spin_animation(rotation_speed, pause_duration, SPEND_HEART_SPIN));
    spawn_at(world, x, y)
        .with_sprite(sprite_sheet, SPEND_HEART_SPIN[0])
        .with(animation_controller)
}

pub fn portal(world: &mut World, x: f32, y: f32) -> EntityBuilder {
    let sprite_sheet = get_sprite_sheet(world);
    let mut animation_controller = AnimationController::new();
    let mut hitstate = HitState::new();
    hitstate.set(PORTAL_BOX, 8.0, 8.0, (0.0, 0.0));
    spawn_at(world, x, y)
        .with_sprite(sprite_sheet, PORTAL_CLOSED)
        .with(Portal)
        .with(hitstate)
        .with(animation_controller)
}
