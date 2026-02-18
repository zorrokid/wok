pub const TILE_SIZE: f32 = 16.0;
pub const LEVEL_WIDTH_IN_TILES: u32 = 100;
pub const LEVEL_HEIGHT_IN_TILES: u32 = 30;

// Bottom-left corner of the map in world space.
// With TilemapAnchor::Center the map is centered on the world origin, so the
// bottom-left is at -(width/2, height/2). Update the dimensions above when the map changes.
pub const TILEMAP_OFFSET_X: f32 = -(LEVEL_WIDTH_IN_TILES as f32 * TILE_SIZE) / 2f32;
pub const TILEMAP_OFFSET_Y: f32 = -(LEVEL_HEIGHT_IN_TILES as f32 * TILE_SIZE) / 2f32;
