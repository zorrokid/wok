use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res},
    },
    input::ButtonInput,
    math::Vec2,
    prelude::KeyCode,
    sprite::Sprite,
    time::Time,
    transform::components::Transform,
};

use crate::level::{LEVEL_HEIGHT, LEVEL_WIDTH};

// Constants
const PLAYER_MAX_SPEED: f32 = 100.0; // pixels per second
const PLAYER_ACCELERATION: f32 = 800.0; // pixels per second²
const PLAYER_DECELERATION: f32 = 1200.0; // pixels per second²

// Player marker component
#[derive(Component)]
pub struct Player;

// Velocity component for physics-based movement
#[derive(Component)]
pub struct Velocity(pub Vec2);

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
        Velocity(Vec2::ZERO),
        Sprite::from_image(player_texture),
        Transform::from_xyz(spawn_x, spawn_y, 10.0),
    ));
}

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((mut transform, mut velocity)) = query.single_mut() {
        let delta = time.delta_secs();

        // Determine target velocity based on input
        let mut target_velocity_x = 0.0;
        if keyboard.pressed(KeyCode::ArrowLeft) {
            target_velocity_x -= PLAYER_MAX_SPEED;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            target_velocity_x += PLAYER_MAX_SPEED;
        }

        // Apply acceleration or deceleration
        if target_velocity_x != 0.0 {
            // Accelerate toward target
            let accel_direction = (target_velocity_x - velocity.0.x).signum();
            velocity.0.x += accel_direction * PLAYER_ACCELERATION * delta;

            // Clamp to target (don't overshoot)
            if accel_direction > 0.0 {
                velocity.0.x = velocity.0.x.min(target_velocity_x);
            } else {
                velocity.0.x = velocity.0.x.max(target_velocity_x);
            }
        } else {
            // Decelerate to zero
            if velocity.0.x.abs() > 0.0 {
                let decel_amount = PLAYER_DECELERATION * delta;

                if velocity.0.x > 0.0 {
                    velocity.0.x = (velocity.0.x - decel_amount).max(0.0);
                } else if velocity.0.x < 0.0 {
                    velocity.0.x = (velocity.0.x + decel_amount).min(0.0);
                }
            }
        }

        // Clamp velocity to max speed
        velocity.0.x = velocity.0.x.clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);

        // Apply velocity to position
        transform.translation.x += velocity.0.x * delta;
    }
}
