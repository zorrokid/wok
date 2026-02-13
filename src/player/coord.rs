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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_new() {
        let coord = Coord::new(10.0, 20.0);
        assert_eq!(coord.x, 10.0);
        assert_eq!(coord.y, 20.0);
    }

    #[test]
    fn test_player_coord_feet_positions() {
        // Given a player at center position
        let center = Coord::new(100.0, 200.0);
        let player_coord = PlayerCoord::new(center);

        // SPRITE_HEIGHT = 16.0, so feet should be 8.0 pixels below center
        assert_eq!(player_coord.feet_y, 192.0); // 200.0 - 8.0

        // SPRITE_WIDTH = 16.0, FOOT_EDGE_INSET = 3.0
        // Left foot: 100.0 - (8.0 - 3.0) = 100.0 - 5.0 = 95.0
        assert_eq!(player_coord.feet_x_left, 95.0);
        // Right foot: 100.0 + (8.0 - 3.0) = 100.0 + 5.0 = 105.0
        assert_eq!(player_coord.feet_x_right, 105.0);
    }

    #[test]
    fn test_player_coord_at_origin() {
        // Test edge case at origin
        let center = Coord::new(0.0, 0.0);
        let player_coord = PlayerCoord::new(center);

        assert_eq!(player_coord.feet_y, -8.0);
        assert_eq!(player_coord.feet_x_left, -5.0);
        assert_eq!(player_coord.feet_x_right, 5.0);
    }

    #[test]
    fn test_ground_check_y() {
        // Given player coordinates
        let center = Coord::new(50.0, 100.0);
        let player_coord = PlayerCoord::new(center);

        // Ground check should be 1 pixel below feet
        // feet_y = 100.0 - 8.0 = 92.0
        // ground_check_y = 92.0 - 1.0 = 91.0
        assert_eq!(player_coord.ground_check_y(), 91.0);
    }

    #[test]
    fn test_ground_check_y_consistency() {
        // Verify GROUND_CHECK_OFFSET is applied correctly
        let center = Coord::new(0.0, 50.0);
        let player_coord = PlayerCoord::new(center);

        let expected = player_coord.feet_y - 1.0; // GROUND_CHECK_OFFSET
        assert_eq!(player_coord.ground_check_y(), expected);
    }

    #[test]
    fn test_feet_width() {
        // Verify feet are 10 pixels apart (5 pixels from center each side)
        let center = Coord::new(100.0, 100.0);
        let player_coord = PlayerCoord::new(center);

        let feet_distance = player_coord.feet_x_right - player_coord.feet_x_left;
        assert_eq!(feet_distance, 10.0); // (8.0 - 3.0) * 2 = 5.0 * 2 = 10.0
    }

    #[test]
    fn test_center_preserved() {
        // Ensure center coordinate is preserved
        let center = Coord::new(42.0, 84.0);
        let player_coord = PlayerCoord::new(center.clone());

        assert_eq!(player_coord.center.x, 42.0);
        assert_eq!(player_coord.center.y, 84.0);
    }
}
