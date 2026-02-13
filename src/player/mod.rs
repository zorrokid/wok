pub mod coord;
pub mod movement;
pub mod spawn;

use bevy::{ecs::component::Component, math::Vec2};

// Re-export coordinate types for convenience
pub use coord::{Coord, PlayerCoord};

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
