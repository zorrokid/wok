# Tile-Based Floor Collision

## Overview
Replace the fixed ground level check with proper tile-based collision detection. The player checks if solid tiles exist beneath their feet and stands on them. If no tiles are present, the player falls. This enables standing on platforms at different heights and creates proper platformer collision.

## Requirements
- Player detects solid tiles beneath their feet
- Player stands on solid tiles (is grounded)
- Player falls when no solid tiles beneath
- Collision checks player's bottom position (feet)
- Works with existing tilemap (type 0=empty, type 1=solid)
- Player can land on platforms at any height
- Smooth collision response (no jittering)

## Acceptance Criteria
- [ ] Player stands on solid tiles (type 1)
- [ ] Player falls through empty space (type 0 or no tile)
- [ ] Player can stand on ground floor (bottom tiles)
- [ ] Player can land on floating platforms
- [ ] Collision detection works at player's feet position
- [ ] No falling through solid tiles
- [ ] No getting stuck in tiles
- [ ] Jump mechanic still works with tile collision

---

## Implementation Plan

### Approach
Convert player's world position to tilemap tile coordinates. Query the tilemap storage to check if a solid tile exists at the calculated position beneath the player's feet. If a solid tile is found, set the player as grounded and prevent falling through by clamping Y position to tile surface. If no solid tile, player falls with gravity. Replace the fixed GROUND_LEVEL constant with dynamic tile-based detection.

### Components & Systems
**Systems:**
- Modify `player_movement()` to use tile-based collision instead of fixed ground level
- Add tile position calculation (world pos → tile coords)
- Query tilemap for tiles at calculated position
- Check tile type for solidity

**Resources/Queries:**
- Query tilemap entities and TileStorage
- Access LEVEL_DATA for tile type checking
- Calculate tile coordinates from player transform

**Helper Functions:**
- `world_to_tile_coords()` - Convert world position to tile coordinates
- `is_solid_tile()` - Check if tile type is solid (type 1)
- `get_tile_at()` - Query tile at specific coordinates

### Tasks
- [x] Add function to convert world position to tile coordinates
- [x] Add function to check if tile type is solid
- [x] Query tilemap storage in player_movement system
- [x] Calculate tile position beneath player feet (Y - half sprite height)
- [x] Check if solid tile exists at that position
- [x] Update is_grounded based on tile presence (not fixed Y)
- [x] Clamp player Y position to top of solid tile when grounded
- [x] Remove GROUND_LEVEL constant (no longer needed)
- [x] Test: Run and verify player stands on ground tiles
- [x] Test: Verify player stands on floating platforms
- [x] Test: Verify player falls through empty space
- [x] Test: Verify jumping still works with tile collision

### Notes
- Tile coordinate calculation: 
  - `tile_x = ((world_x - tilemap_offset_x) / TILE_SIZE).floor()`
  - `tile_y = ((world_y - tilemap_offset_y) / TILE_SIZE).floor()`
- Player feet position: `player_y - SPRITE_HEIGHT/2` (8 pixels for 16x16 sprite)
- **Ground detection**: Check 1 pixel BELOW feet (`feet_y - 1.0`) to detect tile surface, since player standing on tile has feet at boundary
- Tilemap offset: `-(LEVEL_WIDTH * 16.0) / 2.0` for X, `-(LEVEL_HEIGHT * 16.0) / 2.0` for Y
- When grounded on tile, clamp Y to: `tilemap_offset_y + (tile_y + 1) * TILE_SIZE + SPRITE_HEIGHT/2`
- Current tile types: 0 = empty/air, 1 = solid platform
- Query pattern: Direct LEVEL_DATA access (no TileStorage component needed)
- Y-axis inversion: Array index = `(LEVEL_HEIGHT - 1) - tile_y`

### Implementation Issues & Solutions
**Bug 1: Player falls through ground**
- Problem: Checking at exact feet position missed tiles when standing on top
- Solution: Check 1 pixel below feet (`feet_y - 1.0`) to detect ground tile

**Bug 2: Gravity applied when grounded**
- Problem: Gravity applied every frame, even when standing on ground, causing jittery falling
- Solution: Only apply gravity when `!is_grounded || velocity.y > 0.0` (in air or jumping up)
- Set `velocity.y = 0.0` only when grounded with non-positive velocity

**Bug 3: Jump not working**
- Problem: Setting velocity to 0 when grounded was canceling jump velocity
- Solution: Allow gravity/movement when velocity.y > 0 (jumping), only zero out falling velocity

### Future Expansions (Not in this spec)
- **More tile types**: Add types 2=ladder, 3=spikes, 4=one-way platform, etc.
- **Tile property system**: Replace type checks with property queries (is_solid, is_climbable, damages_player)
- **One-way platforms**: Tiles player can jump through from below but land on from above
- **Slope tiles**: Angled surfaces for smoother terrain
- **Wall collision**: Detect tiles to left/right to prevent walking through walls
- **Ceiling collision**: Detect tiles above player to stop upward movement
- **Advanced collision shapes**: Per-tile collision boxes for more precise detection
