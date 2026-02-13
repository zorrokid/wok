# Smooth Player Movement

## Overview
Replace direct position-based player movement with smooth acceleration and deceleration. When the player starts moving, they gradually accelerate to full speed. When stopping, they gradually decelerate instead of stopping instantly. This creates more natural, weighty character movement.

## Requirements
- Player accelerates smoothly when starting to move
- Player decelerates smoothly when stopping
- Movement feels responsive but not instant
- Acceleration and deceleration rates are configurable
- Movement still responds to left/right arrow keys
- Frame-rate independent movement

## Acceptance Criteria
- [x] Player gradually speeds up when starting to move
- [x] Player gradually slows down when stopping
- [x] No instant speed changes (smooth transitions)
- [x] Movement feels natural and responsive
- [x] Left/right controls still work correctly
- [x] Movement is frame-rate independent

---

## Implementation Plan

### Approach
Add a Velocity component to the player to track current movement speed. Instead of directly modifying position based on input, apply acceleration forces that modify velocity. Each frame, apply acceleration toward the target speed (based on input), then apply velocity to position. When no input, apply deceleration to reduce velocity to zero. This creates smooth ramp-up and ramp-down movement.

### Components & Systems
**Components:**
- `Velocity` - Stores current movement velocity as Vec2 (x for horizontal, y for vertical)
  - For this spec, only x component is used
  - y component reserved for future jumping/gravity (spec 007+)

**Systems:**
- Modify `player_movement()` (in `src/player/movement.rs`) - Apply acceleration/deceleration to velocity.x, then velocity to position

**Constants (in `src/player/mod.rs`):**
- `PLAYER_ACCELERATION` - How fast player speeds up horizontally (800.0 pixels/sec²)
- `PLAYER_DECELERATION` - How fast player slows down horizontally (1200.0 pixels/sec²)
- `PLAYER_MAX_SPEED` - Maximum horizontal movement speed (100.0 pixels/sec)

**Functions:**
- `apply_horizontal_acceleration()` - Pure function calculating new horizontal velocity with acceleration/deceleration
- `get_target_velocity_x()` - Determines target velocity based on input

### Tasks
- [x] Create Velocity component as Vec2 (x and y components)
- [x] Add Velocity component to player spawn (initialized to Vec2::ZERO)
- [x] Define movement constants (acceleration, deceleration, max speed)
- [x] Modify player_movement system to use velocity
- [x] Calculate target velocity.x based on input (0.0 or ±PLAYER_MAX_SPEED)
- [x] Apply acceleration toward target when moving
- [x] Apply deceleration toward zero when no input
- [x] Clamp velocity.x to max speed
- [x] Apply velocity.x to transform position (leave velocity.y unused for now)
- [x] Test: Run and verify smooth acceleration
- [x] Test: Run and verify smooth deceleration (fixed overshoot bug)
- [x] Test: Tune acceleration/deceleration for feel

### Notes
- Velocity component uses Vec2: `pub struct Velocity(pub Vec2)`
  - Vec2.x: horizontal velocity (used in this spec)
  - Vec2.y: vertical velocity (reserved for future jumping/gravity)
- Initialize velocity as Vec2::ZERO when spawning player
- Acceleration formula: `velocity.x += acceleration * delta_time`
- When input pressed: accelerate velocity.x toward ±max speed
- When no input: decelerate velocity.x toward zero
- Apply position: `transform.translation.x += velocity.x * delta_time`
- Leave velocity.y unchanged (will be used in future specs for jumping/gravity)
- Suggested values:
  - PLAYER_MAX_SPEED: 100.0 (same as current speed)
  - PLAYER_ACCELERATION: 800.0 (reaches max in ~0.125 seconds)
  - PLAYER_DECELERATION: 1200.0 (stops in ~0.08 seconds, faster than acceleration)
- Higher deceleration than acceleration gives responsive feel
- Query becomes: `Query<(&mut Transform, &mut Velocity), With<Player>>`
- Velocity.x should be clamped to [-MAX_SPEED, MAX_SPEED]
- This replaces direct position modification from spec 004
- Foundation for adding gravity and jumping in future specs (will use velocity.y)
