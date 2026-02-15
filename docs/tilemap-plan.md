# Plan: Adopt Tiled/TMJ + Scrolling Tilemap

## Problem
The current level is hardcoded as a 20x15 Rust const array (`LEVEL_DATA`). This is not scalable for larger levels, hard to edit visually, and collision detection is custom-built. We want to:
1. Switch to the Tiled editor for level design (TMJ format)
2. Use `bevy_ecs_tiled` plugin for map loading
3. Use Avian2D physics for collision instead of custom tile-based collision
4. Implement horizontal scrolling with camera bounds (spec 011)

## Approach
Replace the manual tilemap setup and custom collision system with `bevy_ecs_tiled` + Avian2D. This is a significant refactor that replaces `level/mod.rs` setup, `level/tile.rs` collision helpers, and `player/movement.rs` collision logic with physics-based collision using Avian2D's `ShapeCaster` for ground detection (following the `demo_platformer` example pattern).

### Key Dependencies
- `bevy_ecs_tiled = { version = "0.11", features = ["avian"] }` (includes `avian2d` 0.5)
- Tiled editor installed on the system (user will install)
- TMJ map file created in Tiled

### Reference
- `bevy_ecs_tiled` demo_platformer example: uses Avian2D `ShapeCaster` for ground detection, `RigidBody::Dynamic` for player, auto-generated `RigidBody::Static` colliders on tiles via `TiledPhysicsPlugin`

## Workplan

### Phase 1: Setup & Dependencies
- [ ] Add `bevy_ecs_tiled` with `avian` feature to `Cargo.toml`
- [ ] Remove direct `bevy_ecs_tilemap` dependency (pulled in transitively by `bevy_ecs_tiled`)
- [ ] Verify build compiles with new dependencies
- [ ] Run `cargo vendor` to update vendored dependencies

### Phase 2: Create Tiled Map File
- [ ] Create initial TMJ map file in `assets/` that replicates current 20x15 level layout
  - Orthogonal, 16x16 tile size
  - Use current `tileset.png` as the tileset
  - Tile layer for solid ground and platforms
  - Match current LEVEL_DATA layout
- [ ] Verify TMJ file loads correctly with `bevy_ecs_tiled`

### Phase 3: Replace Level Loading
- [ ] Rewrite `level/mod.rs` to use `bevy_ecs_tiled` `TiledMap` component instead of manual `TilemapBundle` setup
  - Spawn `TiledMap` with asset handle to the TMJ file
  - Use `TilemapAnchor::BottomLeft` (or adjust as needed)
  - Remove `LEVEL_DATA` const array
  - Remove manual tile spawning loop
- [ ] Add `TiledPlugin` and `TiledPhysicsPlugin::<TiledPhysicsAvianBackend>` to app
- [ ] Add `PhysicsPlugins` to app
- [ ] Move camera spawn out of `setup_tilemap` (if needed for separation)
- [ ] Remove or update `LEVEL_WIDTH_IN_TILES` / `LEVEL_HEIGHT_IN_TILES` constants (derive from map)
- [ ] Verify map renders correctly

### Phase 4: Replace Collision System with Avian2D
- [ ] Add `RigidBody::Static` to tile colliders via `TiledEvent<ColliderCreated>` observer
- [ ] Replace player spawn with Avian2D physics body:
  - `RigidBody::Dynamic`
  - `Collider` matching player sprite size
  - `LockedAxes::ROTATION_LOCKED`
  - `ShapeCaster` for ground detection
- [ ] Replace custom `is_grounded()` with Avian2D `ShapeHits`-based ground check
- [ ] Replace custom `ground_snap_y()` — Avian2D handles this via physics
- [ ] Update `player_movement` to use Avian2D `LinearVelocity` instead of custom `Velocity`
- [ ] Update jump logic to set `LinearVelocity.y` directly
- [ ] Configure `Gravity` resource
- [ ] Remove custom collision code:
  - `check_feet_tiles()` function
  - `FeetTiles` struct
  - `ground_snap_y()` function
  - Custom `is_grounded()` function
- [ ] Remove or simplify `level/tile.rs` (no longer needed for collision):
  - `world_to_tile_coords()` — may keep if needed for other purposes
  - `get_tile_type_at()` — no longer needed
  - `TileType` enum — no longer needed
  - `TILEMAP_OFFSET_X/Y` constants — derive from map transform
- [ ] Remove `PlayerCoord` / `coord.rs` if no longer needed
- [ ] Verify player walks, jumps, and collides correctly

### Phase 5: Expand Level for Scrolling
- [ ] Create wider level in Tiled (100+ tiles wide, 15 tiles tall)
  - Starting area with safe platforms
  - Middle section with varied platform arrangements
  - Gaps and obstacles for jumping
- [ ] Verify wider map loads and renders correctly

### Phase 6: Camera Bounds (Spec 011)
- [ ] Add horizontal camera bounds clamping to `camera_follow()`
  - Calculate level bounds from map dimensions (query `TilemapSize` or use map metadata)
  - Clamp camera X after lerp
- [ ] Handle edge case: level smaller than viewport (center the level)
- [ ] Verify camera stops at left/right edges
- [ ] Verify smooth follow in middle of level

### Phase 7: Cleanup & Documentation
- [ ] Remove debug `println!` statements from `movement.rs`
- [ ] Remove the large TODO comment block from `movement.rs`
- [ ] Update spec 011 with actual implementation details
- [ ] Create spec 012 for the Tiled/TMJ adoption
- [ ] Update `docs/bevy-coordinate-system.md` if coordinate system changed
- [ ] Run `cargo test` and fix any broken tests
- [ ] Final manual testing: walk full level, jump on platforms, verify camera bounds

## Notes

### What Gets Removed
- `LEVEL_DATA` const array
- Manual `TilemapBundle` setup in `level/mod.rs`
- Custom collision: `FeetTiles`, `check_feet_tiles()`, `is_grounded()`, `ground_snap_y()`
- Custom `Velocity` component (replaced by Avian2D `LinearVelocity`)
- `PlayerCoord` / `coord.rs` (ground detection handled by physics)
- `level/tile.rs` helpers (`world_to_tile_coords`, `get_tile_type_at`, `TileType`)

### What Gets Added
- `bevy_ecs_tiled` plugin with Avian2D physics backend
- TMJ map file in `assets/`
- Avian2D physics components on player (RigidBody, Collider, ShapeCaster)
- Auto-generated static colliders on solid tiles
- Camera bounds clamping

### Risks
- Avian2D physics feel may differ from current custom movement (acceleration, jump height)
  - Mitigation: Tune `Gravity`, `MovementAcceleration`, `JumpImpulse`, `MovementDampingFactor`
- Player movement code is significantly rewritten — careful testing needed
- `bevy_ecs_tiled` may handle anchoring differently — verify tile alignment
- Build times may increase with Avian2D dependency

### Migration Strategy
- Phase 3 and 4 are the breaking changes — do them together to avoid half-broken state
- Keep old code commented out initially until physics version is verified
- Can fall back to custom collision if Avian2D doesn't work well
