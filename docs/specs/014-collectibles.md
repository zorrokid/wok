# Spec 014 — Collectibles

## Status: Complete

## Overview

Introduces collectible items to the game world. When the player touches a collectible, it is instantly removed from the scene. Collectibles are placed at fixed positions during level initialization. This spec establishes the foundational collectibles system; future specs will add item types, effects, and dynamic spawning.

## Requirements

1. A `Collectible` marker component identifies collectible entities.
2. Collectibles are rendered as yellow squares (16×16 px) using a plain colored sprite — no texture required.
3. Collectibles use an Avian2D `Sensor` collider so they do not physically block the player.
4. When the player's collider overlaps a collectible's sensor, the collectible entity is despawned immediately.
5. An initial set of collectibles is spawned at hardcoded positions during level startup.
6. The collectibles system lives in its own module (`src/collectibles/`) and registers via a `CollectiblesPlugin`.

## Acceptance Criteria

- [x] Yellow squares appear at the defined positions when the game starts.
- [x] Walking or jumping into a yellow square causes it to disappear instantly.
- [x] The player's movement and physics are not affected by touching a collectible (sensor, not solid).
- [x] Collecting all items leaves the level empty of collectibles with no errors or panics.
- [x] The collectibles module compiles cleanly and is wired into `main.rs`.

## Implementation Plan

### Module Structure

Create `src/collectibles/` with:

- `mod.rs` — `CollectiblesPlugin`, `Collectible` marker component, spawn positions constant, and the `spawn_collectibles` startup system.
- `collection.rs` — `collect_items` update system that reads Avian2D `CollisionStarted` events and despawns any collectible that the player touches.

### Visual Representation

Spawn each collectible as a `Sprite` with:
- `color: Color::srgb(1.0, 1.0, 0.0)` (yellow)
- `custom_size: Some(Vec2::splat(16.0))` (16×16 px square)
- `Transform` at the desired world position

No texture asset is needed.

### Collision Detection

Avian2D `Sensor` colliders generate `CollisionStarted` events without exerting forces. The flow is:

1. Each collectible entity is spawned with `Collider::rectangle(16.0, 16.0)` and `Sensor`.
2. Each frame, the `collect_items` system reads `EventReader<CollisionStarted>` events from Avian2D.
3. For each event `CollisionStarted(e1, e2)`, check if one entity has `Player` and the other has `Collectible`.
4. Despawn the `Collectible` entity via `commands.entity(collectible_entity).despawn()`.

`CollisionStarted` is an Avian2D event fired when two colliders first make contact in a physics step. It fires for sensor contacts as well as solid contacts.

### Spawn Positions

Define a constant array of world positions in `src/collectibles/mod.rs`. Use tile-relative coordinates (same origin as the player spawn) so positions remain correct regardless of `TILEMAP_OFFSET_X`/`TILEMAP_OFFSET_Y`. Example set (5 collectibles, adjust in playtesting):

```rust
const COLLECTIBLE_POSITIONS: &[(f32, f32)] = &[
    (5.0, 4.0),   // tile column 5, row 4
    (10.0, 4.0),
    (15.0, 6.0),
    (20.0, 4.0),
    (25.0, 8.0),
];
```

Convert to world coordinates in `spawn_collectibles` using the same `TILEMAP_OFFSET_X`, `TILEMAP_OFFSET_Y`, and `TILE_SIZE` constants already used by the player spawn.

### Plugin Registration

`CollectiblesPlugin::build` registers:
- `Startup`: `spawn_collectibles`
- `Update`: `collect_items`

`main.rs` adds `CollectiblesPlugin` alongside the existing plugins.

## Notes

- **No `RigidBody` on collectibles**: Sensor colliders do not require a `RigidBody`. Omitting it avoids unnecessary physics simulation for these static items.
- **Despawn safety**: Avian2D may emit multiple `CollisionStarted` events for the same pair across frames if the contact persists. Use `commands.entity(...).despawn()` defensively — Bevy silently ignores despawn calls on already-despawned entities as of Bevy 0.18.
- **Future extensibility**: The `Collectible` component can grow fields (item type, value) when future specs introduce multiple collectible types. For now it is a pure marker.
- **Dynamic spawning**: This spec covers only static startup placement. Dynamic mid-gameplay spawning is deferred to a future spec.

## Task Checklist

- [x] Create `src/collectibles/mod.rs` with `Collectible` component, `CollectiblesPlugin`, spawn positions, and `spawn_collectibles` system
- [x] Create `src/collectibles/collection.rs` with `collect_items` system
- [x] Wire `CollectiblesPlugin` into `main.rs`
- [x] Declare `mod collectibles` in `main.rs`
- [x] Verify yellow squares appear at the correct positions in-game
- [x] Verify collecting items removes them without affecting player physics
- [x] Mark spec complete after user verification

## Related Specs

- **013 — Architecture Refactor**: establishes the module/plugin patterns this spec follows
- **012 — Level Bounds**: example of a level-startup spawn system

## Related Files

- `src/main.rs` — add `mod collectibles` and `CollectiblesPlugin`
- `src/collectibles/mod.rs` — new file
- `src/collectibles/collection.rs` — new file
- `src/level/tile.rs` — constants used for world coordinate conversion
- `src/player/mod.rs` — `Player` marker component used in collision query
