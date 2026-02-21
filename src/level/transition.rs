use avian2d::prelude::{Collider, CollisionEventsEnabled, CollisionStart, LinearVelocity, Position, RigidBody, Sensor};
use bevy::{
    ecs::{
        entity::Entity,
        message::{Message, MessageReader, MessageWriter},
        observer::On,
        query::{Has, With},
        system::{Commands, Query, Res, ResMut},
    },
    asset::{AssetServer, Handle},
    math::Vec2,
    prelude::{Component, Reflect, ReflectComponent, Resource, Transform, info},
    reflect::std_traits::ReflectDefault,
    time::{Time, Timer, TimerMode},
};
use bevy_ecs_tiled::prelude::{
    TiledEvent, TiledFilter, TiledMap, TiledMapAsset, TiledObject, ObjectCreated, TilemapAnchor,
    TiledPhysicsAvianBackend, TiledPhysicsSettings,
};

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

/// Tiled class property component for transition zone objects.
///
/// bevy_ecs_tiled deserializes this from a Custom Class property named
/// `wok::level::transition::LevelTransition` on rectangle objects in the
/// map's object layer. The `Reflect` derive and type registration allow
/// bevy_ecs_tiled to insert this component automatically — no observer code
/// needed to read property values.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct LevelTransition {
    /// Asset path of the target map, e.g. `"map2.tmx"`.
    pub target_map: String,
    /// Tile column for the player spawn position in the target map.
    pub spawn_tile_x: f32,
    /// Tile row for the player spawn position in the target map.
    pub spawn_tile_y: f32,
}

/// How long (seconds) to block re-triggering after a transition fires.
///
/// Chosen to be longer than one physics tick but short enough that the
/// player cannot be prevented from immediately walking back through a zone.
/// The root cause is that `Position` writes take effect one physics tick
/// after `apply_transition` runs, so the player may still geometrically
/// overlap a zone on the new map for one frame and fire another `CollisionStart`.
const TRANSITION_COOLDOWN_SECS: f32 = 0.25;

/// Prevents repeated map transitions immediately after one fires.
///
/// The timer starts already finished so the very first contact triggers
/// a transition normally. After that, further triggers are blocked for
/// `TRANSITION_COOLDOWN_SECS` seconds.
#[derive(Resource)]
pub struct TransitionCooldown(pub Timer);

impl Default for TransitionCooldown {
    fn default() -> Self {
        let mut t = Timer::from_seconds(TRANSITION_COOLDOWN_SECS, TimerMode::Once);
        t.tick(t.duration()); // start finished so first transition fires immediately
        Self(t)
    }
}

/// Carries destination data for a map transition.
///
/// `spawn_tile_x` and `spawn_tile_y` are in tile coordinates measured from
/// the bottom-left of the map. `apply_transition` converts them to world
/// space before writing the player's `Position`.
///
/// Uses `Message` (not the legacy `Event`) so it can be queued via
/// `MessageWriter` and consumed in a scheduled system via `MessageReader`.
#[derive(Message)]
pub struct LevelTransitionEvent {
    pub target_map: String,
    pub spawn_tile_x: f32,
    pub spawn_tile_y: f32,
}

