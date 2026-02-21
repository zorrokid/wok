# Spec 019 — Dynamic Map Dimensions

## Status: Complete

## Overview

Map dimensions (width, height, tile size, and world-space origin offset) are currently hardcoded as compile-time constants in `src/level/tile.rs`. If a designer changes a map's dimensions in Tiled, three separate systems produce wrong output: the level-bounds walls are misplaced, the camera clamping range is wrong, and the player spawns at an incorrect world position after a transition. This spec replaces the constants with a `MapDimensions` resource that is populated from `TiledMapAsset` data whenever a map finishes loading. All systems that previously read the constants read the resource instead. No Rust code changes are required when a designer resizes a map.

## Requirements

1. A `MapDimensions` resource exists that stores tile size, map dimensions in tiles, and the world-space bottom-left offset derived from `TilemapAnchor::Center`.
2. `MapDimensions` is updated each time a new map finishes loading, before any system that depends on it reads it within the same frame.
3. Level-bounds wall entities are despawned when the map changes and re-spawned with dimensions matching the newly-loaded map.
4. `apply_transition` computes the player's world-space spawn position using the current `MapDimensions` (which reflects the previous map's dimensions at the moment of transition), not compile-time constants.
5. The camera clamping in `camera_follow` remains correct for maps of any width without referencing compile-time constants.
6. The five constants `TILE_SIZE`, `LEVEL_WIDTH_IN_TILES`, `LEVEL_HEIGHT_IN_TILES`, `TILEMAP_OFFSET_X`, and `TILEMAP_OFFSET_Y` are removed from `src/level/tile.rs` and all their import sites.

## Acceptance Criteria

- [x] Changing `map1.tmx` to 50×20 tiles in the Tiled editor causes the level-bounds walls and camera clamp to reflect the new dimensions without editing any Rust source file.
- [x] The player spawns at the correct tile position when transitioning between maps of different dimensions.
- [x] Level-bounds walls are despawned on map change and re-spawned at the correct edges of the new map.
- [x] `cargo check` passes with zero references to the removed constants.
- [x] No panics or visual glitches during map transitions.

---

## Implementation Plan

### Approach

`TiledMapAsset` exposes the fields needed to derive all map dimension information at load time:

```rust
pub struct TiledMapAsset {
    pub tilemap_size: TilemapSize,         // width/height in tiles (u32)
    pub largest_tile_size: TilemapTileSize, // tile pixel size (f32)
    pub rect: Rect,                        // pixel bounding box, origin bottom-left
    // ...
}
```

`bevy_ecs_tiled` fires `TiledEvent<MapCreated>` (as both a trigger and a message) once per map load, after all layer and object entities are spawned. The event carries the `AssetId<TiledMapAsset>`, accessible via `ev.get_map_asset(&assets)`. This is the correct hook for reading dimensions.

The resource `MapDimensions` is introduced in `src/level/tile.rs` and updated by a `on_map_created` observer (global, registered with `app.add_observer`) that fires on each `TiledEvent<MapCreated>`. Because global observers run immediately when the trigger is sent (not deferred to the next frame), `MapDimensions` is populated in the same frame that the map finishes loading, before any `Update` system reads it.

**Level bounds lifecycle**: A `LevelBoundsWall` marker component tags the two wall entities. `on_map_created` despawns all existing `LevelBoundsWall` entities via `Commands` before spawning new ones with the new map's dimensions. This replaces the current `spawn_level_bounds` `Startup` system.

**Transition spawn position**: `apply_transition` runs in `Update` and reads `MapDimensions` at the moment the transition event is consumed. At that point, the old map is being despawned and the new map has not yet loaded, so `MapDimensions` still holds the old map's dimensions. The current maps happen to share the same dimensions, so this works correctly today. However, for correctness across maps of different sizes, `LevelTransitionEvent` should carry the pre-computed world-space spawn position rather than raw tile coordinates. `trigger_transition` computes `world_x/world_y` from `MapDimensions` when it writes the event; `apply_transition` simply applies the pre-computed position.

**Camera**: `camera_follow` already queries `TilemapSize` from the tile layer children (`Query<&TilemapSize, With<TileStorage>>`) and is already correct for maps of any width under `TilemapAnchor::Center`. It does not use the compile-time constants. No change is needed to the camera clamping logic — only the removal of the `TILE_SIZE` import that is no longer needed.

**`tile.rs`**: After the migration, `src/level/tile.rs` contains only `MapDimensions` (the resource struct and its implementation). All five constants are removed.

### New Types

