use amethyst::{
    prelude::*,
    ecs::*,
    assets::*,
    core::transform::*,
    renderer::*,
};
use crate::basics::stage;
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
    fn place(&self, sprite: Sprite, (xoff, yoff): (f32, f32), (bx, by, bw, bh): (f32, f32, f32, f32)) -> (f32, f32) {
        let x = match self {
            Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => {
                bx + xoff
            },
            Anchor::Top | Anchor::Middle | Anchor::Bottom => {
                bx + (bw / 2.0) + xoff - sprite.width / 2.0
            },
            Anchor::TopRight | Anchor::Right | Anchor::BottomRight => {
                bx + bw - xoff - sprite.width
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
                by + (bh / 2.0) + yoff - sprite.height / 2.0
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
                    println!("{}", point);
                    transform.set_x(point.x);
                    transform.set_y(point.y);
                    transform.set_z(point.z);
                }
            }
        } else {
            println!("BROKEN");
        }
    }
}
