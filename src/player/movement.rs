use std::process::exit;

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

/// Tile information for both feet positions
struct FeetTiles {
    left: Option<TileType>,
    right: Option<TileType>,
    left_tile_y: i32,
    right_tile_y: i32,
}

/// Check tiles beneath both feet and return tile information
fn check_feet_tiles(
    player_coord: &PlayerCoord,
    world_to_tile_coords: impl Fn(f32, f32) -> (i32, i32),
    get_tile_type_at: impl Fn(i32, i32) -> Option<TileType>,
) -> FeetTiles {
    let check_y = player_coord.ground_check_y();

    // Convert both foot positions to tile coordinates
    let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord.feet_x_left, check_y);
    let (right_tile_x, right_tile_y) = world_to_tile_coords(player_coord.feet_x_right, check_y);

    // Get tile types at both positions
    let left = get_tile_type_at(left_tile_x, left_tile_y);
    let right = get_tile_type_at(right_tile_x, right_tile_y);

    /* In this situation, right foot is visually still on top of platofrm:
     *
     * Player center: (23.99, -16.00), Player feet left X=15.99, right X=3
     *
     * Left side of tile map: -160
     *
     * Left empty tile (10, 5) = Some(Empty)
     * Left side of left tile 10 is: -160 + 10 * 16 = 0
     * Right side of left tile 10 is: -160 + 10 * 16 + 16 = 16
     *
     * Right empty tile (11, 5) = Some(Empty)
     * Left side of the right tile 11 is: -160 + 11 * 16 = 16
     * Right side of the right tile 11 is: -160 + 11 * 16 + 16 = 32
     *
     * But it doesn't make sense that
     * feet left X=15.99, right X=3
     *
     * Left feet coordinate should be smaller than right feet coordinate!
     *
     * Checking feet tiles at Y=-25.00: Left tile (11, 5) = Some(Empty), Right tile (12, 5) = Some(Solid)
    Player position: (24.13, -16.00), Velocity: (0.00, 0.00), Grounded: false
    * Checking feet tiles at Y=-25.00: Left tile (10, 5) = Some(Empty), Right tile (11, 5) = Some(Empty)
    Feet check starts falling at Y=-25.00: Left tile (10, 5) = Some(Empty), Right tile (11, 5) = Some(Empty), Player center: (23.99, -16.00), Player feet left X=15.99, right X=3
    1.99
    */

    println!(
        "Checking feet tiles at Y={:.2}: Left tile ({}, {}) = {:?}, Right tile ({}, {}) = {:?}",
        check_y, left_tile_x, left_tile_y, left, right_tile_x, right_tile_y, right,
    );
    if left == Some(TileType::Empty) && right == Some(TileType::Empty) {
        println!(
            "Feet check starts falling at Y={:.2}: Left tile ({}, {}) = {:?}, Right tile ({}, {}) = {:?}, Player center: ({:.2}, {:.2}), Player feet left X={:.2}, right X={:.2}",
            check_y,
            left_tile_x,
            left_tile_y,
            left,
            right_tile_x,
            right_tile_y,
            right,
            player_coord.center.x,
            player_coord.center.y,
            player_coord.feet_x_left,
            player_coord.feet_x_right
        );
    }
    if left.is_none() && right.is_none() {
        //Error: Feet check out of bounds at
        // Y=-25.00:
        // Left tile (-1, 5) = None,
        // Right tile (0, 5) = Some(Empty),
        // Player center: (-155.63, -16.00),
        // Player feet left X=-160.63, right X=-150.63
        //
        // Left tile x-coordinates should be -160 to -144, right tile x-coordinates should be -144
        // to -128
        //
        // While player is visually still on top of the platform under right foot, the calculated
        // tile coordinates for also for the right foot are out of bounds, causing the system to treat it as
        // falling off the edge prematurely.
        println!(
            "Error: Feet check out of bounds at Y={:.2}: Left tile ({}, {}) = {:?}, Right tile ({}, {}) = {:?}, Player center: ({:.2}, {:.2}), Player feet left X={:.2}, right X={:.2}",
            check_y,
            left_tile_x,
            left_tile_y,
            left,
            right_tile_x,
            right_tile_y,
            right,
            player_coord.center.x,
            player_coord.center.y,
            player_coord.feet_x_left,
            player_coord.feet_x_right
        );
        exit(1);
    }

    FeetTiles {
        left,
        right,
        left_tile_y,
        right_tile_y,
    }
}

