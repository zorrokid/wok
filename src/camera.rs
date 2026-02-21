use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        query::{With, Without},
        system::{Query, Res},
    },
    prelude::Camera2d,
    time::Time,
    transform::components::Transform,
};
use bevy_ecs_tiled::prelude::{TileStorage, TilemapSize};

use crate::{level::tile::MapDimensions, player::Player, WINDOW_WIDTH};

const CAMERA_LERP_SPEED: f32 = 5.0;
/// When the camera is further than this from the player, snap immediately
/// instead of lerping. This prevents the slow scroll across the full map
/// width that occurs when the player teleports during a map transition.
const CAMERA_SNAP_THRESHOLD: f32 = 300.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, camera_follow);
    }
}

fn setup_camera(mut commands: bevy::ecs::system::Commands) {
    commands.spawn(Camera2d);
}

pub fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    // bevy_ecs_tiled spawns a child entity per tile layer; each has TilemapSize + TileStorage.
    // All layers share the same tile dimensions, so any one of them gives the level width.
    tilemap_query: Query<&TilemapSize, With<TileStorage>>,
    map_dims: Res<MapDimensions>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let dx = player_transform.translation.x - camera_transform.translation.x;
    let dy = player_transform.translation.y - camera_transform.translation.y;

    let lerp_factor = if dx.abs() > CAMERA_SNAP_THRESHOLD || dy.abs() > CAMERA_SNAP_THRESHOLD {
        1.0
    } else {
        CAMERA_LERP_SPEED * time.delta_secs()
    };

    camera_transform.translation.x += dx * lerp_factor;
    camera_transform.translation.y += dy * lerp_factor;

    // Clamp camera so the viewport never shows empty space beyond the map edges.
    // With TilemapAnchor::Center the map is centered on the world origin, so the
    // left edge is at -half_width and the right edge at +half_width.
    if let Some(tilemap_size) = tilemap_query.iter().next() {
        let half_width = tilemap_size.x as f32 * map_dims.tile_size / 2.0;
        let camera_min_x = -half_width + WINDOW_WIDTH / 2.0;
        let camera_max_x = half_width - WINDOW_WIDTH / 2.0;

        // Only clamp when the level is wider than the viewport; otherwise leave the
        // camera free (the current 20-tile map is narrower than the 800px viewport).
        if camera_max_x > camera_min_x {
            camera_transform.translation.x =
                camera_transform.translation.x.clamp(camera_min_x, camera_max_x);
        }
    }
}
