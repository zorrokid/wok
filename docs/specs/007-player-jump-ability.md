# Player Jump Ability

## Overview
Implement basic jump mechanic for the player. When pressing the Z key, the player jumps vertically with an upward velocity impulse. Gravity constantly pulls the player downward. For this spec, the player jumps from and lands on a fixed ground level (simple Y position check). Horizontal velocity is maintained during jumps (can strafe in air).

## Requirements
- Player jumps when Z key is pressed
- Jump applies upward velocity to velocity.y component
- Gravity constantly pulls player downward
- Player can only jump when on the ground
- Player maintains horizontal velocity while in air
- Player lands back on ground after jumping
- Frame-rate independent physics

## Acceptance Criteria
- [x] Pressing Z key makes player jump upward
- [x] Player cannot jump while already in air (no double jump yet)
- [x] Gravity pulls player back down
- [x] Player lands on ground level
- [x] Can move left/right while jumping (air control)
- [x] Jump height is consistent and feels good
- [x] No jumping through ground or getting stuck

---

## Implementation Plan

### Approach
Add gravity that constantly applies downward acceleration to velocity.y. When Z key is pressed and player is on ground, apply an upward velocity impulse to velocity.y. Each frame, apply gravity to velocity, then apply velocity to position. Detect ground by checking if player Y position is at or below ground level. When on ground, clamp Y position and reset vertical velocity to prevent falling through.

### Components & Systems
**Systems:**
- Modify `player_movement()` (in `src/player/movement.rs`) to handle jump input and apply gravity
- Check if player is on ground using tile-based collision detection
- Apply jump impulse when Z pressed and grounded
- Apply gravity to velocity.y every frame
- Apply velocity.y to position
- Ground collision using tile-based snapping

**Constants (in `src/player/mod.rs`):**
- `JUMP_VELOCITY` - Initial upward velocity when jumping (300.0 pixels/sec)
- `GRAVITY` - Downward acceleration (980.0 pixels/sec²)

**Functions:**
- `get_velocity_y()` - Pure function calculating vertical velocity with jump and gravity
- `is_grounded()` - Checks tile collision beneath player feet
- `ground_snap_y()` - Calculates Y position to snap player to tile surface

### Tasks
- [x] Define jump and gravity constants
- [x] Calculate GROUND_LEVEL constant from player spawn position
- [x] Add jump input detection (Z key pressed)
- [x] Add ground detection (check if player.y <= GROUND_LEVEL)
- [x] Apply jump impulse to velocity.y when Z pressed and grounded
- [x] Apply gravity to velocity.y every frame
- [x] Apply velocity.y to transform position
- [x] Clamp player Y position to ground level
- [x] Reset velocity.y to zero when landing on ground
- [x] Test: Run and verify player jumps when Z pressed
- [x] Test: Verify player falls back down with gravity
- [x] Test: Verify player can't jump in air
- [x] Test: Verify player can move left/right while jumping

### Notes
- Jump velocity of 300.0 with gravity 980.0 gives ~0.9 tile height jump
- Gravity constant: Earth gravity is ~980 pixels/sec² (good starting point)
- Ground detection: Uses tile-based collision (see spec 008-tile-based-floor-collision.md)
  - Checks for solid tiles beneath both feet
  - Implementation in `is_grounded()` function
- When on ground: snap to tile surface via `ground_snap_y()` and set `velocity.0.y = 0.0`
- Jump logic extracted to pure function `get_velocity_y()` for testability
- Gravity only applied when not grounded OR when jumping up (velocity.y > 0.0)
  - This prevents jittery behavior when standing on ground
- Jump only applies when grounded: prevents mid-air jumping
- Use `keyboard.just_pressed()` for jump (not `pressed()`) for single jump per press
- Horizontal velocity (velocity.x) is independent - player can move in air
- This creates "floaty" platformer feel - can tune gravity/jump strength for different feels
- Future specs will add:
  - Collision with tilemap (not just fixed ground level)
  - Double jump ability
  - Variable jump height (hold button for higher jump)
  - Coyote time and jump buffering
