use avian2d::prelude::*;
use bevy::{
    asset::AssetServer,
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res},
    },
    math::{Dir2, Vec2},
    sprite::Sprite,
    transform::components::Transform,
};

use crate::level::tile::MapDimensions;
use crate::player::{
    Health, NeedsRespawn, COLLIDER_HALF_LENGTH, COLLIDER_RADIUS, PLAYER_MAX_HP,
    SPRITE_HEIGHT, Player, SHAPE_CASTER_RADIUS,
};

/// Returns the initial world-space spawn position for the player.
///
/// Placed 3 tiles from the bottom-left corner of the map, centred on the
/// sprite height. Called from both `spawn_player_at` and `respawn_player`.
pub fn initial_spawn_pos(dims: &MapDimensions) -> Vec2 {
    dims.tile_to_world(3.0, 3.0) + Vec2::new(0.0, SPRITE_HEIGHT / 2.0)
}

/// Spawns the player entity at the initial map spawn position.
///
/// Called from `level::on_map_created` on first map load only (guarded by a
/// `Query<(), With<Player>>` check in the observer).
pub fn spawn_player_at(dims: &MapDimensions, commands: &mut Commands, asset_server: &AssetServer) {
    let spawn_pos = initial_spawn_pos(dims);
    let player_texture = asset_server.load("player.png");

    commands.spawn((
        Player,
        Health::full(PLAYER_MAX_HP),
        RigidBody::Dynamic,
        // Capsule collider: rounded ends prevent snagging on tile corners when
        // walking off platform edges. Total height = 2*half_length + 2*radius = 14px.
        Collider::capsule(COLLIDER_HALF_LENGTH, COLLIDER_RADIUS),
        // Prevent Avian from rotating the player when it hits walls or edges.
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,
        Friction::new(0.0),    // No friction: prevent the player from sticking to walls
        Restitution::new(0.0), // No bounce on landing
        // ShapeCaster for ground detection. Uses a slightly narrower capsule than
        // the main collider to avoid false positives at tile edges.
        ShapeCaster::new(
            Collider::capsule(COLLIDER_HALF_LENGTH, SHAPE_CASTER_RADIUS),
            Vec2::ZERO,
            0.0,
            Dir2::NEG_Y,
        )
        .with_max_distance(10.0),
        ShapeHits::default(),
        Sprite::from_image(player_texture),
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 10.0),
    ));
}

/// Marks the player for respawn if they fall below the kill zone threshold.
///
/// The kill zone is 64 px below the map floor so minor physics clipping
/// doesn't trigger a spurious respawn. Inserts `NeedsRespawn` rather than
/// resetting position directly so that `respawn_player` (the single respawn
/// authority) handles both fall deaths and HP-zero deaths through the same
/// code path.
pub fn kill_zone(
    mut commands: Commands,
    map_dims: Res<MapDimensions>,
    query: Query<(Entity, &Position), With<Player>>,
) {
    let Ok((entity, position)) = query.single() else {
        return;
    };

    let kill_zone_y = map_dims.offset_y - 64.0;
    if position.y < kill_zone_y {
        commands.entity(entity).insert(NeedsRespawn);
    }
}
