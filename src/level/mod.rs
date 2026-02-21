pub mod tile;
pub mod transition;

use avian2d::prelude::{Collider, RigidBody};
use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{AssetServer, Handle},
    ecs::{observer::On, system::{Commands, Res}},
    prelude::IntoScheduleConfigs,
    transform::components::Transform,
};
use bevy_ecs_tiled::prelude::{
    ColliderCreated, TiledEvent, TiledMap, TiledMapAsset, TilemapAnchor,
};

use crate::level::tile::{LEVEL_HEIGHT_IN_TILES, LEVEL_WIDTH_IN_TILES, TILE_SIZE, TILEMAP_OFFSET_X};
use crate::level::transition::{
    CurrentMap, LevelTransitionEvent, apply_transition, debug_trigger_transition,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        // LevelTransitionEvent uses Bevy 0.18's Message system (add_message) rather
        // than the older add_event API. Messages are pull-based: writers queue them
        // and readers consume them in a scheduled system.
        app.add_message::<LevelTransitionEvent>()
            .add_systems(Startup, (setup_tilemap, spawn_level_bounds))
            .add_systems(
                Update,
                (
                    debug_trigger_transition,
                    // apply_transition must run after debug_trigger_transition so it
                    // sees any message written this frame before the reader advances.
                    apply_transition.after(debug_trigger_transition),
                ),
            );
    }
}

/// Spawns invisible static colliders at the left and right edges of the level
/// so the player cannot walk off the map.
pub fn spawn_level_bounds(mut commands: Commands) {
    let level_width_px = LEVEL_WIDTH_IN_TILES as f32 * TILE_SIZE;
    let level_height_px = LEVEL_HEIGHT_IN_TILES as f32 * TILE_SIZE;

    // Left wall at the left edge of the tilemap
    commands.spawn((
        Collider::rectangle(1.0, level_height_px),
        RigidBody::Static,
        Transform::from_xyz(TILEMAP_OFFSET_X, 0.0, 0.0),
    ));

    // Right wall at the right edge of the tilemap
    commands.spawn((
        Collider::rectangle(1.0, level_height_px),
        RigidBody::Static,
        Transform::from_xyz(TILEMAP_OFFSET_X + level_width_px, 0.0, 0.0),
    ));
}

/// Named function for the ColliderCreated observer so it can be reattached
/// to new map entities spawned during transitions (see `apply_transition`).
///
/// TiledPhysicsPlugin creates an Avian2D Collider for each solid tile but
/// leaves the RigidBody type unset. This observer fires once per collider
/// entity and marks it as Static so Avian includes it in collision resolution.
pub fn on_collider_created(ev: On<TiledEvent<ColliderCreated>>, mut commands: Commands) {
    commands.entity(ev.event().origin).insert(RigidBody::Static);
}

pub fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    // TiledMap triggers bevy_ecs_tiled to parse the .tmx file and spawn child
    // entities for each layer and tile. TilemapAnchor::Center positions the
    // tilemap so that its center aligns with the world origin (0, 0).
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map1.tmx");
    let entity = commands
        .spawn((TiledMap(map_handle), TilemapAnchor::Center))
        .observe(on_collider_created)
        .id();

    // CurrentMap is used by apply_transition to despawn the old map hierarchy
    // when switching levels. It must be inserted after spawn so the entity ID
    // is known.
    commands.insert_resource(CurrentMap(entity));
}
