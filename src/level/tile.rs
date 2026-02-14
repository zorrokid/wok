use crate::level::{LEVEL_DATA, LEVEL_HEIGHT_IN_TILES, LEVEL_WIDTH_IN_TILES};

pub const TILE_SIZE: f32 = 16.0;
// Tilemap offset (same as in level.rs)
pub const TILEMAP_OFFSET_X: f32 = -(LEVEL_WIDTH_IN_TILES as f32 * TILE_SIZE) / 2.0;
pub const TILEMAP_OFFSET_Y: f32 = -(LEVEL_HEIGHT_IN_TILES as f32 * TILE_SIZE) / 2.0;

#[derive(PartialEq, Eq, Debug)]
pub enum TileType {
    Solid = 1,
    Empty = 0,
}

impl From<u32> for TileType {
    fn from(value: u32) -> Self {
        match value {
            1 => TileType::Solid,
            _ => TileType::Empty,
        }
    }
}

// Helper function: Check if tile is solid
pub fn is_solid_tile(tile_type: TileType) -> bool {
    tile_type == TileType::Solid
}

// Helper function: Get tile type at position from LEVEL_DATA
pub fn get_tile_type_at(tile_x: i32, tile_y: i32) -> Option<TileType> {
    if tile_x < 0
        || tile_y < 0
        || tile_x >= LEVEL_WIDTH_IN_TILES as i32
        || tile_y >= LEVEL_HEIGHT_IN_TILES as i32
    {
        return None;
    }

    // Convert tilemap Y to array index (Y=0 is bottom in tilemap, but top in array)
    let array_y = (LEVEL_HEIGHT_IN_TILES - 1) as i32 - tile_y;
    if array_y < 0 || array_y >= LEVEL_HEIGHT_IN_TILES as i32 {
        return None;
    }
    let tile_type: TileType = LEVEL_DATA[array_y as usize][tile_x as usize].into();
    Some(tile_type)
}

// Helper function: Convert world position to tile coordinates
pub fn world_to_tile_coords(world_x: f32, world_y: f32) -> (i32, i32) {
    let tile_x = ((world_x - TILEMAP_OFFSET_X) / TILE_SIZE).floor() as i32;
    let tile_y = ((world_y - TILEMAP_OFFSET_Y) / TILE_SIZE).floor() as i32;
    (tile_x, tile_y)
}