/// Player movement system handling input, physics, and collision detection.
///
/// This system runs each frame and performs the following:
/// 1. Reads keyboard input (arrow keys for movement, Z for jump)
/// 2. Checks if player is grounded using dual-foot collision detection
/// 3. Updates horizontal velocity based on input with smooth acceleration/deceleration
/// 4. Updates vertical velocity with gravity and jump handling
/// 5. Applies velocity to player position
/// 6. Snaps player to ground surface if landing on solid tiles
///
/// The collision system checks both feet independently, allowing the player to stand
/// on platform edges. When landing, the player snaps to the tile surface to prevent
/// floating or sinking into the ground.
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

        // Here starts falling in the left side (too soon)
        // Player position: (27.18, -16.00), Velocity: (-45.30, 0.00), Grounded: true
        // Player position: (26.81, -16.00), Velocity: (-30.35, 0.00), Grounded: false

        // Here starts falling in the right side (as supposed to be)
        //Player position: (100.58, -16.00), Velocity: (81.32, 0.00), Grounded: true
        //Player position: (101.63, -16.00), Velocity: (60.52, 0.00), Grounded: false
        println!(
            "Player position: ({:.2}, {:.2}), Velocity: ({:.2}, {:.2}), Grounded: {}",
            transform.translation.x,
            transform.translation.y,
            velocity.0.x,
            velocity.0.y,
            is_grounded
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

/// Calculates vertical velocity with gravity and jump handling.
///
/// Applies gravity when airborne or moving upward (jumping). When grounded and not jumping,
/// velocity is set to zero to prevent the player from "sticking" to the ground or drifting.
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
        //velocity -= GRAVITY * delta;
    } else {
        velocity = 0.0; // Stop falling when on ground
    }
    velocity
}

/// Determines target horizontal velocity based on directional input.
///
/// Left takes priority when both directions are pressed.
fn get_target_velocity_x(is_left_pressed: bool, is_right_pressed: bool) -> f32 {
    match (is_left_pressed, is_right_pressed) {
        (true, _) => -PLAYER_MAX_SPEED,
        (_, true) => PLAYER_MAX_SPEED,
        _ => 0.0,
    }
}

/// Checks if player is standing on solid ground.
///
/// Returns true if EITHER foot is on a solid tile, allowing the player to stand
/// on platform edges without falling through prematurely.
fn is_grounded(
    player_coord: &PlayerCoord,
    world_to_tile_coords: impl Fn(f32, f32) -> (i32, i32),
    get_tile_type_at: impl Fn(i32, i32) -> Option<TileType>,
    is_solid_tile: impl Fn(TileType) -> bool,
) -> bool {
    let feet_tiles = check_feet_tiles(player_coord, world_to_tile_coords, get_tile_type_at);

    feet_tiles.left.map(&is_solid_tile).unwrap_or(false)
        && feet_tiles.right.map(&is_solid_tile).unwrap_or(false)
}

/// Calculates the Y position to snap player to when landing on solid ground.
///
/// Returns the Y coordinate for the player's center (sprite center) aligned to the top
/// of the tile surface. Returns None if no solid ground beneath either foot.
///
/// When both feet are on solid tiles, snaps to the left foot's tile to maintain stability.
fn ground_snap_y(
    player_coord_after: &PlayerCoord,
    world_to_tile_coords: impl Fn(f32, f32) -> (i32, i32),
    get_tile_type_at: impl Fn(i32, i32) -> Option<TileType>,
    is_solid_tile: impl Fn(TileType) -> bool,
    tilemap_offset_y: f32,
    tile_size: f32,
    sprite_height: f32,
) -> Option<f32> {
    let feet_tiles = check_feet_tiles(player_coord_after, world_to_tile_coords, get_tile_type_at);

    let left_solid = feet_tiles.left.map(&is_solid_tile).unwrap_or(false);
    let right_solid = feet_tiles.right.map(&is_solid_tile).unwrap_or(false);

    if left_solid || right_solid {
        let snap_tile_y = if left_solid {
            feet_tiles.left_tile_y
        } else {
            feet_tiles.right_tile_y
        };
        let tile_top_y = tilemap_offset_y + ((snap_tile_y + 1) as f32 * tile_size);
        Some(tile_top_y + sprite_height / 2.0)
    } else {
        None
    }
}

/// Applies acceleration or deceleration to horizontal velocity.
///
/// When moving toward a target velocity, accelerates without overshooting.
/// When no input is given, decelerates smoothly to zero without oscillation.
/// Enforces maximum speed limit in both directions.
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
