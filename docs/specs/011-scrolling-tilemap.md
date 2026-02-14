# 011: Scrolling Tilemap with Camera Bounds

## Overview
Expand the tilemap beyond the viewport and implement camera bounds so only the visible portion of the level is shown. The camera follows the player horizontally, revealing new areas as they explore. This creates the classic side-scrolling platformer feel where the level extends beyond what's initially visible.

## Current State
- Tilemap is 20x15 tiles (320x240 pixels)
- Camera follows player smoothly on both X and Y axes (spec 005)
- No camera bounds - camera can follow player anywhere, showing empty space beyond tilemap edges
- Level is small enough to fit mostly on screen

## Requirements

### Phase 1: Horizontal Scrolling (Initial Implementation)
- Tilemap width significantly larger than viewport (e.g., 100+ tiles wide)
- Camera follows player horizontally within level bounds
- Camera stops at level edges (doesn't show empty space beyond tilemap)
- Camera smoothly tracks player using existing lerp system
- Vertical camera behavior remains unchanged (follows player on Y axis)

### Phase 2: Vertical Scrolling (Future)
- Implement vertical camera bounds similar to horizontal
- Consider different vertical camera strategies:
  - Follow player freely (current behavior)
  - Dead zone (camera only moves when player reaches zone edges)
  - Room-based (camera snaps to room boundaries for metroidvania-style rooms)
- Vertical bounds prevent showing empty space above/below tilemap

## Acceptance Criteria

### Phase 1 (Horizontal)
- [ ] Tilemap width increased to at least 100 tiles (1600 pixels)
- [ ] Camera horizontal position clamped to level bounds
- [ ] Camera stops at left edge when player is near start
- [ ] Camera stops at right edge when player is near end
- [ ] Camera smoothly follows player in the middle of the level
- [ ] No visual glitches or empty space shown beyond tilemap edges
- [ ] Player remains visible on screen at all camera positions
- [ ] Existing smooth camera follow preserved (lerp behavior)

### Phase 2 (Vertical - Future)
- [ ] Vertical camera bounds implemented
- [ ] Vertical camera strategy chosen and documented
- [ ] Camera stops at top/bottom level edges
- [ ] Smooth transitions when camera hits vertical bounds

---

## Implementation Plan - Phase 1: Horizontal Scrolling

### Approach
Add horizontal camera bounds that constrain the camera X position based on viewport size and level dimensions. Calculate the valid camera range (minimum and maximum X positions that keep the viewport within level bounds) and clamp the camera position after the lerp calculation. The existing smooth follow behavior is preserved, but the camera simply stops moving when it reaches the edges.

### Level Expansion
Increase `LEVEL_WIDTH_IN_TILES` from 20 to 100+ tiles. Design a longer horizontal level layout in `LEVEL_DATA` with platforms, gaps, and obstacles that encourage exploration. The increased width makes horizontal scrolling necessary.

### Camera Bounds Calculation

```rust
// Constants (example for 800x600 viewport)
const VIEWPORT_WIDTH: f32 = 800.0;
const LEVEL_WIDTH_IN_TILES: u32 = 100;
const TILE_SIZE: f32 = 16.0;

// Level bounds in world coordinates
let level_width = LEVEL_WIDTH_IN_TILES as f32 * TILE_SIZE;
let level_left = TILEMAP_OFFSET_X;  // -800.0 for 100 tiles centered
let level_right = TILEMAP_OFFSET_X + level_width;

// Camera bounds (keep viewport within level)
let camera_min_x = level_left + VIEWPORT_WIDTH / 2.0;
let camera_max_x = level_right - VIEWPORT_WIDTH / 2.0;

// After lerp, clamp camera position
camera_transform.translation.x = camera_transform.translation.x.clamp(camera_min_x, camera_max_x);
```

### Components & Systems

**Modified Systems:**
- `camera_follow()` - Add horizontal bounds clamping after lerp
  - Calculate camera bounds based on level and viewport dimensions
  - Clamp camera X after lerp calculation
  - Keep existing Y axis behavior unchanged

**New Constants:**
- `VIEWPORT_WIDTH` - Game window width (e.g., 800.0)
- `VIEWPORT_HEIGHT` - Game window height (e.g., 600.0) - for future vertical bounds

**Modified Constants:**
- `LEVEL_WIDTH_IN_TILES` - Increase from 20 to 100 (or more)
- `LEVEL_DATA` - Expand to match new width

### Tasks

#### Level Design
- [ ] Increase `LEVEL_WIDTH_IN_TILES` to 100 tiles minimum
- [ ] Expand `LEVEL_DATA` array width to match
- [ ] Design horizontal level layout with:
  - [ ] Starting area with safe platforms
  - [ ] Middle section with varied platform arrangements
  - [ ] Ending area or goal
  - [ ] Gaps and obstacles to test jumping
- [ ] Keep `LEVEL_HEIGHT_IN_TILES` at 15 for now

#### Camera Bounds Implementation
- [ ] Add `VIEWPORT_WIDTH` and `VIEWPORT_HEIGHT` constants to camera module
- [ ] Calculate level bounds from tilemap dimensions
- [ ] Calculate camera min/max X positions
- [ ] Add horizontal clamp after lerp in `camera_follow()`
- [ ] Test: Camera stops at left edge
- [ ] Test: Camera stops at right edge
- [ ] Test: Camera follows smoothly in middle
- [ ] Test: Player walks from start to end of level

#### Polish
- [ ] Verify no visual glitches at edges
- [ ] Ensure player remains centered when camera is not at bounds
- [ ] Test that existing player movement and jumping work with larger level
- [ ] Update spawn position if needed for better level start

### Technical Details

#### Viewport Size
For now, use hardcoded viewport dimensions matching the window size. In a future polish pass, query the window size dynamically:

```rust
// Future enhancement
fn get_viewport_size(windows: Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    if let Ok(window) = windows.single() {
        Vec2::new(window.width(), window.height())
    } else {
        Vec2::new(800.0, 600.0) // fallback
    }
}
```

#### Camera Bounds Formula

The camera bounds ensure that the viewport edges don't extend beyond the level:

```
Viewport shows world positions: [camera_x - viewport_width/2, camera_x + viewport_width/2]

For viewport to stay within level [level_left, level_right]:
  camera_x - viewport_width/2 >= level_left
  camera_x + viewport_width/2 <= level_right

Therefore:
  camera_min_x = level_left + viewport_width/2
  camera_max_x = level_right - viewport_width/2
```

#### Edge Cases
- **Small level**: If `level_width < viewport_width`, set `camera_min_x = camera_max_x = level_center_x` to center the level
- **Player near edge**: Player can walk to level edges; camera stops but player keeps moving
- **Lerp at bounds**: Lerp target might be beyond bounds, but clamp ensures camera doesn't exceed limits

### Notes

#### Why Separate Horizontal and Vertical?
Horizontal scrolling is standard for side-scrolling platformers, but vertical camera behavior varies by game design:
- **Free follow**: Smooth for exploration-heavy games
- **Dead zone**: Reduces motion sickness, feels more stable
- **Room-based**: Best for metroidvania with discrete rooms

Starting with horizontal-only lets us test the scrolling mechanics before committing to a vertical strategy.

#### Alternative Approaches Considered

**1. Dead Zone (Horizontal)**
Camera only moves when player reaches edges of a central "dead zone":
```
[---- camera view ----]
   [-- dead zone --]
   
Camera stays still while player is in dead zone.
Camera moves when player exits dead zone.
```
**Pros**: Reduces camera motion, feels stable  
**Cons**: More complex, less smooth

**Decision**: Use smooth follow for simplicity. Dead zone can be added later if needed.

**2. Predictive Camera**
Camera leads player based on velocity:
```
camera_target_x = player_x + player_velocity_x * LOOK_AHEAD_FACTOR
```
**Pros**: Shows more of where player is going  
**Cons**: Can feel floaty or disorienting

**Decision**: Stick with direct follow. Can experiment with look-ahead later.

#### Testing Strategy
1. **Unit Tests**: Test camera bounds calculation with various level sizes
2. **Manual Testing**: 
   - Walk from level start to end
   - Jump at level edges
   - Verify camera smoothness in middle
   - Check edge visuals

#### Performance Considerations
- Camera bounds calculation is cheap (a few floating-point ops per frame)
- Larger tilemap doesn't affect performance (`bevy_ecs_tilemap` handles frustum culling automatically)
- Only visible tiles are rendered, so a 100-tile level performs similar to a 20-tile level

## Phase 2 Planning: Vertical Scrolling (Future)

### Vertical Camera Strategies to Evaluate

**Option A: Free Follow (Current Behavior)**
- Camera follows player on Y axis with same lerp smoothness
- Add vertical bounds similar to horizontal
- Simple and consistent with horizontal

**Option B: Dead Zone**
- Define vertical dead zone (e.g., center 40% of viewport height)
- Camera only moves when player exits dead zone
- Reduces vertical camera motion during jumping

**Option C: Room-Based (Metroidvania)**
- Level divided into rectangular rooms
- Camera snaps to show entire room when player enters
- Vertical scrolling only between rooms

### Recommendation
Start with **Option A (Free Follow)** for consistency, then evaluate feel during playtesting. Option B or C can be implemented later if vertical motion feels excessive.

### Vertical Bounds Implementation (Future)

Similar to horizontal bounds:
```rust
let level_height = LEVEL_HEIGHT_IN_TILES as f32 * TILE_SIZE;
let level_bottom = TILEMAP_OFFSET_Y;
let level_top = TILEMAP_OFFSET_Y + level_height;

let camera_min_y = level_bottom + VIEWPORT_HEIGHT / 2.0;
let camera_max_y = level_top - VIEWPORT_HEIGHT / 2.0;

camera_transform.translation.y = camera_transform.translation.y.clamp(camera_min_y, camera_max_y);
```

## Related Specs
- **005-camera-follow-system.md** - Existing smooth camera follow implementation
- **002-basic-tilemap-system.md** - Tilemap setup and rendering
- **008-tile-based-floor-collision.md** - Collision detection that works with expanded levels

## Related Files
- `src/camera.rs` - Camera follow system (needs bounds added)
- `src/level/mod.rs` - Level dimensions and data (needs expansion)
- `src/level/tile.rs` - Tile constants and utilities

## Future Enhancements
- Dynamic viewport size detection from window
- Configurable camera strategies (dead zone, room-based)
- Smooth camera transitions between rooms/areas
- Camera shake effects for game feel
- Parallax scrolling background layers
- Mini-map showing full level layout
