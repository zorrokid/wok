# Spec 016 — Map Switching

## Status: Complete

## Overview

Establishes the core map switching infrastructure: a second map file, a `CurrentMap` resource tracking the active map entity, a `LevelTransitionEvent` that carries destination data, and an `apply_transition` system that despawns the old map, loads the new one, and repositions the player. A temporary keyboard shortcut (T / R keys) triggers test transitions so the plumbing can be verified independently of Tiled object zones, which are added in spec 017.

## Requirements

1. `assets/map.tmx` is renamed to `assets/map1.tmx`; a second map `assets/map2.tmx` is created using the same tileset and dimensions.
2. A `CurrentMap(Entity)` resource tracks the active `TiledMap` entity; it is inserted at the end of `setup_tilemap` and updated after every transition.
3. The existing inline `ColliderCreated` observer closure in `setup_tilemap` is extracted to a named function `on_collider_created` so it can be reattached when a new map entity is spawned post-transition.
4. A `LevelTransitionEvent { target_map: String, spawn_tile_x: f32, spawn_tile_y: f32 }` Bevy event carries transition destination data.
5. An `apply_transition` system reads `LevelTransitionEvent`, despawns the old map hierarchy, spawns the new `TiledMap` entity with `on_collider_created` attached, updates `CurrentMap`, and repositions the player using `Position` (not `Transform`).
6. A temporary `debug_trigger_transition` system fires a `LevelTransitionEvent` when the player presses T (→ map2) or R (→ map1), enabling in-game verification without transition zones.
7. All new logic lives in `src/level/transition.rs` and is wired through `LevelPlugin`.

## Acceptance Criteria

- [x] `map1.tmx` loads on startup as before; the game is otherwise unchanged.
- [x] `map2.tmx` loads correctly when triggered (tiles render, physics colliders present).
- [x] Pressing T switches from map1 to map2 and places the player at the correct position.
- [x] Pressing R switches from map2 to map1 and places the player at the correct position.
- [x] The previous map's tiles and colliders are fully removed after each switch.
- [x] Player health and physics state are preserved; only position and velocity are overwritten.
- [x] No panics or errors on repeated back-and-forth switches.

## Implementation Plan

### Map Files

Rename `assets/map.tmx` → `assets/map1.tmx`. Create `assets/map2.tmx` using the same tileset (`tileset.tsx`) and dimensions (`LEVEL_WIDTH_IN_TILES` × `LEVEL_HEIGHT_IN_TILES`). Map2 can have a different tile layout to make the switch visually obvious. No object layer is needed yet — that is added in spec 017.

### New Types

**`CurrentMap`** — resource tracking the active map entity:

```rust
#[derive(Resource)]
pub struct CurrentMap(pub Entity);
```

**`LevelTransitionEvent`** — carries destination data for `apply_transition`:

```rust
#[derive(Event)]
pub struct LevelTransitionEvent {
    pub target_map: String,
    pub spawn_tile_x: f32,
    pub spawn_tile_y: f32,
}
```

### `on_collider_created` (extracted from `setup_tilemap`)

The existing inline observer closure that marks tile colliders as `RigidBody::Static` is extracted to a named function:

```rust
fn on_collider_created(ev: On<TiledEvent<ColliderCreated>>, mut commands: Commands) {
    commands.entity(ev.event().origin).insert(RigidBody::Static);
}
```

This allows it to be referenced by name when attaching to new map entities spawned during transitions.

### Updated `setup_tilemap`

```rust
pub fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map1.tmx");
    let entity = commands
        .spawn((TiledMap(map_handle), TilemapAnchor::Center))
        .observe(on_collider_created)
        .id();
    commands.insert_resource(CurrentMap(entity));
}
```

### `apply_transition`

