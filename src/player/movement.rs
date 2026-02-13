use bevy::{
    ecs::{
        query::With,
        system::{Query, Res},
    },
    input::ButtonInput,
    prelude::KeyCode,
    time::Time,
    transform::components::Transform,
};

use crate::{
    level::tile::{
        TILE_SIZE, TILEMAP_OFFSET_Y, TileType, get_tile_type_at, is_solid_tile,
        world_to_tile_coords,
    },
    player::{
        GRAVITY, JUMP_VELOCITY, PLAYER_ACCELERATION, PLAYER_DECELERATION, PLAYER_MAX_SPEED, Player,
        PlayerCoord, SPRITE_HEIGHT, Velocity,
    },
};

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((mut transform, mut velocity)) = query.single_mut() {
        let is_jump_pressed = keyboard.just_pressed(KeyCode::KeyZ);
        let is_left_pressed = keyboard.pressed(KeyCode::ArrowLeft);
        let is_right_pressed = keyboard.pressed(KeyCode::ArrowRight);
        let delta = time.delta_secs();
        let player_coord: PlayerCoord = (*transform).into();

        let is_grounded = is_grounded(
            &player_coord,
            world_to_tile_coords,
            get_tile_type_at,
            is_solid_tile,
        );

        let target_velocity_x = get_target_velocity_x(is_left_pressed, is_right_pressed);

        velocity.0.x = apply_horizontal_acceleration(
            velocity.0.x,
            target_velocity_x,
            delta,
            PLAYER_ACCELERATION,
            PLAYER_DECELERATION,
            PLAYER_MAX_SPEED,
        );

        velocity.0.y = get_velocity_y(
            is_grounded,
            velocity.0.y,
            delta,
            JUMP_VELOCITY,
            is_jump_pressed,
        );

        // Apply velocity to position
        transform.translation.x += velocity.0.x * delta;
        transform.translation.y += velocity.0.y * delta;

        // Get updated player coordinates after movement
        let player_coord_after: PlayerCoord = (*transform).into();

        if let Some(snap_y) = ground_snap_y(
            &player_coord_after,
            world_to_tile_coords,
            get_tile_type_at,
            is_solid_tile,
            TILEMAP_OFFSET_Y,
            TILE_SIZE,
            SPRITE_HEIGHT,
        ) {
            transform.translation.y = snap_y;
            velocity.0.y = 0.0;
        }
    }
}

fn get_velocity_y(
    is_grounded: bool,
    current_velocity_y: f32,
    delta: f32,
    jump_velocity: f32,
    is_jump_pressed: bool,
) -> f32 {
    let mut velocity = current_velocity_y;

    if is_jump_pressed && is_grounded {
        velocity = jump_velocity;
    }

    // Apply gravity (only when not grounded or when jumping up)
    if !is_grounded || velocity > 0.0 {
        velocity -= GRAVITY * delta;
    } else {
        velocity = 0.0; // Stop falling when on ground
    }
    velocity
}

// Horizontal movement - determine target velocity based on input
fn get_target_velocity_x(is_left_pressed: bool, is_right_pressed: bool) -> f32 {
    match (is_left_pressed, is_right_pressed) {
        (true, _) => -PLAYER_MAX_SPEED,
        (_, true) => PLAYER_MAX_SPEED,
        _ => 0.0,
    }
}

fn is_grounded(
    player_coord: &PlayerCoord,
    world_to_tile_coords: impl Fn(f32, f32) -> (i32, i32),
    get_tile_type_at: impl Fn(i32, i32) -> Option<TileType>,
    is_solid_tile: impl Fn(TileType) -> bool,
) -> bool {
    // Check for ground slightly below feet
    let check_y = player_coord.ground_check_y();

    // Convert both foot positions to tile coordinates
    let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord.feet_x_left, check_y);
    let (right_tile_x, right_tile_y) = world_to_tile_coords(player_coord.feet_x_right, check_y);

    let left_tile_below = get_tile_type_at(left_tile_x, left_tile_y);
    let right_tile_below = get_tile_type_at(right_tile_x, right_tile_y);

    left_tile_below.map(&is_solid_tile).unwrap_or(false)
        || right_tile_below.map(&is_solid_tile).unwrap_or(false)
}

fn ground_snap_y(
    player_coord_after: &PlayerCoord,
    world_to_tile_coords: impl Fn(f32, f32) -> (i32, i32),
    get_tile_type_at: impl Fn(i32, i32) -> Option<TileType>,
    is_solid_tile: impl Fn(TileType) -> bool,
    tilemap_offset_y: f32,
    tile_size: f32,
    sprite_height: f32,
) -> Option<f32> {
    let check_y = player_coord_after.ground_check_y();
    let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord_after.feet_x_left, check_y);
    let (right_tile_x, right_tile_y) =
        world_to_tile_coords(player_coord_after.feet_x_right, check_y);

    let left_tile = get_tile_type_at(left_tile_x, left_tile_y);
    let right_tile = get_tile_type_at(right_tile_x, right_tile_y);

    let left_solid = left_tile.map(&is_solid_tile).unwrap_or(false);
    let right_solid = right_tile.map(&is_solid_tile).unwrap_or(false);

    if left_solid || right_solid {
        let snap_tile_y = if left_solid {
            left_tile_y
        } else {
            right_tile_y
        };
        let tile_top_y = tilemap_offset_y + ((snap_tile_y + 1) as f32 * tile_size);
        Some(tile_top_y + sprite_height / 2.0)
    } else {
        None
    }
}

fn apply_horizontal_acceleration(
    velocity_x: f32,
    target_velocity_x: f32,
    delta: f32,
    player_acceleration: f32,
    player_deceleration: f32,
    player_max_speed: f32,
) -> f32 {
    let mut velocity_x = if target_velocity_x != 0.0 {
        // Accelerate toward target
        let accel_direction = (target_velocity_x - velocity_x).signum();
        let new_velocity_x = velocity_x + accel_direction * player_acceleration * delta;

        // Clamp to target (don't overshoot)
        if accel_direction > 0.0 {
            new_velocity_x.min(target_velocity_x)
        } else {
            new_velocity_x.max(target_velocity_x)
        }
    } else {
        // Decelerate to zero
        if velocity_x.abs() > 0.0 {
            let decel_amount = player_deceleration * delta;

            if velocity_x > 0.0 {
                (velocity_x - decel_amount).max(0.0)
            } else if velocity_x < 0.0 {
                (velocity_x + decel_amount).min(0.0)
            } else {
                velocity_x
            }
        } else {
            velocity_x
        }
    };
    // Clamp horizontal velocity to max speed
    velocity_x = velocity_x.clamp(-player_max_speed, player_max_speed);
    velocity_x
}
