mod camera;
mod keyboard;
mod level;
mod player;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::{
    camera::camera_follow,
    keyboard::exit_on_esc,
    level::setup_tilemap_new,
    player::{
        movement::{player_movement, update_grounded},
        spawn::spawn_player,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "WoK".into(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        // Loads and renders Tiled .tmx map files, spawning ECS entities for
        // each layer and tile.
        .add_plugins(TiledPlugin::default())
        // Scans tile layers and generates Avian2D Collider components on tile
        // entities. Does NOT add RigidBody — see the observer in level/mod.rs.
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        // Core Avian2D physics: collision detection, constraint solving, and
        // velocity integration. with_length_unit(100.0) tells Avian that one
        // unit equals 100 pixels, keeping gravity/force values in human-readable
        // pixel-space numbers (e.g. 980 px/s² instead of 9.81 m/s²).
        .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        // Global gravity applied to all Dynamic rigid bodies each physics step.
        // NEG_Y * 980.0 matches the gravity constant used in the old custom system.
        .insert_resource(Gravity(Vec2::NEG_Y * 980.0))
        .add_systems(Startup, (setup_tilemap_new, spawn_player))
        // update_grounded reads ShapeHits from the last physics step and inserts/removes
        // the Grounded marker. It must run before player_movement so the jump check sees
        // the current grounded state. Both run in Update (not FixedUpdate) so that
        // keyboard.just_pressed() reliably fires exactly once per player input.
        .add_systems(Update, (update_grounded, player_movement).chain())
        .add_systems(Update, (camera_follow, exit_on_esc))
        .run();
}
