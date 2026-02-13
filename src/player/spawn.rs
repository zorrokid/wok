use bevy::{
    asset::AssetServer,
    ecs::system::{Commands, Res},
    math::Vec2,
    sprite::Sprite,
    transform::components::Transform,
};

use crate::{
    level::tile::{TILE_SIZE, TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y},
    player::{Player, SPRITE_HEIGHT, Velocity},
};

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load player sprite
    let player_texture = asset_server.load("player.png");

    // Calculate spawn position: 3 tiles from left, on top of ground (3 tile rows)
    // Ground starts at y=0, is 3 tiles (48 pixels) tall
    // Player sprite center should be at ground + half player height
    let spawn_x = TILEMAP_OFFSET_X + (3.0 * TILE_SIZE);
    let spawn_y = TILEMAP_OFFSET_Y + (3.0 * TILE_SIZE) + SPRITE_HEIGHT / 2.0;

    commands.spawn((
        Player,
        Velocity(Vec2::ZERO),
        Sprite::from_image(player_texture),
        Transform::from_xyz(spawn_x, spawn_y, 10.0),
    ));
}