**`MapDimensions`** (`src/level/tile.rs`, `#[derive(Resource)]`): Runtime map metadata derived from `TiledMapAsset`. Replaces the five compile-time constants. Fields:
- `tile_size: f32` — pixel size of one tile square (from `largest_tile_size`)
- `width_tiles: u32` — map width in tiles
- `height_tiles: u32` — map height in tiles
- `offset_x: f32` — world X of the bottom-left corner (negative half-width under `TilemapAnchor::Center`)
- `offset_y: f32` — world Y of the bottom-left corner (negative half-height under `TilemapAnchor::Center`)

Convenience methods on `MapDimensions`:
- `width_px(&self) -> f32` — `width_tiles as f32 * tile_size`
- `height_px(&self) -> f32` — `height_tiles as f32 * tile_size`
- `tile_to_world(&self, tile_x: f32, tile_y: f32) -> Vec2` — converts tile column/row to the tile's center in world space: `Vec2::new(self.offset_x + tile_x * self.tile_size + self.tile_size / 2.0, self.offset_y + tile_y * self.tile_size + self.tile_size / 2.0)`

Pure function `map_dimensions_from_asset(asset: &TiledMapAsset) -> MapDimensions`: extracts fields from the asset and computes offsets. Extracted for unit-testability.

**`LevelBoundsWall`** (`src/level/mod.rs`, `#[derive(Component)]`): Marker component on the two invisible static wall collider entities. Used to query and despawn old walls before spawning new ones.

**`LevelTransitionEvent`** (modified): Replace `spawn_tile_x: f32` / `spawn_tile_y: f32` with `spawn_pos: Vec2`. The world position is computed by `trigger_transition` when writing the event, using the current `MapDimensions`. `apply_transition` no longer does coordinate conversion.

### Systems

- `on_map_created(ev: On<TiledEvent<MapCreated>>, ...)` — Global observer in `src/level/mod.rs`. Fires each time a map finishes loading. Reads the `TiledMapAsset` via the event's asset ID, calls `map_dimensions_from_asset`, writes `MapDimensions` resource, despawns all entities with `LevelBoundsWall`, spawns two new wall colliders with the updated dimensions. Registered with `app.add_observer`. Replaces `spawn_level_bounds`.

- `trigger_transition` (modified) — Now reads `Res<MapDimensions>` and calls `map_dimensions.tile_to_world(t.spawn_tile_x, t.spawn_tile_y)` before writing `LevelTransitionEvent { spawn_pos, .. }`.

- `apply_transition` (modified) — Reads `spawn_pos: Vec2` directly from the event instead of computing from `TILEMAP_OFFSET_X/Y`. No longer imports anything from `tile.rs`.

- `spawn_level_bounds` — **Removed**. Its responsibility is absorbed into `on_map_created`.

### Phases

- **Phase 1**: Introduce `MapDimensions` resource and `on_map_created` observer; keep constants but do not use them for bounds spawning. Gate: compiles; walls spawn correctly on initial map load using asset data.

- **Phase 2**: Update `trigger_transition` to compute world-space spawn position from `MapDimensions` and embed it in `LevelTransitionEvent`. Update `apply_transition` to consume `spawn_pos` directly. Gate: map transitions place the player correctly; `TILEMAP_OFFSET_X/Y` no longer used in `transition.rs`.

- **Phase 3**: Remove the five constants from `tile.rs` and all import sites. Remove `TILE_SIZE` import from `camera.rs`. Verify `cargo check` passes clean. Gate: zero compile errors, zero references to removed constants.

### Tasks

- [x] Add `MapDimensions` resource struct to `src/level/tile.rs` with `tile_size`, `width_tiles`, `height_tiles`, `offset_x`, `offset_y` fields
- [x] Add `tile_to_world`, `width_px`, `height_px` methods to `MapDimensions`
- [x] Add pure function `map_dimensions_from_asset(asset: &TiledMapAsset) -> MapDimensions` to `src/level/tile.rs`
- [x] Write unit tests for `map_dimensions_from_asset` and `tile_to_world` covering a known tile size and map size
- [x] Add `LevelBoundsWall` marker component to `src/level/mod.rs`
- [x] Add `on_map_created` global observer to `src/level/mod.rs` that: reads asset, calls `map_dimensions_from_asset`, inserts `MapDimensions` resource, queries + despawns `LevelBoundsWall` entities, spawns two new walls tagged `LevelBoundsWall`
- [x] Register `MapDimensions` with `app.init_resource::<MapDimensions>()` and `on_map_created` with `app.add_observer(on_map_created)` in `LevelPlugin`
- [x] Remove `spawn_level_bounds` function and its `Startup` registration from `LevelPlugin`
- [x] Add `spawn_pos: Vec2` field to `LevelTransitionEvent`; remove `spawn_tile_x` and `spawn_tile_y` fields
- [x] Update `trigger_transition` to read `Res<MapDimensions>` and call `map_dimensions.tile_to_world(t.spawn_tile_x, t.spawn_tile_y)` when writing the event
- [x] Update `apply_transition` to read `ev.spawn_pos` directly and write it to `pos.0`; remove all imports from `tile.rs`
- [x] Remove `TILE_SIZE`, `LEVEL_WIDTH_IN_TILES`, `LEVEL_HEIGHT_IN_TILES`, `TILEMAP_OFFSET_X`, `TILEMAP_OFFSET_Y` from `src/level/tile.rs`
- [x] Remove all import sites of these constants (`src/level/mod.rs`, `src/level/transition.rs`, `src/camera.rs`)
- [x] Run `cargo check` and confirm zero errors and zero references to removed constants
- [x] Test: initial map load shows walls at correct left and right edges of map1
- [x] Test: transitioning to map2 despawns old walls and spawns new walls at map2's edges
- [x] Test: player spawns at the correct tile position in map2 after transitioning from map1
- [x] Test: transitioning back to map1 restores correct wall positions
- [x] Mark spec complete after user verification

