use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use bevy::{prelude::*, sprite::Anchor, tasks::AsyncComputeTaskPool};
use dcl_common::Parcel;
use imagesize::size;

use crate::{
    components,
    renderer::{
        constants::{LAYERS_DISTANCE, PARCEL_SIZE_X, PARCEL_SIZE_Y},
        scene_loader::world_location_to_parcel,
    },
};

use super::transform::get_parcel_rect;

#[derive(Bundle, Default)]
pub struct SpriteRenderer {
    pub sprite: SpriteBundle,
    pub renderer: components::SpriteRenderer,
}

impl SpriteRenderer {
    pub fn from_texture(
        sprite_renderer_component: &dcl2d_ecs_v1::components::SpriteRenderer,
        transform: &Transform,
        texture: Handle<Image>,
        image_size: Vec2,
        parcels: &Vec<Parcel>,
        level_id: usize,
    ) -> Self {
        let mut final_transform = *transform;
        final_transform.translation = Vec3 {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z + sprite_renderer_component.layer as f32 * LAYERS_DISTANCE,
        };

        let anchor =
            dcl_anchor_to_bevy_anchor((sprite_renderer_component).anchor.clone(), image_size);

        let parcels_overlapping = match level_id {
            0 => get_parcels_overlapping(final_transform.translation, image_size, parcels, &anchor),
            _ => Vec::default(),
        };

        let color = Color::Rgba {
            red: (sprite_renderer_component).color.r,
            green: (sprite_renderer_component).color.g,
            blue: (sprite_renderer_component).color.b,
            alpha: 0.,
        };

        let default_color = Color::Rgba {
            red: (sprite_renderer_component).color.r,
            green: (sprite_renderer_component).color.g,
            blue: (sprite_renderer_component).color.b,
            alpha: (sprite_renderer_component).color.a,
        };

        let final_texture = if level_id == 0
            && (image_size.x > PARCEL_SIZE_X * 1.5 || image_size.y > PARCEL_SIZE_Y * 1.5)
        {
            Handle::default()
        } else {
            texture
        };

        let sprite = SpriteBundle {
            sprite: Sprite {
                color,
                flip_x: sprite_renderer_component.flip.x,
                flip_y: sprite_renderer_component.flip.y,
                anchor,
                ..default()
            },
            transform: final_transform,
            texture: final_texture,
            ..default()
        };

        SpriteRenderer {
            renderer: components::SpriteRenderer {
                default_color,
                parcels_overlapping,
                parent_parcels: parcels.clone(),
                transparency_alpha: 1.,
                ..default()
            },
            sprite,
        }
    }

    pub fn load_async<P>(
        sprite_renderer_component: &dcl2d_ecs_v1::components::SpriteRenderer,
        transform: &Transform,
        image_asset_path: P,
        asset_server: &AssetServer,
        parcels: Vec<Parcel>,
        level_id: usize,
    ) -> components::LoadingSpriteRenderer
    where
        P: AsRef<Path>,
    {
        //Download scenes
        let thread_pool = AsyncComputeTaskPool::get();

        let image_asset_path = image_asset_path.as_ref().to_path_buf();
        let asset_server = asset_server.clone();
        let task = thread_pool.spawn(async move {
            let mut absolute_path = std::fs::canonicalize(PathBuf::from_str(".").unwrap()).unwrap();
            absolute_path.push("assets");
            absolute_path.push(&image_asset_path);

            let image_size = match size(&absolute_path) {
                Ok(v) => Vec2::new(v.width as f32, v.height as f32),
                Err(e) => {
                    println!("{:} {}", e, absolute_path.display());
                    Vec2::new(0.0, 0.0)
                }
            };

            let texture = asset_server.load(image_asset_path);
            (texture, image_size)
        });

        components::LoadingSpriteRenderer {
            task,
            sprite_renderer_component: sprite_renderer_component.clone(),
            transform: *transform,
            parcels,
            level_id,
        }
    }
}

fn dcl_anchor_to_bevy_anchor(anchor: dcl2d_ecs_v1::Anchor, size: Vec2) -> Anchor {
    match anchor {
        dcl2d_ecs_v1::Anchor::BottomCenter => Anchor::BottomCenter,
        dcl2d_ecs_v1::Anchor::BottomLeft => Anchor::BottomLeft,
        dcl2d_ecs_v1::Anchor::BottomRight => Anchor::BottomRight,
        dcl2d_ecs_v1::Anchor::Center => Anchor::Center,
        dcl2d_ecs_v1::Anchor::CenterLeft => Anchor::CenterLeft,
        dcl2d_ecs_v1::Anchor::CenterRight => Anchor::CenterRight,
        dcl2d_ecs_v1::Anchor::Custom(vec) => {
            let x = vec.x.min(size.x as i32).max(0);
            let y = vec.y.min(size.y as i32).max(0);
            Anchor::Custom(Vec2::new(x as f32 - size.x / 2.0, y as f32 - size.y / 2.0) / size)
        }
        dcl2d_ecs_v1::Anchor::TopCenter => Anchor::TopCenter,
        dcl2d_ecs_v1::Anchor::TopLeft => Anchor::TopLeft,
        dcl2d_ecs_v1::Anchor::TopRight => Anchor::TopRight,
    }
}

