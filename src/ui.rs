use amethyst::{
    prelude::*,
    ecs::*,
    core::transform::*,
    renderer::*,
};
use crate::utils::*;
use crate::nalgebra::{Point3};

#[derive(Debug)]
pub enum Anchor {
    TopLeft, Top, TopRight,
    Left, Middle, Right,
    BottomLeft, Bottom, BottomRight,
    None,
}
impl Anchor {
    fn place(&self, (xoff, yoff): (f32, f32), (bx, by, bw, bh): (f32, f32, f32, f32)) -> (f32, f32) {
        let x = match self {
            Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => {
                bx + xoff
            },
            Anchor::Top | Anchor::Middle | Anchor::Bottom => {
                bx + (bw / 2.0) + xoff
            },
            Anchor::TopRight | Anchor::Right | Anchor::BottomRight => {
                bx + bw - xoff
            },
            _ => {
                xoff
            }
        };
        let y = match self {
            Anchor::TopLeft | Anchor::Top | Anchor::TopRight => {
                by + bh - yoff
            },
            Anchor::Left | Anchor::Middle | Anchor::Right => {
                by + (bh / 2.0) + yoff
            },
            Anchor::BottomLeft | Anchor::Bottom | Anchor::BottomRight => {
                by + yoff
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

pub fn draw_sprite(world: &mut World, sprite_sheet: SpriteSheetHandle, sprite_number: usize, anchor: Anchor, offset: (f32, f32)) -> EntityBuilder {
    world.create_entity()
        .with(at(0.0, 0.0))
        .with(GlobalTransform::default())
        .with(SpriteRender { sprite_sheet, sprite_number })
        .with(UiSprite { anchor, offset })
}

pub struct UiSpriteSystem;
impl<'s> System<'s> for UiSpriteSystem {
    type SystemData = (
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, ActiveCamera>,
        ReadStorage<'s, Camera>,
        ReadStorage<'s, UiSprite>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, GlobalTransform>,
    );
    fn run(&mut self, (screen, active, camera, ui_sprite, mut transform, global) : Self::SystemData) {
        if let Some((camera, camera_global)) = get_camera(active, &camera, &global) {
            for (ui_sprite, mut transform) in (&ui_sprite, &mut transform).join() {
                let point: Point3<f32> = [ui_sprite.offset.0, ui_sprite.offset.1, -0.1].into();
                // let point: Point3<f32> = [-100.0, -100.0, 1.0].into();
                let point = camera_global.0.transform_point(&point);
                println!("{}", point);
                transform.set_x(point.x);
                transform.set_y(point.y);
                transform.set_z(point.z);
            }
        } else {
            println!("BROKEN");
        }
    }
}
