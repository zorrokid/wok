use bevy::prelude::{Resource, Vec2};
use bevy_ecs_tiled::prelude::TiledMapAsset;

/// Runtime map dimensions populated from the loaded `TiledMapAsset`.
///
/// Inserted/updated by `on_map_created` each time a map finishes loading so
/// that level-bounds walls, camera clamping, spawn positions, and the kill
/// zone all reflect the actual map dimensions rather than compile-time
/// constants. The `Default` impl gives zeros; systems that need real data
/// must run after `on_map_created` (which fires in `PreUpdate`).
#[derive(Resource, Default, Clone, Copy)]
pub struct MapDimensions {
    pub tile_size: f32,
    pub width_tiles: u32,
    pub height_tiles: u32,
    /// World X of the map's left edge (negative half-width under Center anchor).
    pub offset_x: f32,
    /// World Y of the map's bottom edge.
    pub offset_y: f32,
}

impl MapDimensions {
    pub fn width_px(&self) -> f32 {
        self.width_tiles as f32 * self.tile_size
    }

    pub fn height_px(&self) -> f32 {
        self.height_tiles as f32 * self.tile_size
    }

    /// Convert tile column/row → world-space center of that tile.
    ///
    /// `tile_x` is column (left→right), `tile_y` is row (bottom→top, matching
    /// Bevy's Y-up coordinate system). Both may be fractional.
    pub fn tile_to_world(&self, tile_x: f32, tile_y: f32) -> Vec2 {
        Vec2::new(
            self.offset_x + tile_x * self.tile_size + self.tile_size / 2.0,
            self.offset_y + tile_y * self.tile_size + self.tile_size / 2.0,
        )
    }
}

/// Extracts map dimensions from a loaded `TiledMapAsset`.
///
/// Pure function — no Bevy World access — so it can be unit-tested without
/// spinning up a full app. Called from `on_map_created` each time a map loads.
pub fn map_dimensions_from_asset(asset: &TiledMapAsset) -> MapDimensions {
    let ts = asset.largest_tile_size;
    let tile_size = ts.x; // square tiles: x == y
    let w = asset.tilemap_size.x;
    let h = asset.tilemap_size.y;
    MapDimensions {
        tile_size,
        width_tiles: w,
        height_tiles: h,
        offset_x: -(w as f32 * tile_size) / 2.0,
        offset_y: -(h as f32 * tile_size) / 2.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_dimensions_100x30_tile_to_world() {
        let dims = MapDimensions {
            tile_size: 16.0,
            width_tiles: 100,
            height_tiles: 30,
            offset_x: -800.0,
            offset_y: -240.0,
        };
        // Tile (0, 0) bottom-left corner → center of that tile
        assert_eq!(dims.tile_to_world(0.0, 0.0), Vec2::new(-792.0, -232.0));
    }

    #[test]
    fn map_dimensions_100x30_width_height_px() {
        let dims = MapDimensions {
            tile_size: 16.0,
            width_tiles: 100,
            height_tiles: 30,
            offset_x: -800.0,
            offset_y: -240.0,
        };
        assert_eq!(dims.width_px(), 1600.0);
        assert_eq!(dims.height_px(), 480.0);
    }

    #[test]
    fn map_dimensions_offset_is_negative_half_size() {
        let dims = MapDimensions {
            tile_size: 16.0,
            width_tiles: 100,
            height_tiles: 30,
            offset_x: -800.0,
            offset_y: -240.0,
        };
        assert_eq!(dims.offset_x, -dims.width_px() / 2.0);
        assert_eq!(dims.offset_y, -dims.height_px() / 2.0);
    }
}
