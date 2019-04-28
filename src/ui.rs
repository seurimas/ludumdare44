use amethyst::{
    prelude::*,
    ecs::*,
    assets::*,
    core::*,
    core::transform::*,
    renderer::*,
};
use crate::basics::stage;
use crate::utils::*;
use crate::sprites::*;
use crate::nalgebra::{Point3};

#[derive(Debug)]
pub enum Anchor {
    TopLeft, Top, TopRight,
    Left, Middle, Right,
    BottomLeft, Bottom, BottomRight,
    None,
}
impl Anchor {
    fn place(&self, sprite: Sprite, (xoff, yoff): (f32, f32), (bx, by, bw, bh): (f32, f32, f32, f32)) -> (f32, f32) {
        let x = match self {
            Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => {
                bx + xoff + sprite.width / 2.0
            },
            Anchor::Top | Anchor::Middle | Anchor::Bottom => {
                bx + (bw / 2.0) + xoff
            },
            Anchor::TopRight | Anchor::Right | Anchor::BottomRight => {
                bx + bw - xoff - sprite.width / 2.0
            },
            _ => {
                xoff
            }
        };
        let y = match self {
            Anchor::TopLeft | Anchor::Top | Anchor::TopRight => {
                by + bh - yoff - sprite.height / 2.0
            },
            Anchor::Left | Anchor::Middle | Anchor::Right => {
                by + (bh / 2.0) + yoff
            },
            Anchor::BottomLeft | Anchor::Bottom | Anchor::BottomRight => {
                by + yoff + sprite.height / 2.0
            },
            _ => {
                yoff
            }
        };
        (x, y)
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct UiSprite {
    pub anchor: Anchor,
    pub offset: (f32, f32),
}

pub fn draw_sprite(world: &mut World, sprite_number: usize, anchor: Anchor, offset: (f32, f32)) -> EntityBuilder {
    let sprite_sheet = get_sprite_sheet(world);
    spawn_at(world, 0.0, 0.0)
        .with_sprite(sprite_sheet, sprite_number)
        .with(UiSprite { anchor, offset })
}

pub struct UiSpriteSystem;
impl<'s> System<'s> for UiSpriteSystem {
    type SystemData = (
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, ActiveCamera>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, UiSprite>,
        ReadStorage<'s, SpriteRender>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, GlobalTransform>,
    );
    fn run(&mut self, (screen, active, assets, camera, ui_sprite, sprite, mut transform, global) : Self::SystemData) {
        if let Some((camera, camera_global)) = get_camera(active, &camera, &global) {
            for (ui_sprite, sprite, mut transform) in (&ui_sprite, &sprite, &mut transform).join() {
                if let Some(sprite_sheet) = assets.get(&sprite.sprite_sheet) {
                    let sprite = sprite_sheet.sprites[sprite.sprite_number].clone();
                    let screen_box = (0.0, 0.0, stage.0, stage.1);
                    let (x, y) = ui_sprite.anchor.place(sprite, ui_sprite.offset, screen_box);
                    let point: Point3<f32> = [x, y, -0.1].into();
                    let point = camera_global.0.transform_point(&point);
                    transform.set_x(point.x);
                    transform.set_y(point.y);
                    transform.set_z(point.z);
                }
            }
        }
    }
}
#[derive(Debug, Component)]
#[storage(VecStorage)]
pub struct ContinueIcon;
pub struct ContinueTimer {
    pub time_left: f32,
    pub fade_time: f32,
}
pub struct ContinueSystem;
impl<'s> System<'s> for ContinueSystem {
    type SystemData = (
        Write<'s, Option<ContinueTimer>>,
        WriteStorage<'s, UiSprite>,
        ReadStorage<'s, ContinueIcon>,
        Read<'s, Time>,
    );
    fn run(&mut self, (mut to_continue, mut ui_sprite, icon, time) : Self::SystemData) {
        if let Some(to_continue) = to_continue.as_mut() {
            to_continue.time_left -= time.delta_seconds();
            if to_continue.time_left < 0.0 {
                let progress = (-to_continue.time_left / to_continue.fade_time).min(1.0);
                for (ui_sprite, icon) in (&mut ui_sprite, &icon).join() {
                    ui_sprite.offset.1 = -16.0 + progress * 16.0;
                }
            }
        }
    }
}
pub fn can_continue<'s>(timer: Read<'s, Option<ContinueTimer>>) -> bool {
    timer.as_ref().unwrap().time_left < 0.0
}
pub fn want_continue(event: &StateEvent) -> bool {
    if let StateEvent::Window(Event::WindowEvent{ event: WindowEvent::KeyboardInput { .. }, ..}) = &event {
        true
    } else {
        false
    }
}
pub fn init_continue(world: &mut World, time_left: f32, fade_time: f32) {
    world.add_resource(Some(ContinueTimer { time_left, fade_time }));
    draw_sprite(world, CONTINUE, Anchor::Bottom, (0.0, -32.0))
        .with(ContinueIcon)
        .build();
}
