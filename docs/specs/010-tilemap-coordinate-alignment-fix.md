# 010: Tilemap Coordinate Alignment Fix

## Overview
Fix the asymmetric platform edge collision bug where the player would fall off the left side of platforms prematurely while the right side worked correctly. The root cause was a misalignment between visual tile rendering and collision detection due to `bevy_ecs_tilemap`'s default center-anchoring of tiles.

## Problem Statement

### Observed Behavior
- Player falls off left platform edges when ~8 pixels of sprite is still visually on the platform
- Right platform edges work correctly - player only falls when fully off the platform
- Asymmetric behavior caused player confusion and made platforming feel inconsistent

### Technical Root Cause
`bevy_ecs_tilemap` with default `TilemapAnchor::None` positions tiles by their **center**, but our collision math in `world_to_tile_coords` assumes tiles are positioned by their **bottom-left corner**. This creates an 8-pixel (half-tile) misalignment between visual rendering and collision grid boundaries.

**Example scenario:**
```
Platform at tile indices 12-15 (tile_y=5)
Visual rendering with center-anchoring:
  Tile 12 center: (-160, -120) + (12*16, 5*16) = (32, -40)
  Tile 12 spans: [24, 40] in X

Collision detection (corner-aligned assumption):
  Tile 12 left edge: -160 + 12*16 = 32
  Player right foot at X=31.99 → floor((31.99+160)/16) = floor(11.999) = 11 → Empty!
```

Player's right foot at X=31.99 is visually on tile 12, but collision system thinks it's on tile 11 (empty), causing premature falling.

### Why Only Left Edge?
The `is_grounded` function originally used `&&` (both feet must be on solid ground), so:
- **Left edge**: Left foot leaves platform first → both feet check fails immediately → falls too early
- **Right edge**: Right foot leaves last → by the time both feet are off, it looks correct visually

## Requirements
- [x] Player falls off platform edges symmetrically on both left and right sides
- [x] Visual tile positions match collision grid boundaries exactly
- [x] Maintain existing collision detection logic without breaking spawning, snapping, etc.
- [x] Use `bevy_ecs_tilemap`'s built-in anchor system rather than manual offsets
- [x] Document the tilemap coordinate system for future maintainers

## Acceptance Criteria
- [x] Player walking left off a platform falls when one foot is still on (symmetric with right side)
- [x] Player walking right off a platform falls when one foot is still on (already worked)
- [x] No visual/collision misalignment visible during gameplay
- [x] Jumping and ground snapping continue to work correctly
- [x] Coordinate system documented in `/docs/bevy-coordinate-system.md`

---

## Implementation Plan

### Approach
Use `TilemapAnchor::BottomLeft` to align the tilemap's bottom-left corner with the transform position, matching what `world_to_tile_coords` expects. Additionally, change `is_grounded` from requiring both feet on solid ground (`&&`) to requiring at least one foot (`||`), allowing the player to stand on platform edges.

### Key Changes

1. **Import TilemapAnchor** (`src/level/mod.rs`)
   - Add `anchor::TilemapAnchor` to imports

2. **Set anchor to BottomLeft** (`src/level/mod.rs`)
   - Add `anchor: TilemapAnchor::BottomLeft` to `TilemapBundle`
   - Remove manual `+8.0` transform offset (if present from previous attempts)
   - Tilemap transform remains at `(TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y)`

3. **Change grounding logic** (`src/player/movement.rs`)
   - Change `is_grounded` from `&&` to `||` (OR instead of AND)
   - This allows player to stand on edges with one foot on solid ground

4. **Document coordinate system** (`docs/bevy-coordinate-system.md`)
   - Add section explaining `bevy_ecs_tilemap` coordinate system
   - Document why we use `TilemapAnchor::BottomLeft`
   - Explain anchor options and trade-offs
   - Document built-in `TilePos::from_world_pos` API for future use

### Tasks
- [x] Import `TilemapAnchor` in `src/level/mod.rs`
- [x] Add `anchor: TilemapAnchor::BottomLeft` to tilemap setup
- [x] Change `is_grounded` to use `||` instead of `&&`
- [x] Test left platform edge collision
- [x] Test right platform edge collision
- [x] Verify jumping still works correctly
- [x] Verify ground snapping still works correctly
- [x] Document tilemap coordinate system in `docs/bevy-coordinate-system.md`
- [x] Create this spec

### Technical Details

#### TilemapAnchor::BottomLeft Effect

