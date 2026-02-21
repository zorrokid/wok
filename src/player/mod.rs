pub mod health;
pub mod movement;
pub mod spawn;

use bevy::{app::{App, Plugin, Update}, ecs::component::Component, prelude::IntoScheduleConfigs};

use crate::player::{
    health::{tick_invincibility, respawn_player},
    movement::{player_movement, update_grounded},
    spawn::kill_zone,
};

pub use health::{Health, InvincibilityTimer, NeedsRespawn};

// Movement constants
pub const PLAYER_MAX_SPEED: f32 = 100.0; // pixels per second
pub const PLAYER_ACCELERATION: f32 = 800.0; // pixels per second²
pub const PLAYER_DECELERATION: f32 = 1200.0; // pixels per second²
pub const JUMP_VELOCITY: f32 = 400.0; // pixels per second
pub const SPRITE_HEIGHT: f32 = 16.0;

// Capsule collider dimensions. A capsule (rounded ends) is used instead of a
// rectangle so the player slides off platform edges rather than getting
// snagged on tile corners. Total height = 2*HALF_LENGTH + 2*RADIUS = 14px.
pub const COLLIDER_HALF_LENGTH: f32 = 2.0;
pub const COLLIDER_RADIUS: f32 = 5.0;
// Slightly narrower than the main collider to avoid false-positive ground
// detection at tile edges when the player is flush against a wall.
pub const SHAPE_CASTER_RADIUS: f32 = COLLIDER_RADIUS - 1.0;

pub const PLAYER_MAX_HP: i32 = 3;

// Player marker component
#[derive(Component)]
pub struct Player;

// Present when the ShapeCaster detects ground beneath the player.
// Inserted/removed each frame by `update_grounded`. SparseSet is efficient
// for components that are added and removed frequently.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // spawn_player is called from level::on_map_created on first map load,
        // so there is no Startup registration here.
        app.add_systems(
            Update,
            (tick_invincibility, update_grounded, player_movement, kill_zone, respawn_player)
                .chain(),
        );
    }
}
