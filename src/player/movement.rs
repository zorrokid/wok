use avian2d::prelude::{LinearVelocity, Position, ShapeHits};
use bevy::{
    ecs::{
        entity::Entity,
        query::{Has, With},
        system::{Commands, Query, Res},
    },
    input::ButtonInput,
    prelude::KeyCode,
    time::Time,
};

use crate::{
    level::tile::{TILE_SIZE, TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y},
    player::{
        Grounded, JUMP_VELOCITY, PLAYER_ACCELERATION, PLAYER_DECELERATION, PLAYER_MAX_SPEED,
        Player, SPRITE_HEIGHT,
    },
};

/// Updates the `Grounded` marker based on ShapeCaster results.
///
/// ShapeCaster populates `ShapeHits` each physics step. Any hit means the player
/// is touching the ground. Using a marker component (rather than a bool field)
/// lets other systems use `Has<Grounded>` in their query without a query join.
pub fn update_grounded(
    mut commands: Commands,
    query: Query<(Entity, &ShapeHits), With<Player>>,
) {
    for (entity, hits) in &query {
        if hits.is_empty() {
            commands.entity(entity).remove::<Grounded>();
        } else {
            commands.entity(entity).insert(Grounded);
        }
    }
}

/// Handles player input and writes to `LinearVelocity`.
///
/// Avian2D owns the player's `Transform` — we only set `LinearVelocity.x` from
/// input and let the physics solver integrate position each step. Gravity is
/// applied by Avian automatically; we never overwrite `velocity.y` except to
/// set a jump impulse.
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut LinearVelocity, Has<Grounded>), With<Player>>,
) {
    let Ok((mut velocity, is_grounded)) = query.single_mut() else {
        return;
    };

    let is_left = keyboard.pressed(KeyCode::ArrowLeft);
    let is_right = keyboard.pressed(KeyCode::ArrowRight);
    let is_jump = keyboard.just_pressed(KeyCode::KeyZ);

    let target_x = get_target_velocity_x(is_left, is_right);
    velocity.x = apply_horizontal_acceleration(
        velocity.x,
        target_x,
        time.delta_secs(),
        PLAYER_ACCELERATION,
        PLAYER_DECELERATION,
        PLAYER_MAX_SPEED,
    );

    if is_jump && is_grounded {
        velocity.y = JUMP_VELOCITY;
    }
}

/// Determines the target horizontal velocity from directional input.
/// Left takes priority when both directions are pressed simultaneously.
fn get_target_velocity_x(is_left: bool, is_right: bool) -> f32 {
    match (is_left, is_right) {
        (true, _) => -PLAYER_MAX_SPEED,
        (_, true) => PLAYER_MAX_SPEED,
        _ => 0.0,
    }
}

/// Applies acceleration or deceleration toward the target horizontal velocity.
///
/// Accelerates without overshooting the target. When target is zero, decelerates
/// to zero without oscillation. Enforces the max speed cap in both directions.
pub fn apply_horizontal_acceleration(
    velocity_x: f32,
    target_velocity_x: f32,
    delta: f32,
    player_acceleration: f32,
    player_deceleration: f32,
    player_max_speed: f32,
) -> f32 {
    let velocity_x = if target_velocity_x != 0.0 {
        let accel_direction = (target_velocity_x - velocity_x).signum();
        let new_velocity_x = velocity_x + accel_direction * player_acceleration * delta;
        // Clamp to target to prevent overshooting
        if accel_direction > 0.0 {
            new_velocity_x.min(target_velocity_x)
        } else {
            new_velocity_x.max(target_velocity_x)
        }
    } else {
        // Decelerate to zero without overshooting
        let decel_amount = player_deceleration * delta;
        if velocity_x > 0.0 {
            (velocity_x - decel_amount).max(0.0)
        } else if velocity_x < 0.0 {
            (velocity_x + decel_amount).min(0.0)
        } else {
            velocity_x
        }
    };
    velocity_x.clamp(-player_max_speed, player_max_speed)
}

/// Teleports the player back to spawn if they fall below the level floor.
///
/// The threshold is 64px below `TILEMAP_OFFSET_Y` so minor physics clipping
/// doesn't trigger it. Uses `Position` (not `Transform`) because Avian2D owns
/// the transform for dynamic bodies; writing to `Position` directly is safe at
/// any point in the Update schedule.
pub fn kill_zone(mut query: Query<(&mut Position, &mut LinearVelocity), With<Player>>) {
    const KILL_ZONE_Y: f32 = TILEMAP_OFFSET_Y - 64.0;
    const SPAWN_X: f32 = TILEMAP_OFFSET_X + 3.0 * TILE_SIZE;
    const SPAWN_Y: f32 = TILEMAP_OFFSET_Y + 3.0 * TILE_SIZE + SPRITE_HEIGHT / 2.0;

    let Ok((mut position, mut velocity)) = query.single_mut() else {
        return;
    };

    if position.y < KILL_ZONE_Y {
        *position = Position::from_xy(SPAWN_X, SPAWN_Y);
        *velocity = LinearVelocity::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_velocity_right() {
        assert_eq!(get_target_velocity_x(false, true), PLAYER_MAX_SPEED);
    }

    #[test]
    fn test_target_velocity_left() {
        assert_eq!(get_target_velocity_x(true, false), -PLAYER_MAX_SPEED);
    }

    #[test]
    fn test_target_velocity_left_priority_over_right() {
        assert_eq!(get_target_velocity_x(true, true), -PLAYER_MAX_SPEED);
    }

    #[test]
    fn test_target_velocity_no_input() {
        assert_eq!(get_target_velocity_x(false, false), 0.0);
    }

    #[test]
    fn test_acceleration_from_rest_rightward() {
        let result = apply_horizontal_acceleration(
            0.0,
            PLAYER_MAX_SPEED,
            0.016,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        assert!(result > 0.0);
        assert!(result <= PLAYER_MAX_SPEED);
    }

    #[test]
    fn test_acceleration_from_rest_leftward() {
        let result = apply_horizontal_acceleration(
            0.0,
            -PLAYER_MAX_SPEED,
            0.016,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        assert!(result < 0.0);
        assert!(result >= -PLAYER_MAX_SPEED);
    }

    #[test]
    fn test_symmetric_acceleration() {
        let right = apply_horizontal_acceleration(
            0.0,
            PLAYER_MAX_SPEED,
            0.016,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        let left = apply_horizontal_acceleration(
            0.0,
            -PLAYER_MAX_SPEED,
            0.016,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        assert_eq!(right, -left);
    }

    #[test]
    fn test_deceleration_reduces_speed() {
        let result = apply_horizontal_acceleration(
            PLAYER_MAX_SPEED,
            0.0,
            0.016,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        assert!(result < PLAYER_MAX_SPEED);
        assert!(result >= 0.0);
    }

    #[test]
    fn test_deceleration_does_not_overshoot_zero() {
        // Large delta should decelerate all the way to zero, never past it.
        let result = apply_horizontal_acceleration(
            5.0,
            0.0,
            10.0,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_does_not_exceed_max_speed() {
        // Large delta + high acceleration should still be capped at max speed.
        let result = apply_horizontal_acceleration(
            0.0,
            PLAYER_MAX_SPEED,
            100.0,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        assert!(result <= PLAYER_MAX_SPEED);
    }

    #[test]
    fn test_zero_velocity_no_input_stays_zero() {
        let result = apply_horizontal_acceleration(
            0.0,
            0.0,
            0.016,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );
        assert_eq!(result, 0.0);
    }
}
