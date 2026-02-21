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

use crate::player::{
    Health, NeedsRespawn, COLLIDER_HALF_LENGTH, COLLIDER_RADIUS, KILL_ZONE_Y, PLAYER_MAX_HP,
    PLAYER_SPAWN_X, PLAYER_SPAWN_Y, Player, SHAPE_CASTER_RADIUS,
};

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        Transform::from_xyz(PLAYER_SPAWN_X, PLAYER_SPAWN_Y, 10.0),
    ));
}

/// Marks the player for respawn if they fall below the kill zone threshold.
///
/// Inserts `NeedsRespawn` rather than resetting position directly so that
/// `respawn_player` (the single respawn authority) handles both fall deaths
/// and HP-zero deaths through the same code path.
pub fn kill_zone(
    mut commands: Commands,
    query: Query<(Entity, &Position), With<Player>>,
) {
    let Ok((entity, position)) = query.single() else {
        return;
    };

    if position.y < KILL_ZONE_Y {
        commands.entity(entity).insert(NeedsRespawn);
    }
}
