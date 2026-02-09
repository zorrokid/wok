# Player Character Rendering

## Overview
Render a simple player character sprite on screen. The player is represented by a single static 16x16 pixel sprite that appears above the ground in the level. The sprite can be flipped horizontally to indicate facing direction.

## Requirements
- Player character appears on screen
- Player sprite is 16x16 pixels
- Player spawns at a fixed starting position above the ground
- Player sprite renders above the tilemap (correct z-order)
- Sprite uses placeholder graphics (colored square or simple character image)
- Player entity has a marker component for identification

## Acceptance Criteria
- [x] Player sprite is visible on screen
- [x] Player is 16x16 pixels in size
- [x] Player spawns at a reasonable starting position (on ground level)
- [x] Player renders above tilemap tiles (z-order correct)
- [x] Player marker component exists for future systems to query
- [x] Placeholder sprite asset is loaded and displayed

---

## Implementation Plan

### Approach
Create a simple player entity with Bevy's sprite rendering. Use a `Player` marker component for identification. Spawn the player at a fixed starting position (e.g., near the left side of the level, on the ground). Load a placeholder sprite image from the assets folder - can be a simple colored square initially, easily replaced with proper character art later.

### Components & Systems
**Components:**
- `Player` - Marker component to identify player entity
- Bevy's `Sprite` - For rendering
- Bevy's `Transform` - For position/rotation/scale

**Systems:**
- `spawn_player()` - Startup system to create player entity
- Loads sprite texture from assets
- Sets initial position above ground
- Configures z-order to render above tilemap

### Tasks
- [x] Create Player marker component
- [x] Create placeholder player sprite image (16x16 pixels)
- [x] Implement spawn_player startup system
- [x] Set player spawn position (e.g., x: 48.0, y: 48.0 to be on ground)
- [x] Configure sprite z-order (z: 10.0 for above tilemap)
- [x] Load player texture from assets/player.png
- [x] Test: Run and verify player sprite appears
- [x] Test: Verify player renders above tiles
- [x] Test: Verify player is at correct starting position

### Notes
- Player position is in world space pixels, not tile coordinates
- Z-order: tilemap is at z=0, player should be z=10.0 or higher
- Starting position should be calculated to place player on ground (consider tile size and level layout)
- For 16x16 sprite, position (48.0, 48.0) would place player at tile (3, 3) approximately
- Sprite origin is at center by default in Bevy
- Can add `TextureAtlas` later for animations, but single sprite is sufficient for now
- Consider spawning player at y position that accounts for 3 rows of ground tiles (around y = 64.0 to 80.0)