Despawns the old map hierarchy and spawns the new one. Player is repositioned using `Position` (Avian2D's source of truth for dynamic bodies — writing `Transform` directly would be overwritten by the next physics tick):

```rust
pub fn apply_transition(
    mut events: EventReader<LevelTransitionEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current_map: Res<CurrentMap>,
    mut player_q: Query<(&mut Position, &mut LinearVelocity), With<Player>>,
) {
    let Some(ev) = events.read().next() else { return };

    commands.entity(current_map.0).despawn_related::<Children>();
    commands.entity(current_map.0).despawn();

    let map_handle: Handle<TiledMapAsset> = asset_server.load(ev.target_map.clone());
    let new_entity = commands
        .spawn((TiledMap(map_handle), TilemapAnchor::Center))
        .observe(on_collider_created)
        .id();

    commands.insert_resource(CurrentMap(new_entity));

    if let Ok((mut pos, mut vel)) = player_q.single_mut() {
        let world_x = TILEMAP_OFFSET_X + ev.spawn_tile_x * TILE_SIZE + TILE_SIZE / 2.0;
        let world_y = TILEMAP_OFFSET_Y + ev.spawn_tile_y * TILE_SIZE + TILE_SIZE / 2.0;
        pos.0 = Vec2::new(world_x, world_y);
        // Clear momentum so the player does not arrive at speed.
        vel.0 = Vec2::ZERO;
    }
}
```

### `debug_trigger_transition` (temporary)

Allows testing the map switch without transition zones. Removed in spec 017 once zones replace it:

```rust
pub fn debug_trigger_transition(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut writer: EventWriter<LevelTransitionEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        writer.write(LevelTransitionEvent {
            target_map: "map2.tmx".to_string(),
            spawn_tile_x: 2.0,
            spawn_tile_y: 4.0,
        });
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        writer.write(LevelTransitionEvent {
            target_map: "map1.tmx".to_string(),
            spawn_tile_x: 28.0,
            spawn_tile_y: 4.0,
        });
    }
}
```

### Plugin Registration

```rust
app
    .add_event::<LevelTransitionEvent>()
    .add_systems(Startup, (setup_tilemap, spawn_level_bounds))
    .add_systems(Update, (
        debug_trigger_transition,
        apply_transition.after(debug_trigger_transition),
    ));
```

### Module Structure

```
src/level/
    mod.rs          Updated: on_collider_created extracted, CurrentMap inserted,
                    LevelTransitionEvent and transition systems registered
    transition.rs   New: CurrentMap, LevelTransitionEvent, apply_transition,
                    debug_trigger_transition

assets/
    map1.tmx        Renamed from map.tmx
    map2.tmx        New second map
```

## Notes

- **`Position` not `Transform`**: Avian2D owns the `Transform` of dynamic rigid bodies. Writing `Position` is the correct way to teleport a physics body; the engine propagates it to `Transform` each frame.
- **`despawn_related::<Children>` + `despawn()`**: Despawns the map root entity and all its descendants (tiles, colliders, child entities). This is the Bevy 0.18 pattern for hierarchy teardown.
- **Level bounds persist**: `spawn_level_bounds` runs only at `Startup` and spawns wall colliders as independent entities (not children of the map). They remain valid across transitions as long as both maps share the same dimensions.
- **Enemies and collectibles are independent entities**: They are not children of `TiledMap` and will persist after a transition. For now, keep enemies and collectibles only in map1. Scoping gameplay entities to the active map is deferred to a future spec.
- **T and R keys are temporary**: `debug_trigger_transition` exists only to test the plumbing. It is removed in spec 017 when physics-based transition zones take over.
- **Message not Event**: Bevy 0.18 replaced the old `Event`/`EventWriter`/`EventReader`/`add_event` API with `Message`/`MessageWriter`/`MessageReader`/`add_message`. The spec's code samples used the old names; the implementation uses the correct Bevy 0.18 API. The derive macro is `#[derive(Message)]` and registration is `app.add_message::<T>()`.

## Task Checklist

- [x] Rename `assets/map.tmx` to `assets/map1.tmx`
- [x] Create `assets/map2.tmx` with same tileset and dimensions
- [x] Create `src/level/transition.rs` with `CurrentMap`, `LevelTransitionEvent`, `apply_transition`, `debug_trigger_transition`
- [x] Update `src/level/mod.rs`: extract `on_collider_created`, update `setup_tilemap` to use `map1.tmx` and insert `CurrentMap`, register event and systems in `LevelPlugin`
- [x] Verify map1 loads on startup as before
- [x] Verify T key switches to map2 with correct player position
- [x] Verify R key switches back to map1 with correct player position
- [x] Verify old map is fully removed after each switch
- [x] Verify no panics on repeated switching
- [x] Mark spec complete after user verification

## Related Specs

- **017 — Transition Zones**: replaces `debug_trigger_transition` with physics-based Tiled object zones
- **015 — Enemy Hazards**: documents `Position` vs `Transform` for dynamic body repositioning

## Related Files

- `assets/map.tmx` → `assets/map1.tmx`
- `assets/map2.tmx` — new file
- `src/level/mod.rs` — extract observer, update map filename, insert `CurrentMap`, register event and systems
- `src/level/transition.rs` — new file
- `src/level/tile.rs` — coordinate constants used in `apply_transition`
- `src/player/mod.rs` — `Player` marker used in player query
