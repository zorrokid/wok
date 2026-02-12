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

use crate::level::{LEVEL_DATA, LEVEL_HEIGHT, LEVEL_WIDTH};

// Constants
const PLAYER_MAX_SPEED: f32 = 100.0; // pixels per second
const PLAYER_ACCELERATION: f32 = 800.0; // pixels per second²
const PLAYER_DECELERATION: f32 = 1200.0; // pixels per second²
const JUMP_VELOCITY: f32 = 300.0; // pixels per second
const GRAVITY: f32 = 980.0; // pixels per second²
const TILE_SIZE: f32 = 16.0;
const SPRITE_HEIGHT: f32 = 16.0;
const SPRITE_WIDTH: f32 = 16.0;

// Tilemap offset (same as in level.rs)
const TILEMAP_OFFSET_X: f32 = -(LEVEL_WIDTH as f32 * TILE_SIZE) / 2.0;
const TILEMAP_OFFSET_Y: f32 = -(LEVEL_HEIGHT as f32 * TILE_SIZE) / 2.0;

// Player marker component
#[derive(Component)]
pub struct Player;

// Velocity component for physics-based movement
#[derive(Component)]
pub struct Velocity(pub Vec2);

// Helper function: Convert world position to tile coordinates
fn world_to_tile_coords(world_x: f32, world_y: f32) -> (i32, i32) {
    let tile_x = ((world_x - TILEMAP_OFFSET_X) / TILE_SIZE).floor() as i32;
    let tile_y = ((world_y - TILEMAP_OFFSET_Y) / TILE_SIZE).floor() as i32;
    (tile_x, tile_y)
}

// Helper function: Check if tile is solid
fn is_solid_tile(tile_type: u32) -> bool {
    tile_type == 1 // Type 1 is solid platform
}

// Helper function: Get tile type at position from LEVEL_DATA
fn get_tile_type_at(tile_x: i32, tile_y: i32) -> Option<u32> {
    if tile_x < 0 || tile_y < 0 || tile_x >= LEVEL_WIDTH as i32 || tile_y >= LEVEL_HEIGHT as i32 {
        return None;
    }

    // Convert tilemap Y to array index (Y=0 is bottom in tilemap, but top in array)
    let array_y = (LEVEL_HEIGHT - 1) as i32 - tile_y;
    if array_y < 0 || array_y >= LEVEL_HEIGHT as i32 {
        return None;
    }

    Some(LEVEL_DATA[array_y as usize][tile_x as usize])
}

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

#[derive(Clone)]
struct Coord {
    x: f32,
    y: f32,
}

impl Coord {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<Transform> for Coord {
    fn from(transform: Transform) -> Self {
        Coord {
            x: transform.translation.x,
            y: transform.translation.y,
        }
    }
}

struct PlayerCoord {
    center: Coord,
    feet_y: f32,
    feet_x_left: f32,
    feet_x_right: f32,
}

impl PlayerCoord {
    fn new(center: Coord) -> Self {
        Self {
            center: center.clone(),
            feet_y: center.y - SPRITE_HEIGHT / 2.0,
            feet_x_left: center.x - (SPRITE_WIDTH / 2.0 - 3.0),
            feet_x_right: center.x + (SPRITE_WIDTH / 2.0 - 3.0),
        }
    }
}

impl From<Transform> for PlayerCoord {
    fn from(transform: Transform) -> Self {
        PlayerCoord::new(transform.into())
    }
}

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if let Ok((mut transform, mut velocity)) = query.single_mut() {
        let delta = time.delta_secs();

        let player_coord: PlayerCoord = (*transform).into();

        // Check for ground slightly below feet (1 pixel below to detect tile surface)
        let check_y = player_coord.feet_y - 1.0;

        // Convert both foot positions to tile coordinates
        let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord.feet_x_left, check_y);
        let (right_tile_x, right_tile_y) = world_to_tile_coords(player_coord.feet_x_right, check_y);

        // Check if there's a solid tile beneath either foot
        let left_tile_below = get_tile_type_at(left_tile_x, left_tile_y);
        let right_tile_below = get_tile_type_at(right_tile_x, right_tile_y);

        let is_grounded = left_tile_below.map(is_solid_tile).unwrap_or(false)
            || right_tile_below.map(is_solid_tile).unwrap_or(false);

        // Jump input
        if keyboard.just_pressed(KeyCode::KeyZ) && is_grounded {
            velocity.0.y = JUMP_VELOCITY;
        }

        // Horizontal movement - determine target velocity based on input
        let mut target_velocity_x = 0.0;
        if keyboard.pressed(KeyCode::ArrowLeft) {
            target_velocity_x -= PLAYER_MAX_SPEED;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            target_velocity_x += PLAYER_MAX_SPEED;
        }

        // Apply acceleration or deceleration (horizontal)
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

        // Clamp horizontal velocity to max speed
        velocity.0.x = velocity.0.x.clamp(-PLAYER_MAX_SPEED, PLAYER_MAX_SPEED);

        // Apply gravity (only when not grounded or when jumping up)
        if !is_grounded || velocity.0.y > 0.0 {
            velocity.0.y -= GRAVITY * delta;
        } else {
            velocity.0.y = 0.0; // Stop falling when on ground
        }

        // Apply velocity to position
        transform.translation.x += velocity.0.x * delta;
        transform.translation.y += velocity.0.y * delta;

        // Get updated player coordinates after movement
        let player_coord_after: PlayerCoord = (*transform).into();

        // Ground collision - re-check at new position and snap if on solid tile
        let new_check_y = player_coord_after.feet_y - 1.0;

        let (new_left_tile_x, new_left_tile_y) =
            world_to_tile_coords(player_coord_after.feet_x_left, new_check_y);
        let (new_right_tile_x, new_right_tile_y) =
            world_to_tile_coords(player_coord_after.feet_x_right, new_check_y);

        let new_left_tile = get_tile_type_at(new_left_tile_x, new_left_tile_y);
        let new_right_tile = get_tile_type_at(new_right_tile_x, new_right_tile_y);

        let left_solid = new_left_tile.map(is_solid_tile).unwrap_or(false);
        let right_solid = new_right_tile.map(is_solid_tile).unwrap_or(false);

        if left_solid || right_solid {
            let snap_tile_y = if left_solid {
                new_left_tile_y
            } else {
                new_right_tile_y
            };
            let tile_top_y = TILEMAP_OFFSET_Y + ((snap_tile_y + 1) as f32 * TILE_SIZE);
            transform.translation.y = tile_top_y + SPRITE_HEIGHT / 2.0;
            velocity.0.y = 0.0;
        }
    }
}
