use super::scene_loader::{level_changer, parcel_to_world_location};
use super::screen_fade::FadeDirection;
use super::{animations::*, collision::*, screen_fade};
use crate::components::{LevelChange, PlayerInputState};
use crate::renderer::constants::*;
use crate::{components, resources};
use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*, sprite::Anchor};
use bevy_console::ConsoleOpen;
use dcl2d_ecs_v1::collision_type::CollisionType;
use dcl_common::Parcel;

pub struct PlayerPlugin;

#[derive(Debug)]
pub struct LevelChangeStackData {
    location: Vec3,
    level_id: usize,
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (player_interact, player_movement))
            .add_systems(Update, player_input_state_update.before(level_changer));
    }
}

pub fn make_player_animator(
    assets: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) -> Result<components::Animator, String> {
    let mut player_animator_path = std::env::current_exe().unwrap_or_default();
    player_animator_path.pop();
    player_animator_path.push("assets");
    player_animator_path.push("player.json");

    get_animator(player_animator_path, assets, texture_atlases)
}

pub fn update_player_scale(
    new_scale: f32,
    camera_size: f32,
    player_transform: &mut Transform,
    interact_icon_transform: &mut Transform,
    orthografic_projection: &mut OrthographicProjection,
    animator: &components::Animator,
) {
    player_transform.scale = (Vec2::ONE * new_scale * animator.scale / 2.).extend(1.);
    interact_icon_transform.translation =
        Vec3::new(0.0, INTERACT_ICON_HEIGHT * 2. / new_scale, 0.0);
    interact_icon_transform.scale = (Vec2::ONE * 2. / new_scale).extend(1.);
    update_camera_size(camera_size, new_scale, orthografic_projection);
}

pub fn update_camera_size(
    new_camera_size: f32,
    player_scale: f32,
    orthografic_projection: &mut OrthographicProjection,
) {
    orthografic_projection.scale = new_camera_size * 2. / player_scale;
}
fn spawn_player(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    config: Res<resources::Config>,
) {
    let player_animator = make_player_animator(&assets, &mut texture_atlases);

    if let Err(e) = player_animator {
        println!("{}", e);
        return;
    }

    let mut interact_animator_path = std::env::current_exe().unwrap_or_default();
    interact_animator_path.pop();
    interact_animator_path.push("assets");
    interact_animator_path.push("interact.json");

    let interact_animator = get_animator(interact_animator_path, &assets, &mut texture_atlases);

    if let Err(e) = interact_animator {
        println!("{}", e);
        return;
    }

    let interact_animator = interact_animator.unwrap();
    let player_animator = player_animator.unwrap();
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.anchor = Anchor::BottomCenter;

    let translation = parcel_to_world_location(Parcel(
        config.world.starting_parcel_x,
        config.world.starting_parcel_y,
    ));

    //Spawning Entity
    let player = commands
        .spawn(SpriteSheetBundle {
            sprite: sprite.clone(),
            texture_atlas: player_animator.atlas.clone(),
            transform: Transform {
                scale: (Vec2::ONE * config.player.scale * player_animator.scale / 2.).extend(1.),
                translation,
                ..default()
            },
            ..default()
        })
        .insert(player_animator)
        .insert(Name::new("Player"))
        .insert(components::Player {
            current_level: 0,
            current_parcel: Parcel(0, 0),
            level_change_stack: vec![],
            input_state: PlayerInputState::Normal,
        })
        .id();

    let interact_icon = commands
        .spawn(SpriteSheetBundle {
            sprite,
            texture_atlas: interact_animator.atlas.clone(),
            transform: Transform::from_translation(Vec3::new(
                0.0,
                INTERACT_ICON_HEIGHT * 2. / config.player.scale,
                0.0,
            ))
            .with_scale((Vec2::ONE * 2. / config.player.scale).extend(1.)),
            ..default()
        })
        .insert(interact_animator)
        .insert(Name::new("Interact_icon"))
        .insert(components::InteractIcon)
        .id();

    let clear_color = ClearColorConfig::Custom(Color::BLACK);
    let mut camera_bundle = Camera2dBundle::new_with_far(10000.0);
    camera_bundle.camera_2d.clear_color = clear_color;
    camera_bundle.transform = Transform::from_translation(Vec3 {
        x: 0.0,
        y: config.player.collider_size_y,
        z: 5000.0,
    });

    camera_bundle.projection.scale = config.world.camera_size * 2. / config.player.scale;
    let camera_entity = commands.spawn(camera_bundle).id();

    commands.entity(player).add_child(camera_entity);
    commands.entity(player).add_child(interact_icon);
}

