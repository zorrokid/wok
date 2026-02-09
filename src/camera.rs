use bevy::{
    ecs::{
        query::{With, Without},
        system::{Query, Res},
    },
    prelude::Camera2d,
    time::Time,
    transform::components::Transform,
};

use crate::player::Player;

// Camera configuration
const CAMERA_LERP_SPEED: f32 = 5.0;

pub fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if let (Ok(player_transform), Ok(mut camera_transform)) =
        (player_query.single(), camera_query.single_mut())
    {
        let lerp_factor = CAMERA_LERP_SPEED * time.delta_secs();

        // Lerp camera position toward player position
        camera_transform.translation.x +=
            (player_transform.translation.x - camera_transform.translation.x) * lerp_factor;
        camera_transform.translation.y +=
            (player_transform.translation.y - camera_transform.translation.y) * lerp_factor;
    }
}
