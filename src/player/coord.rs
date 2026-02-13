use bevy::transform::components::Transform;

use super::{SPRITE_HEIGHT, SPRITE_WIDTH};

/// Distance from sprite edge where foot collision is checked (pixels)
const FOOT_EDGE_INSET: f32 = 3.0;

/// Offset below feet where ground detection occurs (pixels)
const GROUND_CHECK_OFFSET: f32 = 1.0;

/// Simple 2D coordinate
#[derive(Clone)]
pub struct Coord {
    pub x: f32,
    pub y: f32,
}

impl Coord {
    pub fn new(x: f32, y: f32) -> Self {
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

/// Player position with derived foot positions for collision detection
pub struct PlayerCoord {
    pub center: Coord,
    pub feet_y: f32,
    pub feet_x_left: f32,
    pub feet_x_right: f32,
}

impl PlayerCoord {
    pub fn new(center: Coord) -> Self {
        Self {
            center: center.clone(),
            feet_y: center.y - SPRITE_HEIGHT / 2.0,
            feet_x_left: center.x - (SPRITE_WIDTH / 2.0 - FOOT_EDGE_INSET),
            feet_x_right: center.x + (SPRITE_WIDTH / 2.0 - FOOT_EDGE_INSET),
        }
    }

    /// Get the Y position to check for ground (slightly below feet)
    pub fn ground_check_y(&self) -> f32 {
        self.feet_y - GROUND_CHECK_OFFSET
    }
}

impl From<Transform> for PlayerCoord {
    fn from(transform: Transform) -> Self {
        PlayerCoord::new(transform.into())
    }
}