fn player_movement(
    mut player_query: Query<(
        &mut components::Player,
        &mut Transform,
        &mut components::Animator,
        &mut TextureAtlasSprite,
    )>,
    box_collision_query: Query<(Entity, &GlobalTransform, &components::BoxCollider)>,
    entities_with_level_change: Query<(Entity, &components::LevelChange)>,
    keyboard: Res<Input<KeyCode>>,
    collision_map: Res<resources::CollisionMap>,
    scenes_query: Query<&components::Scene>,
    time: Res<Time>,
    console: Res<ConsoleOpen>,
    config: Res<resources::Config>,
) {
    if console.open {
        return;
    }

    let result = player_query.get_single_mut();

    if let Err(e) = result {
        println!("{}", e);
        return;
    }

    let (mut player, mut transform, mut animator, mut texture_atlas) = result.unwrap();

    let mut animation_state = "Idle";
    let mut movement_input = get_movment_axis_input(&keyboard);

    movement_input = movement_input.normalize();
    movement_input = movement_input * config.player.speed * time.delta_seconds();

    let mut walking = false;
    if movement_input.length() > 0f32 {
        walking = true;
    }

    if walking {
        let runing_up = movement_input.y > 0.0;
        let running_diagonal = movement_input.x.abs() > 0.25;

        animation_state = match runing_up {
            true => match running_diagonal {
                true => "RunUpSide",
                false => "RunUp",
            },
            false => match running_diagonal {
                true => "RunDownSide",
                false => "RunDown",
            },
        };

        let mut target = transform.translation + Vec3::new(movement_input.x, 0.0, 0.0);

        if !check_player_collision(
            player.as_mut(),
            &transform.translation,
            &target,
            &box_collision_query,
            &entities_with_level_change,
            &scenes_query,
            &collision_map,
            &config,
        ) {
            transform.translation = target;
        }

        target = transform.translation + Vec3::new(0.0, movement_input.y, movement_input.y * -1.0);

        if !check_player_collision(
            player.as_mut(),
            &transform.translation,
            &target,
            &box_collision_query,
            &entities_with_level_change,
            &scenes_query,
            &collision_map,
            &config,
        ) {
            transform.translation = target;
        }
    }

    if movement_input.x > 0.0 {
        texture_atlas.flip_x = true;
    } else if movement_input.x < 0.0 {
        texture_atlas.flip_x = false;
    }
    change_animator_state(animator.as_mut(), texture_atlas.as_mut(), animation_state);
}

