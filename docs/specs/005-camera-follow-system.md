# Camera Follow System

## Overview
Implement smooth camera follow that tracks the player position. The camera smoothly interpolates (lerps) toward the player on both X and Y axes, creating a professional camera feel. The camera provides some "lag" behind player movement for visual polish.

## Requirements
- Camera follows player position smoothly
- Camera tracks player on both X and Y axes
- Camera movement uses lerp interpolation (not instant)
- Camera smoothness is configurable via lerp speed constant
- Camera maintains proper z-position (looking at the game plane)
- Frame-rate independent camera movement

## Acceptance Criteria
- [ ] Camera smoothly follows player horizontally
- [ ] Camera smoothly follows player vertically  
- [ ] Camera movement is smooth, not instant or jerky
- [ ] Camera lag feels natural (not too slow, not too fast)
- [ ] Player remains visible on screen during movement
- [ ] Frame-rate independent camera updates

---

## Implementation Plan

### Approach
Create a camera follow system that queries both camera and player transforms. Calculate the target position (player position) and use linear interpolation (lerp) to smoothly move the camera toward that target. The lerp speed constant controls how quickly the camera catches up - higher values make camera more responsive, lower values add more lag. Use delta time to ensure smooth movement regardless of frame rate.

### Components & Systems
**Systems:**
- `camera_follow()` - Update system that smoothly moves camera toward player
- Queries player Transform
- Queries camera Transform  
- Lerps camera position toward player position
- Uses lerp speed and delta time

**Resources:**
- `Time` - For delta time (frame-independent interpolation)

**Constants:**
- Camera lerp speed (e.g., 5.0 - higher = more responsive, lower = more lag)

**Queries:**
- Player: `Query<&Transform, With<Player>>`
- Camera: `Query<&mut Transform, With<Camera2d>>`

### Tasks
- [ ] Define CAMERA_LERP_SPEED constant (suggested: 5.0)
- [ ] Implement camera_follow system
- [ ] Query player transform
- [ ] Query camera transform (mutable)
- [ ] Calculate lerp factor: lerp_speed * delta_time
- [ ] Lerp camera X toward player X
- [ ] Lerp camera Y toward player Y
- [ ] Keep camera Z unchanged (viewing plane)
- [ ] Add camera_follow to Update schedule
- [ ] Test: Run and verify camera follows player smoothly
- [ ] Test: Verify camera catches up when player stops
- [ ] Test: Tune lerp speed if needed (too fast/slow)

### Notes
- Lerp formula: `current + (target - current) * factor`
- Or use Bevy's built-in lerp: `current.lerp(target, factor)`
- Lerp factor = `CAMERA_LERP_SPEED * time.delta_secs()`
- Suggested lerp speed range: 3.0 (lazy) to 8.0 (snappy), 5.0 is good default
- Camera Z should remain constant (e.g., 999.0 or whatever default Camera2d uses)
- Both queries need `Without` filters to avoid conflicts: 
  - Player: `Query<&Transform, (With<Player>, Without<Camera2d>)>`
  - Camera: `Query<&mut Transform, With<Camera2d>>`
- This removes the screen boundary restriction from spec 004 - player can now move freely
- Consider removing SCREEN_BOUND clamping from player_movement system once camera follow is working
- Camera naturally keeps player centered, so explicit bounds become unnecessary
