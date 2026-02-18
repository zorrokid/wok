use bevy::{
    ecs::{
        query::{With, Without},
        system::{Query, Res},
    },
    prelude::Camera2d,
    time::Time,
    transform::components::Transform,
};
use bevy_ecs_tiled::prelude::{TileStorage, TilemapSize};

use crate::{level::tile::TILE_SIZE, player::Player};

const CAMERA_LERP_SPEED: f32 = 5.0;
const VIEWPORT_WIDTH: f32 = 800.0;

pub fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    // bevy_ecs_tiled spawns a child entity per tile layer; each has TilemapSize + TileStorage.
    // All layers share the same tile dimensions, so any one of them gives the level width.
    tilemap_query: Query<&TilemapSize, With<TileStorage>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let lerp_factor = CAMERA_LERP_SPEED * time.delta_secs();

    camera_transform.translation.x +=
        (player_transform.translation.x - camera_transform.translation.x) * lerp_factor;
    camera_transform.translation.y +=
        (player_transform.translation.y - camera_transform.translation.y) * lerp_factor;

    // Clamp camera so the viewport never shows empty space beyond the map edges.
    // With TilemapAnchor::Center the map is centered on the world origin, so the
    // left edge is at -half_width and the right edge at +half_width.
    if let Some(tilemap_size) = tilemap_query.iter().next() {
        let half_width = tilemap_size.x as f32 * TILE_SIZE / 2.0;
        let camera_min_x = -half_width + VIEWPORT_WIDTH / 2.0;
        let camera_max_x = half_width - VIEWPORT_WIDTH / 2.0;

        // Only clamp when the level is wider than the viewport; otherwise leave the
        // camera free (the current 20-tile map is narrower than the 800px viewport).
        if camera_max_x > camera_min_x {
            camera_transform.translation.x =
                camera_transform.translation.x.clamp(camera_min_x, camera_max_x);
        }
    }
}