fn player_input_state_update(
    mut player_query: Query<(&mut components::Player, &mut Transform)>,
    mut fade_event_reader: EventReader<screen_fade::FadeFinished>,
    mut fade: ResMut<screen_fade::Fade>,
) {
    let result = player_query.get_single_mut();

    if let Err(e) = result {
        println!("{}", e);
        return;
    }

    let (mut player, mut transform) = result.unwrap();

    match player.input_state.clone() {
        PlayerInputState::Normal => return,
        PlayerInputState::LoadingLevel(level_to_load) => {
            if fade_out_finished(&mut fade_event_reader) {
                player.input_state = PlayerInputState::Normal;
                change_level(&mut player, &mut transform, &level_to_load, &mut fade);
            }
        }
        PlayerInputState::ExitingLevel => {
            if fade_out_finished(&mut fade_event_reader) {
                player.input_state = PlayerInputState::Normal;
                exit_level(&mut player, &mut transform, &mut fade);
            }
        }
    }

    fn fade_out_finished(fade_event_reader: &mut EventReader<screen_fade::FadeFinished>) -> bool {
        for new_event in fade_event_reader.iter() {
            if new_event.0 == FadeDirection::FadeOut {
                return true;
            }
        }

        false
    }
}
fn player_interact(
    mut player_query: Query<(&mut components::Player, &mut Transform)>,
    mut iteract_query: Query<(
        &components::InteractIcon,
        &mut components::Animator,
        &mut TextureAtlasSprite,
    )>,
    box_collision_query: Query<(Entity, &GlobalTransform, &components::BoxCollider)>,
    entities_with_level_change: Query<(Entity, &components::LevelChange)>,
    keyboard: Res<Input<KeyCode>>,
    collision_map: Res<resources::CollisionMap>,
    scenes_query: Query<&components::Scene>,
    mut fade: ResMut<screen_fade::Fade>,
    console: Res<ConsoleOpen>,
    config: Res<resources::Config>,
) {
    if console.open {
        return;
    }
    let result = player_query.get_single_mut();

    if let Err(e) = result {
        println!("{}", e);
        return;
    }

    let (mut player, transform) = result.unwrap();

    let collisions = get_collisions(
        &player.current_parcel,
        player.current_level,
        &transform.translation,
        &Vec2 {
            x: config.player.collider_size_x,
            y: config.player.collider_size_y,
        },
        &box_collision_query,
        &entities_with_level_change,
        &scenes_query,
        &collision_map,
    );

    if keyboard.just_pressed(KeyCode::E) {
        if let Some(level_change) = is_in_level_change_trigger(&collisions) {
            player.input_state = PlayerInputState::LoadingLevel(level_change);
            fade.direction = FadeDirection::FadeOut;
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) && !player.level_change_stack.is_empty() {
        player.input_state = PlayerInputState::ExitingLevel;
        fade.direction = FadeDirection::FadeOut;
    }

    let result = iteract_query.get_single_mut();

    match result {
        Ok((_, mut animator, mut texture_atlas)) => {
            update_interact_icon_visibility(&collisions, animator.as_mut(), texture_atlas.as_mut());
        }

        Err(e) => {
            println!("{}", e);
        }
    }
}

fn update_interact_icon_visibility(
    collisions: &Vec<CollisionResult>,
    interact_icon_animator: &mut components::Animator,
    interact_sprite_atlas: &mut TextureAtlasSprite,
) {
    let mut player_is_in_trigger = false;
    for collision in collisions {
        if collision.collision_type == CollisionType::Trigger {
            player_is_in_trigger = true;
            break;
        }
    }

    if player_is_in_trigger {
        if interact_icon_animator.current_animation.name == "hidden"
            || interact_icon_animator.current_animation.name == "fade_out"
        {
            change_animator_state(interact_icon_animator, interact_sprite_atlas, "fade_in");
            queue_animation(interact_icon_animator, "idle")
        }
    } else if interact_icon_animator.current_animation.name == "idle"
        || interact_icon_animator.current_animation.name == "fade_in"
    {
        change_animator_state(interact_icon_animator, interact_sprite_atlas, "fade_out");
        queue_animation(interact_icon_animator, "hidden")
    }
}
fn change_level(
    player: &mut components::Player,
    player_transform: &mut Transform,
    level_change: &components::LevelChange,
    fade: &mut ResMut<screen_fade::Fade>,
) {
    let level_change_stack_data = LevelChangeStackData {
        level_id: player.current_level,
        location: player_transform.translation,
    };

    player_transform.translation = level_change
        .spawn_point
        .extend(level_change.spawn_point.y * -1f32);
    player.current_level = level_change.level;
    player.level_change_stack.push(level_change_stack_data);

    if level_change.level == 0 {
        player.level_change_stack.clear();
    }

    fade.direction = FadeDirection::FadeIn;
}

fn is_in_level_change_trigger(collisions: &Vec<CollisionResult>) -> Option<LevelChange> {
    for collision in collisions {
        if collision.collision_type == CollisionType::Trigger && collision.level_change.is_some() {
            let level_change = collision.level_change.clone().unwrap();
            return Some(level_change);
        }
    }
    None
}

fn exit_level(
    player: &mut components::Player,
    transform: &mut Transform,
    fade: &mut ResMut<screen_fade::Fade>,
) {
    if let Some(data) = player.level_change_stack.pop() {
        transform.translation = data.location;
        player.current_level = data.level_id;
    }
    fade.direction = FadeDirection::FadeIn;
}

fn check_player_collision(
    player: &mut components::Player,
    current_location: &Vec3,
    target_location: &Vec3,
    box_collision_query: &Query<(Entity, &GlobalTransform, &components::BoxCollider)>,
    entities_with_level_change: &Query<(Entity, &components::LevelChange)>,
    scenes_query: &Query<&components::Scene>,
    collision_map: &resources::CollisionMap,
    config: &resources::Config,
) -> bool {
    let collisions = get_collisions(
        &player.current_parcel,
        player.current_level,
        current_location,
        &Vec2 {
            x: config.player.collider_size_x,
            y: config.player.collider_size_y,
        },
        box_collision_query,
        entities_with_level_change,
        scenes_query,
        collision_map,
    );

    for collision in collisions {
        if collision.hit && collision.collision_type == CollisionType::Solid {
            return false;
        }
    }

    let collisions = get_collisions(
        &player.current_parcel,
        player.current_level,
        target_location,
        &Vec2 {
            x: config.player.collider_size_x,
            y: config.player.collider_size_y,
        },
        box_collision_query,
        entities_with_level_change,
        scenes_query,
        collision_map,
    );

    for collision in collisions {
        if collision.hit && collision.collision_type == CollisionType::Solid {
            return true;
        }
    }
    false
}

fn get_movment_axis_input(keyboard: &Res<Input<KeyCode>>) -> Vec3 {
    let mut movement_input = Vec3::default();

    if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
        movement_input.y += 1f32;
    }

    if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
        movement_input.y -= 1f32;
    }

    if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
        movement_input.x += 1f32;
    }

    if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
        movement_input.x -= 1f32;
    }

    movement_input
}