/// Global observer: fires for every `TiledEvent<ObjectCreated>` in the app.
///
/// Registered with `app.add_observer` (not per-entity `.observe()`) so it
/// automatically covers both the initial map load and every subsequent map
/// loaded via `apply_transition` — no need to re-attach per entity.
///
/// If the spawned object entity already has a `LevelTransition` component
/// (inserted by bevy_ecs_tiled from the Tiled class property before this
/// observer fires), adds Avian2D sensor components to make it a trigger zone.
/// Non-transition objects are ignored via the early return on `has_transition`.
pub fn setup_transition_colliders(
    ev: On<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    query: Query<(&TiledObject, Has<LevelTransition>, &Transform)>,
) {
    let entity = ev.event().origin;
    let Ok((obj, has_transition, transform)) = query.get(entity) else {
        return;
    };

    if !has_transition { return; }

    let TiledObject::Rectangle { width, height } = obj else { return };
    let (width, height) = (*width, *height);

    // bevy_ecs_tiled places the Transform at the top-left corner of the Tiled
    // rectangle. Avian2D centers the Collider on the Transform. To make the
    // collider cover the actual rectangle area, shift the Transform to the
    // rectangle's center: +width/2 in X, -height/2 in Y (world Y is up).
    let center = Transform::from_xyz(
        transform.translation.x + width / 2.0,
        transform.translation.y - height / 2.0,
        transform.translation.z,
    );

    info!("Adding transition collider {}x{} at ({}, {})",
        width, height, center.translation.x, center.translation.y);

    commands.entity(entity).insert((
        center,
        RigidBody::Static,
        Sensor,
        Collider::rectangle(width, height),
        CollisionEventsEnabled,
    ));
}

/// Reads `CollisionStart` messages each frame. When the player contacts a
/// `LevelTransition` entity, sends a `LevelTransitionEvent` for
/// `apply_transition` to handle.
///
/// Uses `CollisionStart` (not the `Collisions` system param) because a
/// one-shot trigger per contact begin is correct here — re-firing every
/// frame while inside the zone would cause rapid repeated transitions.
pub fn trigger_transition(
    mut collision_reader: MessageReader<CollisionStart>,
    player_q: Query<Entity, With<Player>>,
    transition_q: Query<&LevelTransition>,
    mut writer: MessageWriter<LevelTransitionEvent>,
    mut cooldown: ResMut<TransitionCooldown>,
    time: Res<Time>,
) {
    cooldown.0.tick(time.delta());
    if !cooldown.0.is_finished() {
        return;
    }

    let Ok(player) = player_q.single() else { return };

    for event in collision_reader.read() {
        let other = if event.collider1 == player {
            event.collider2
        } else if event.collider2 == player {
            event.collider1
        } else {
            continue
        };

        if let Ok(t) = transition_q.get(other) {
            info!("Player touched transition zone → {}", t.target_map);
            writer.write(LevelTransitionEvent {
                target_map: t.target_map.clone(),
                spawn_tile_x: t.spawn_tile_x,
                spawn_tile_y: t.spawn_tile_y,
            });
            cooldown.0.reset();
            break;
        }
    }
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
    commands.entity(current_map.0).despawn_children();
    commands.entity(current_map.0).despawn();

    let map_handle: Handle<TiledMapAsset> = asset_server.load(ev.target_map.clone());

    // Spawn the new map and reattach both observers so tile colliders are
    // marked as RigidBody::Static and transition zones get their sensors.
    let new_entity = commands
        .spawn((
            TiledMap(map_handle),
            TilemapAnchor::Center,
            TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
                objects_layer_filter: TiledFilter::None,
                ..Default::default()
            },
        ))
        .observe(on_collider_created)
        .id();

    commands.insert_resource(CurrentMap(new_entity));

    if let Ok((mut pos, mut vel)) = player_q.single_mut() {
        // TILEMAP_OFFSET_X/Y are compile-time constants derived from
        // LEVEL_WIDTH_IN_TILES / LEVEL_HEIGHT_IN_TILES. This formula is correct
        // only while all maps share those dimensions. If maps ever differ in size,
        // spawn offsets must become per-map metadata stored in LevelTransitionEvent.
        let world_x = TILEMAP_OFFSET_X + ev.spawn_tile_x * TILE_SIZE + TILE_SIZE / 2.0;
        let world_y = TILEMAP_OFFSET_Y + ev.spawn_tile_y * TILE_SIZE + TILE_SIZE / 2.0;
        pos.0 = Vec2::new(world_x, world_y);
        // Clear momentum so the player does not arrive at speed.
        vel.0 = Vec2::ZERO;
    }
}