### Notes

**Observer vs. MessageReader for `MapCreated`**: `on_map_created` is registered as a global observer (`app.add_observer`), not as a `MessageReader`-based system. Observers run synchronously when the trigger fires (inside `spawn_map`), before `Update` systems. A `MessageReader`-based system in `Update` would work but runs one frame later, creating a one-frame window where `MapDimensions` is stale and `LevelBoundsWall` entities from the old map still exist. The observer approach avoids this gap.

**`MapDimensions` default before first map load**: `app.init_resource::<MapDimensions>()` requires `Default`. The default should use sensible zero or unit values — systems that would read it before any map loads (none currently) should guard against it. Alternatively, initialize it in the `Startup` system after confirming the asset is not yet available (see the observer).  The simplest approach: derive `Default` with all fields zero, and note that `on_map_created` will populate it before any game logic runs, since map loading completes in `PreUpdate`.

**`LevelTransitionEvent` change is a breaking API change within the crate**: `spawn_tile_x` and `spawn_tile_y` are replaced by `spawn_pos: Vec2`. The only write site is `trigger_transition` and the only read site is `apply_transition`, both in `src/level/transition.rs`. No external callers exist.

**Timing of `on_map_created` vs. bounds despawn**: `Commands::despawn` is deferred; the `LevelBoundsWall` entities are not actually removed until the end of the current command queue flush. The new wall entities are spawned in the same `Commands` queue. Bevy processes `Commands` in order, so the despawn happens before the spawn within the same observer call. This is safe.

**`camera_follow` and `TILE_SIZE`**: `camera_follow` currently imports `TILE_SIZE` from `src/level/tile`. After removing the constant, the camera system should compute `half_width = tilemap_size.x as f32 * tile_size / 2.0` using the `TilemapTileSize` from a tilemap query, or use `MapDimensions`. The simplest fix: add a `Res<MapDimensions>` parameter to `camera_follow` and use `map_dimensions.tile_size`. Alternatively query `TilemapTileSize` from the tilemap child entities (already available as `TilemapBundle` sets it). Using `MapDimensions` is less coupling to bevy_ecs_tiled internals and is preferred.

**Test with maps of different dimensions**: to fully verify this feature, temporarily change `map2.tmx` to a different size (e.g., 60×25 tiles) and confirm the walls and camera update correctly after transition.

**`world_space_from_tiled_position` as alternative**: `TiledMapAsset` exposes `world_space_from_tiled_position(&self, anchor, tiled_position)` which converts a Tiled pixel position to Bevy world space. The `tile_to_world` method on `MapDimensions` reimplements this for the specific case of tile-center positions under `TilemapAnchor::Center` with an orthogonal square map. This is intentional — `MapDimensions` must be readable without `Assets<TiledMapAsset>` access (e.g., in `trigger_transition` which has no asset access).

## Related Specs

- **016 — Map Switching**: introduces `apply_transition` and `CurrentMap`; defines the comment that explicitly flags this future work
- **017 — Transition Zones**: introduces `LevelTransition`, `spawn_tile_x/y` fields, and the coordinate formula being replaced
- **012 — Wall Tile Types**: original introduction of `spawn_level_bounds`

## Related Files

- `src/level/tile.rs` — `MapDimensions` resource replaces constants
- `src/level/mod.rs` — `on_map_created` observer, `LevelBoundsWall` marker, remove `spawn_level_bounds`
- `src/level/transition.rs` — `LevelTransitionEvent` gains `spawn_pos`, loses tile fields; `trigger_transition` and `apply_transition` updated
- `src/camera.rs` — remove `TILE_SIZE` import; use `MapDimensions` for tile size