fn get_parcels_overlapping(
    location: Vec3,
    size: Vec2,
    parcels: &Vec<Parcel>,
    anchor: &Anchor,
) -> Vec<Parcel> {
    let mut overlapping_parcels: Vec<Parcel> = Vec::default();

    if parcels.is_empty() {
        return overlapping_parcels;
    }

    let mut bounds = get_parcel_rect(&parcels[0]);

    for parcel in parcels {
        let parcel_rect = get_parcel_rect(parcel);
        if parcel_rect.min.x < bounds.min.x {
            bounds.min.x = parcel_rect.min.x;
        }

        if parcel_rect.min.y < bounds.min.y {
            bounds.min.y = parcel_rect.min.y;
        }

        if parcel_rect.max.x > bounds.max.x {
            bounds.max.x = parcel_rect.max.x;
        }

        if parcel_rect.max.y > bounds.max.y {
            bounds.max.y = parcel_rect.max.y;
        }
    }

    let center = Vec3 {
        x: (bounds.min.x + bounds.max.x) / 2.,
        y: (bounds.min.y + bounds.max.y) / 2.,
        z: (bounds.min.y + bounds.max.y) / -2.,
    };

    let location = get_translation_by_anchor(&size, &location, anchor);
    let target_location = center + location;

    let min_x = target_location.x - size.x / 2.;
    let max_x = target_location.x + size.x / 2.;
    let min_y = target_location.y - size.y / 2.;
    let max_y = target_location.y + size.y / 2.;

    if min_x < bounds.min.x {
        overlapping_parcels.push(world_location_to_parcel(&Vec3 {
            x: min_x,
            y: center.y,
            z: -center.y,
        }));

        if min_y < bounds.min.y {
            overlapping_parcels.push(world_location_to_parcel(&Vec3 {
                x: min_x,
                y: min_y,
                z: -min_y,
            }));
        } else if max_y > bounds.max.y {
            overlapping_parcels.push(world_location_to_parcel(&Vec3 {
                x: min_x,
                y: max_y,
                z: -max_y,
            }));
        }
    } else if max_x > bounds.max.x {
        overlapping_parcels.push(world_location_to_parcel(&Vec3 {
            x: max_x,
            y: center.y,
            z: -center.y,
        }));

        if min_y < bounds.min.y {
            overlapping_parcels.push(world_location_to_parcel(&Vec3 {
                x: max_x,
                y: min_y,
                z: -min_y,
            }));
        } else if max_y > bounds.max.y {
            overlapping_parcels.push(world_location_to_parcel(&Vec3 {
                x: max_x,
                y: max_y,
                z: -max_y,
            }));
        }
    }

    if min_y < bounds.min.y {
        overlapping_parcels.push(world_location_to_parcel(&Vec3 {
            x: center.x,
            y: min_y,
            z: -min_y,
        }));
    } else if max_y > bounds.max.y {
        overlapping_parcels.push(world_location_to_parcel(&Vec3 {
            x: center.x,
            y: max_y,
            z: -max_y,
        }));
    }

    overlapping_parcels
}

pub fn get_translation_by_anchor(size: &Vec2, translation: &Vec3, anchor: &Anchor) -> Vec3 {
    match anchor {
        Anchor::BottomCenter => Vec3 {
            x: translation.x,
            y: translation.y + size.y / 2.,
            z: translation.z,
        },
        Anchor::BottomLeft => Vec3 {
            x: translation.x + size.x / 2.,
            y: translation.y + size.y / 2.,
            z: translation.z,
        },
        Anchor::BottomRight => Vec3 {
            x: translation.x - size.x / 2.,
            y: translation.y + size.y / 2.,
            z: translation.z,
        },
        Anchor::Center => *translation,
        Anchor::CenterLeft => Vec3 {
            x: translation.x + size.x / 2.,
            y: translation.y,
            z: translation.z,
        },
        Anchor::CenterRight => Vec3 {
            x: translation.x - size.x / 2.,
            y: translation.y,
            z: translation.z,
        },
        Anchor::Custom(vec) => Vec3 {
            x: translation.x - size.x * vec.x,
            y: translation.y - size.y * vec.y,
            z: translation.z,
        },
        Anchor::TopCenter => Vec3 {
            x: translation.x,
            y: translation.y - size.y / 2.,
            z: translation.z,
        },
        Anchor::TopLeft => Vec3 {
            x: translation.x + size.x / 2.,
            y: translation.y - size.y / 2.,
            z: translation.z,
        },
        Anchor::TopRight => Vec3 {
            x: translation.x - size.x / 2.,
            y: translation.y - size.y / 2.,
            z: translation.z,
        },
    }
}
