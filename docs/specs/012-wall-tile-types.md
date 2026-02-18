# Spec 012 — Level Bounds and Wall Tile Collision

## Status: Complete

## Overview

The player can currently walk past the left/right edges of the tilemap and fall into the void.
This spec adds invisible boundary walls at the level edges and a kill zone below the floor.
It also documents that wall tile collision works via existing Avian2D collider shapes in the tileset.

## Requirements

1. **Invisible boundary walls**: The player cannot walk off the left or right edge of the level.
2. **Bottom kill zone**: If the player falls below the level floor, they are teleported back to spawn.
   No damage/lives system — just a safety net.
3. **Wall tile collision**: Tiles with vertical surfaces (walls) already block the player because
   any tile with an `<objectgroup>` in `tileset.tsx` generates an Avian2D collider.
   Verify wall tiles have collision shapes in the TSX; add them if missing.

## Acceptance Criteria

- [ ] Player cannot walk off the left edge of the tilemap.
- [ ] Player cannot walk off the right edge of the tilemap.
- [ ] Player falling below `TILEMAP_OFFSET_Y - 64.0` is teleported back to spawn with velocity reset.
- [ ] Wall tiles (vertical tile surfaces) block the player from passing through them.

## Implementation Plan

### Boundary Walls (`src/level/mod.rs`)

Add `spawn_level_bounds` startup system that spawns two thin static colliders:

- **Left wall**: `x = TILEMAP_OFFSET_X`, `y = 0`, `width = 1px`, `height = level_height_px`
- **Right wall**: `x = TILEMAP_OFFSET_X + level_width_px`, `y = 0`, same dimensions

Each entity: `(Collider::rectangle(1.0, level_height_px), RigidBody::Static, Transform::from_xyz(x, 0.0, 0.0))`

### Kill Zone (`src/player/movement.rs`)

Add `kill_zone` system that runs each frame:

```rust
if position.y < TILEMAP_OFFSET_Y - 64.0 {
    *position = Position::from_xy(spawn_x, spawn_y);
    *velocity = LinearVelocity::ZERO;
}
```

Uses `Position` (not `Transform`) because Avian2D owns the transform for dynamic bodies.

### Wall Tiles (`assets/tileset.tsx`)

Any tile with an `<objectgroup>` collision shape automatically gets an Avian2D collider via
`TiledPhysicsPlugin`. Verify wall tiles have shapes; add a full-tile rectangle if missing.

## Task Checklist

- [x] Rewrite spec file with new scope
- [x] Add `spawn_level_bounds` system to `src/level/mod.rs`
- [x] Add `kill_zone` system to `src/player/movement.rs`
- [x] Register both systems in `src/main.rs`
- [x] Verify wall tiles in `assets/tileset.tsx` have collision shapes
- [x] Run game and confirm left/right walls block the player
- [x] Run game and confirm kill zone teleports player back to spawn