With `TilemapAnchor::BottomLeft`, the anchor offset is `(-min.x, -min.y)` where `min` is the AABB minimum of the tilemap. For a square tilemap, tile (0,0) has its center at (0,0) in unanchored space, so `min = (-grid_size/2, -grid_size/2) = (-8, -8)`, giving offset `(8, 8)`.

This shifts tile rendering by +8 pixels in both axes, placing tile (0,0)'s **bottom-left corner** at the tilemap transform position instead of its center.

**Result:**
```
Tilemap transform: (-160, -120)

Tile 0 bottom-left corner: (-160, -120)
Tile 0 top-right corner: (-160 + 16, -120 + 16) = (-144, -104)

Tile 12 bottom-left corner: (-160 + 12*16, -120 + 5*16) = (32, -40)
Tile 12 top-right corner: (48, -24)

Collision grid for tile 12: X ∈ [32, 48), Y ∈ [-40, -24)
Visual rendering for tile 12: X ∈ [32, 48], Y ∈ [-40, -24]

Perfect alignment! ✓
```

#### is_grounded Change

```rust
// Before (both feet must be on solid ground)
feet_tiles.left.map(&is_solid_tile).unwrap_or(false)
    && feet_tiles.right.map(&is_solid_tile).unwrap_or(false)

// After (at least one foot on solid ground)
feet_tiles.left.map(&is_solid_tile).unwrap_or(false)
    || feet_tiles.right.map(&is_solid_tile).unwrap_or(false)
```

This allows the player to stand on platform edges with one foot hanging off, which is standard platformer behavior and makes both edges symmetric.

#### Why Not Change world_to_tile_coords?

An alternative approach would be to add `+0.5` to `world_to_tile_coords` to account for center-anchoring (matching `bevy_ecs_tilemap`'s `TilePos::from_world_pos` logic). However, this breaks the relationship between tile indices and `LEVEL_DATA` array indices:

```rust
// With +0.5 offset
tile_y = floor((-25 + 120) / 16 + 0.5) = floor(6.4375) = 6
array_y = 14 - 6 = 8  → LEVEL_DATA[8] (row 8 from top)

// But the ground is at array row 12 (tile_y=2 in current mapping)
```

The `LEVEL_DATA` array was designed for corner-aligned mapping. Changing the mapping would require updating spawn positions, snap calculations, and verifying all tile-based logic. Using `TilemapAnchor::BottomLeft` changes only the rendering, keeping all logic unchanged.

### Notes

#### Alternative: Use TilePos::from_world_pos

`bevy_ecs_tilemap` provides built-in coordinate conversion APIs that handle all tilemap types (square, hexagonal, isometric) and anchoring automatically:

```rust
let tile_pos = TilePos::from_world_pos(
    &world_pos,
    &map_size,
    &grid_size,
    &tile_size,
    &map_type,
    &anchor,
);
```

**Trade-offs:**
- ✅ Handles all tilemap types and dynamic configurations
- ✅ Guaranteed to match rendering
- ❌ Requires passing many tilemap parameters to movement systems
- ❌ Couples movement logic to `bevy_ecs_tilemap` types
- ❌ Harder to unit test with mocks

For a simple square tilemap project, our custom `world_to_tile_coords` is simpler and more testable. Consider migrating to built-in APIs if the project needs non-square tilemaps or dynamic configurations in the future.

#### Debug Investigation Process

The bug was diagnosed through:
1. Observing asymmetric edge behavior in gameplay
2. Adding debug prints showing player position, feet positions, and tile lookups
3. Manually calculating expected tile boundaries
4. Discovering 8-pixel offset between visual tiles and collision grid
5. Researching `bevy_ecs_tilemap` source code to understand anchor system
6. Testing with manual `+8.0` transform offset (worked but was hacky)
7. Discovering `TilemapAnchor::BottomLeft` as the proper solution

## Related Files
- `src/level/mod.rs` - Tilemap setup with anchor
- `src/level/tile.rs` - `world_to_tile_coords` function
- `src/player/movement.rs` - `is_grounded` and collision logic
- `docs/bevy-coordinate-system.md` - Coordinate system documentation
- `vendor/bevy_ecs_tilemap/` - Tilemap crate source code for reference

## Commits
- c70951d: Initial fix attempt with `+8.0` transform offset and `||` change
- e37468c: Proper fix using `TilemapAnchor::BottomLeft`
- b047482: Documentation updates

## Future Considerations
- If the project needs to support hexagonal or isometric tilemaps, migrate to `TilePos::from_world_pos` and `TilePos::center_in_world` built-in APIs
- If tilemap size, grid size, or anchor becomes configurable at runtime, refactor collision systems to query tilemap components rather than using constants
- Consider adding unit tests for `world_to_tile_coords` edge cases
