pub const TILE_SIZE: f32 = 16.0;

// Bottom-left corner of the map in world space.
// With TilemapAnchor::Center on a 20x15 tile map these are -(20*16)/2 and -(15*16)/2.
// Update these when the map dimensions change (Phase 7).
pub const TILEMAP_OFFSET_X: f32 = -160.0;
pub const TILEMAP_OFFSET_Y: f32 = -120.0;
