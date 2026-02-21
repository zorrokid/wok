mod camera;
mod collectibles;
mod keyboard;
mod level;
mod player;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::{
    camera::CameraPlugin,
    collectibles::CollectiblesPlugin,
    keyboard::exit_on_esc,
    level::LevelPlugin,
    player::PlayerPlugin,
};

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "WoK".into(),
                resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
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
        .add_plugins((LevelPlugin, PlayerPlugin, CameraPlugin, CollectiblesPlugin))
        .add_systems(Update, exit_on_esc)
        .run();
}
