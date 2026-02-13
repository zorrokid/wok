pub mod movement;

use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        system::{Commands, Res},
    },
    math::Vec2,
    sprite::Sprite,
    transform::components::Transform,
};

use crate::level::tile::{TILE_SIZE, TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y};

// Constants
const PLAYER_MAX_SPEED: f32 = 100.0; // pixels per second
const PLAYER_ACCELERATION: f32 = 800.0; // pixels per second²
const PLAYER_DECELERATION: f32 = 1200.0; // pixels per second²
const JUMP_VELOCITY: f32 = 300.0; // pixels per second
const GRAVITY: f32 = 980.0; // pixels per second²
const SPRITE_HEIGHT: f32 = 16.0;
const SPRITE_WIDTH: f32 = 16.0;

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
