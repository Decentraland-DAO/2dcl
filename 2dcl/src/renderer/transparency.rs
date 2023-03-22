use bevy::{prelude::*, sprite::collide_aabb::collide};

use crate::{bundles::get_translation_by_anchor, components};

use super::config::{PLAYER_VISIBILITY_BOX, TRANSPARENCY_VALUE_FOR_HIDING_ELEMENTS, PLAYER_VISIBILITY_BOX_OFFSET};

pub fn update_transparency(mut sprites_query: Query<(&mut Sprite, &components::SpriteRenderer)>) {
    for (mut sprite, sprite_renderer) in sprites_query.iter_mut() {
        if sprite_renderer.is_on_top_of_player || sprite_renderer.is_on_top_of_player_parcel {
            sprite.color.set_a(TRANSPARENCY_VALUE_FOR_HIDING_ELEMENTS);
        } else {
            sprite.color.set_a(sprite_renderer.default_color.a());
        }
    }
}
pub fn check_elements_on_top_of_player(
    player_query: Query<&GlobalTransform, With<components::Player>>,
    images: Res<Assets<Image>>,
    mut sprites_query: Query<
        (
            &GlobalTransform,
            &Sprite,
            &Handle<Image>,
            &mut components::SpriteRenderer,
        ),
        Without<components::Player>,
    >,
) {
    let player_transform = match player_query.get_single() {
        Ok(transform) => transform,
        Err(e) => {
            println!("Player Transform not found in world: {}", e);
            return;
        }
    };

    for (other_transform, other_sprite, other_image, mut other_sprite_renderer) in
        sprites_query.iter_mut()
    {
        let other_size = match images.get(other_image) {
            Some(image) => image.size(),
            None => Vec2::ZERO,
        };

        let other_location = get_translation_by_anchor(
            &other_size,
            &other_transform.translation(),
            &other_sprite.anchor,
        );
        if is_object_covering_player(player_transform.translation(),other_location,other_size)
        {
            other_sprite_renderer.is_on_top_of_player = true;
        } else {
            other_sprite_renderer.is_on_top_of_player = false;
        }
    }
}


fn is_object_covering_player(player_location: Vec3, object_location: Vec3, object_size: Vec2) ->bool
{
  if object_location.z < player_location.z
    {
     return false; 
    }

  //Check top left
  if collide(
      Vec3{
        x: player_location.x + PLAYER_VISIBILITY_BOX_OFFSET.x - PLAYER_VISIBILITY_BOX.x,
        y: player_location.y + PLAYER_VISIBILITY_BOX_OFFSET.y + PLAYER_VISIBILITY_BOX.y,
        z: player_location.z
      },
      Vec2::ONE,
      object_location,
      object_size,
    )
    .is_none()
    {
      return false;
    }

  //Check top right
  if collide(
    Vec3{
      x: player_location.x + PLAYER_VISIBILITY_BOX_OFFSET.x + PLAYER_VISIBILITY_BOX.x,
      y: player_location.y + PLAYER_VISIBILITY_BOX_OFFSET.y + PLAYER_VISIBILITY_BOX.y,
      z: player_location.z
    },
    Vec2::ONE,
    object_location,
    object_size,
  )
  .is_none()
  {
    return false;
  }

  //Check bottom right
  if collide(
    Vec3{
      x: player_location.x + PLAYER_VISIBILITY_BOX_OFFSET.x + PLAYER_VISIBILITY_BOX.x,
      y: player_location.y + PLAYER_VISIBILITY_BOX_OFFSET.y - PLAYER_VISIBILITY_BOX.y,
      z: player_location.z
    },
    Vec2::ONE,
    object_location,
    object_size,
  )
  .is_none()
  {
    return false;
  }

  //Check bottom left
  if collide(
    Vec3{
      x: player_location.x + PLAYER_VISIBILITY_BOX_OFFSET.x - PLAYER_VISIBILITY_BOX.x,
      y: player_location.y + PLAYER_VISIBILITY_BOX_OFFSET.y - PLAYER_VISIBILITY_BOX.y,
      z: player_location.z
    },
    Vec2::ONE,
    object_location,
    object_size,
  )
  .is_none()
  {
    return false;
  }

    true
}
pub fn check_elements_overlapping_parcels(
    player_query: Query<&components::Player>,
    mut sprites_query: Query<&mut components::SpriteRenderer, Without<components::Player>>,
    scenes_query: Query<&components::Scene>,
) {
    let player = match player_query.get_single() {
        Ok(player) => player,
        Err(e) => {
            println!("Player not found in world: {}", e);
            return;
        }
    };

    for mut sprite_renderer in sprites_query.iter_mut() {
        let sprite_is_in_default_parcel =
            is_sprite_renderer_in_default_parcel(&sprite_renderer, &scenes_query);
        'outer: for parcel_overlapping in sprite_renderer.parcels_overlapping.clone() {
            for scene in scenes_query.iter() {
                if scene.parcels.contains(&parcel_overlapping) {
                    if scene.parcels.contains(&player.current_parcel)
                        && (!sprite_is_in_default_parcel || !scene.is_default)
                    {
                        sprite_renderer.is_on_top_of_player_parcel = true;
                        break 'outer;
                    } else {
                        sprite_renderer.is_on_top_of_player_parcel = false;
                        break;
                    }
                }
            }
        }
    }

    fn is_sprite_renderer_in_default_parcel(
        sprite_renderer: &components::SpriteRenderer,
        scenes_query: &Query<&components::Scene>,
    ) -> bool {
        for scene in scenes_query.iter() {
            for parcel in sprite_renderer.parent_parcels.iter() {
                if scene.parcels.contains(parcel) {
                    return scene.is_default;
                }
            }
        }
        false
    }
}
