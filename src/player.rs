use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    input::ButtonInput,
    prelude::KeyCode,
    sprite::Sprite,
    time::Time,
    transform::components::Transform,
};

use crate::level::{LEVEL_HEIGHT, LEVEL_WIDTH};

// Constants
const PLAYER_SPEED: f32 = 100.0; // pixels per second
const SCREEN_LEFT_BOUND: f32 = -152.0; // -160 (tilemap offset) + 8 (half player)
const SCREEN_RIGHT_BOUND: f32 = 152.0; // 160 (tilemap offset) - 8 (half player)

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

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut transform) = query.single_mut() {
        let mut direction = 0.0;

        // Read keyboard input
        if keyboard.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }

        // Calculate new position
        let movement = direction * PLAYER_SPEED * time.delta_secs();
        transform.translation.x += movement;

        // Clamp to screen boundaries
        transform.translation.x = transform
            .translation
            .x
            .clamp(SCREEN_LEFT_BOUND, SCREEN_RIGHT_BOUND);
    }
}
