pub mod coord;
pub mod movement;
pub mod spawn;

use bevy::ecs::component::Component;

// Constants
pub const PLAYER_MAX_SPEED: f32 = 100.0; // pixels per second
pub const PLAYER_ACCELERATION: f32 = 800.0; // pixels per second²
pub const PLAYER_DECELERATION: f32 = 1200.0; // pixels per second²
pub const JUMP_VELOCITY: f32 = 400.0; // pixels per second
pub const SPRITE_HEIGHT: f32 = 16.0;
pub const SPRITE_WIDTH: f32 = 16.0;

// Capsule collider dimensions. A capsule (rounded ends) is used instead of a
// rectangle so the player slides off platform edges rather than getting
// snagged on tile corners. Total height = 2*HALF_LENGTH + 2*RADIUS = 14px.
pub const COLLIDER_HALF_LENGTH: f32 = 2.0;
pub const COLLIDER_RADIUS: f32 = 5.0;

// Player marker component
#[derive(Component)]
pub struct Player;

// Present when the ShapeCaster detects ground beneath the player.
// Inserted/removed each frame by `update_grounded`. SparseSet is efficient
// for components that are added and removed frequently.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
