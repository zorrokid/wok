use avian2d::prelude::{LinearVelocity, Position};
use bevy::{
    ecs::{
        entity::Entity,
        message::{Message, MessageReader, MessageWriter},
        query::With,
        system::{Commands, Query, Res},
    },
    asset::{AssetServer, Handle},
    input::{ButtonInput, keyboard::KeyCode},
    math::Vec2,
    prelude::Resource,
};
use bevy_ecs_tiled::prelude::{TiledMap, TiledMapAsset, TilemapAnchor};

use crate::level::{
    on_collider_created,
    tile::{TILE_SIZE, TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y},
};
use crate::player::Player;

/// Tracks the entity that owns the currently loaded TiledMap.
///
/// Inserted by `setup_tilemap` and updated after each transition so that
/// `apply_transition` knows which entity to despawn when loading a new map.
#[derive(Resource)]
pub struct CurrentMap(pub Entity);

/// Carries destination data for a map transition.
///
/// `spawn_tile_x` and `spawn_tile_y` are in tile coordinates measured from
/// the bottom-left of the map. `apply_transition` converts them to world
/// space before writing the player's `Position`.
///
/// Uses `Message` (not `Event`) so it can be queued via `MessageWriter` and
/// consumed in a scheduled system via `MessageReader` — the Bevy 0.18 pattern
/// for pull-based, frame-delayed event handling.
#[derive(Message)]
pub struct LevelTransitionEvent {
    pub target_map: String,
    pub spawn_tile_x: f32,
    pub spawn_tile_y: f32,
}

/// Despawns the current map, loads the target map, and repositions the player.
///
/// Writing `Position` (not `Transform`) is intentional: Avian2D owns the
/// `Transform` for dynamic rigid bodies and propagates `Position` to it each
/// physics tick. Writing `Transform` directly would be overwritten immediately.
pub fn apply_transition(
    mut messages: MessageReader<LevelTransitionEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_map: Res<CurrentMap>,
    mut player_q: Query<(&mut Position, &mut LinearVelocity), With<Player>>,
) {
    let Some(ev) = messages.read().next() else {
        return;
    };

    // Remove the old map and all its children (tile entities, colliders, etc.)
    commands
        .entity(current_map.0)
        .despawn_related::<bevy::prelude::Children>();
    commands.entity(current_map.0).despawn();

    let map_handle: Handle<TiledMapAsset> = asset_server.load(ev.target_map.clone());

    // Spawn the new map and reattach the collider observer so tile colliders
    // are marked as RigidBody::Static just as they were for the initial map.
    let new_entity = commands
        .spawn((TiledMap(map_handle), TilemapAnchor::Center))
        .observe(on_collider_created)
        .id();

    commands.insert_resource(CurrentMap(new_entity));

    if let Ok((mut pos, mut vel)) = player_q.single_mut() {
        let world_x = TILEMAP_OFFSET_X + ev.spawn_tile_x * TILE_SIZE + TILE_SIZE / 2.0;
        let world_y = TILEMAP_OFFSET_Y + ev.spawn_tile_y * TILE_SIZE + TILE_SIZE / 2.0;
        pos.0 = Vec2::new(world_x, world_y);
        // Clear momentum so the player does not arrive at speed.
        vel.0 = Vec2::ZERO;
    }
}

/// Temporary debug system: press T to go to map2, R to return to map1.
///
/// This system exists only to test the transition plumbing without physics-
/// based trigger zones. It is removed in spec 017 when transition zones
/// replace it.
pub fn debug_trigger_transition(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: MessageWriter<LevelTransitionEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        writer.write(LevelTransitionEvent {
            target_map: "map2.tmx".to_string(),
            spawn_tile_x: 2.0,
            spawn_tile_y: 4.0,
        });
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        writer.write(LevelTransitionEvent {
            target_map: "map1.tmx".to_string(),
            spawn_tile_x: 28.0,
            spawn_tile_y: 4.0,
        });
    }
}
