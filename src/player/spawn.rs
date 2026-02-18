use avian2d::prelude::*;
use bevy::{
    asset::AssetServer,
    ecs::system::{Commands, Res},
    math::{Dir2, Vec2},
    sprite::Sprite,
    transform::components::Transform,
};

use crate::{
    level::tile::{TILE_SIZE, TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y},
    player::{COLLIDER_HALF_LENGTH, COLLIDER_RADIUS, Player, SPRITE_HEIGHT},
};

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_texture = asset_server.load("player.png");

    // Calculate spawn position: 3 tiles from left, on top of 3-row ground.
    // TILEMAP_OFFSET_X/Y are the bottom-left corner of the centered map (-160, -120).
    let spawn_x = TILEMAP_OFFSET_X + (3.0 * TILE_SIZE);
    let spawn_y = TILEMAP_OFFSET_Y + (3.0 * TILE_SIZE) + SPRITE_HEIGHT / 2.0;

    commands.spawn((
        Player,
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
            Collider::capsule(COLLIDER_HALF_LENGTH, COLLIDER_RADIUS - 1.0),
            Vec2::ZERO,
            0.0,
            Dir2::NEG_Y,
        )
        .with_max_distance(10.0),
        ShapeHits::default(),
        Sprite::from_image(player_texture),
        Transform::from_xyz(spawn_x, spawn_y, 10.0),
    ));
}
