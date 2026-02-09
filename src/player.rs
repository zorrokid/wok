use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        system::{Commands, Res},
    },
    sprite::Sprite,
    transform::components::Transform,
};

use crate::level::{LEVEL_HEIGHT, LEVEL_WIDTH};

// Player marker component
#[derive(Component)]
pub struct Player;

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load player sprite
    let player_texture = asset_server.load("player.png");

    // Calculate spawn position: 3 tiles from left, on top of ground (3 tile rows)
    // Ground starts at y=0, is 3 tiles (48 pixels) tall
    // Player sprite center should be at ground + half player height
    let spawn_x = -(LEVEL_WIDTH as f32 * 16.0) / 2.0 + (3.0 * 16.0);
    let spawn_y = -(LEVEL_HEIGHT as f32 * 16.0) / 2.0 + (3.0 * 16.0) + 8.0;

    commands.spawn((
        Player,
        Sprite::from_image(player_texture),
        Transform::from_xyz(spawn_x, spawn_y, 10.0),
    ));
}
