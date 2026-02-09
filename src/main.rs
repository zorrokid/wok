mod keyboard;
mod level;
mod player;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::{keyboard::exit_on_esc, level::setup_tilemap, player::{spawn_player, player_movement}};

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
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, (setup_tilemap, spawn_player))
        .add_systems(Update, (player_movement, exit_on_esc))
        .run();
}
