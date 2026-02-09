# Player Horizontal Movement

## Overview
Implement basic horizontal player movement controlled by left and right cursor keys. The player moves directly left or right when keys are pressed, with movement restricted to the visible screen bounds to prevent moving off-screen. Camera is static and does not follow the player.

## Requirements
- Player moves left when left arrow key is pressed
- Player moves right when right arrow key is pressed
- Player cannot move outside the visible screen boundaries
- Movement speed is constant and responsive
- Camera remains static (does not follow player)
- Player stays visible on screen at all times

## Acceptance Criteria
- [x] Pressing left arrow key moves player left
- [x] Pressing right arrow key moves player right
- [x] Player stops at left screen edge (removed - now camera follows)
- [x] Player stops at right screen edge (removed - now camera follows)
- [x] Movement is smooth and responsive
- [x] Player sprite remains fully visible (not clipped at edges)

---

## Implementation Plan

### Approach
Create a movement system that reads keyboard input and directly modifies the player's Transform position. Calculate screen boundaries based on window size and tilemap offset, then clamp player position to stay within visible bounds. Use a constant movement speed for predictable, arcade-style control.

### Components & Systems
**Systems:**
- `player_movement()` - Update system that handles keyboard input and position updates
- Reads left/right arrow key state
- Calculates new position based on speed and delta time
- Clamps position to screen boundaries

**Resources:**
- `Time` - For delta time (frame-independent movement)
- `ButtonInput<KeyCode>` - For keyboard state

**Constants:**
- Movement speed (e.g., 100.0 pixels per second)
- Screen bounds calculation based on window size and tilemap position

### Tasks
- [x] Define PLAYER_SPEED constant (e.g., 100.0)
- [x] Implement player_movement system
- [x] Read left/right arrow key input
- [x] Calculate new position: current + (direction * speed * delta_time)
- [x] Calculate screen boundaries (left edge, right edge) - removed in spec 005
- [x] Clamp player x position to boundaries - removed in spec 005
- [x] Apply clamped position to Transform
- [x] Add player_movement to Update schedule
- [x] Test: Run and verify left arrow moves player left
- [x] Test: Run and verify right arrow moves player right
- [x] Test: Verify player stops at screen edges - removed in spec 005
- [x] Test: Verify no jittery/stuttering movement

### Notes
- Screen bounds calculation: Based on 800x600 window with centered tilemap
  - Tilemap is 20 tiles * 16 pixels = 320 pixels wide
  - Tilemap center offset: -160 pixels from world origin
  - Left boundary: -160 + 8 (half player width) = -152
  - Right boundary: 160 - 8 (half player width) = 152
- Use `Time` resource for delta_time to ensure frame-rate independent movement
- Direct position modification (not velocity-based) for immediate response
- Query: `Query<&mut Transform, With<Player>>`
- Movement speed of 100.0 pixels/second is a good starting point, can be tuned
- Consider adding separate constants for screen bounds to make them easy to adjust
- Player sprite is 16x16, so add 8 pixel padding from edges to keep fully visible
- This is foundation for physics later - velocity component can replace direct movement
