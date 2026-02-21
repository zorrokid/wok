# Spec 017 — Transition Zones

## Status: Complete

## Overview

Replaces the temporary keyboard trigger from spec 016 with physics-based transition zones defined in the Tiled map. Each map has an object layer containing a rectangle whose transition destination is stored as a Tiled Custom Class property. bevy_ecs_tiled automatically deserializes the property into a `LevelTransition` component on the spawned object entity. A global observer adds `Sensor`, `Collider`, and `CollisionEventsEnabled` to complete the trigger zone. A `trigger_transition` system detects when the player enters the zone and sends a `LevelTransitionEvent`, which `apply_transition` (from spec 016) handles.

## Requirements

1. `LevelTransition { target_map: String, spawn_tile_x: f32, spawn_tile_y: f32 }` is a Bevy component that derives `Reflect` and is registered in the type registry.
2. Each map has an object layer with at least one rectangle object carrying a `LevelTransition` custom class property.
3. A global observer on `TiledEvent<ObjectCreated>` detects entities that have a `LevelTransition` component (added automatically by bevy_ecs_tiled) and inserts `Sensor`, `Collider::rectangle`, and `CollisionEventsEnabled` to make the zone physically detectable.
4. A `trigger_transition` system reads Avian2D `CollisionStart` messages and sends `LevelTransitionEvent` when the player contacts a `LevelTransition` entity.
5. The temporary `debug_trigger_transition` system (T / R keys) from spec 016 is removed.
6. `apply_transition` and the rest of the map switching infrastructure from spec 016 are unchanged except for `TiledPhysicsSettings` addition (see Implementation Notes).

## Acceptance Criteria

- [x] `LevelTransition` component is present on transition zone entities after map load (confirming bevy_ecs_tiled deserialization works).
- [x] Transition zone entities have `Sensor` and `Collider` (confirming the observer fires).
- [x] Walking into the right-edge zone on map1 transitions to map2 and places the player at the correct position.
- [x] Walking into the left-edge zone on map2 transitions back to map1 and places the player at the correct position.
- [x] The T and R debug keys no longer trigger transitions.
- [x] No panics or errors on repeated back-and-forth transitions.

## Implementation Plan

### `LevelTransition` Component

Derives `Reflect` so bevy_ecs_tiled can deserialize it from Tiled class properties and insert it automatically onto spawned object entities:

```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct LevelTransition {
    pub target_map: String,
    pub spawn_tile_x: f32,
    pub spawn_tile_y: f32,
}
```

`target_map` is `String` (not `&'static str`) because it is deserialized from Tiled at runtime. `spawn_tile_x/y` use `f32` because bevy_ecs_tiled's property system deserializes Tiled float properties as `f32`.

### Tiled Setup: Custom Class Property

bevy_ecs_tiled's property system supports **Class** type properties backed by registered Rust types. The `user_properties` feature **must** be enabled in `Cargo.toml` — without it, the entire deserialization block is compiled out and properties are silently ignored:

```toml
bevy_ecs_tiled = { version = "0.11.2", features = ["avian", "user_properties"] }
```

Custom types are not configurable via the Tiled 1.11 GUI. Instead, add the object XML directly to the `.tmx` file. The property name and `propertytype` attribute must match the **full Rust type path** exactly:

```xml
<objectgroup id="3" name="Entities">
  <object id="1" x="1520" y="0" width="80" height="480">
    <properties>
      <property name="wok::level::transition::LevelTransition"
                type="class"
                propertytype="wok::level::transition::LevelTransition">
        <properties>
          <property name="target_map" value="map2.tmx"/>
          <property name="spawn_tile_x" type="float" value="7"/>
          <property name="spawn_tile_y" type="float" value="4"/>
        </properties>
      </property>
    </properties>
  </object>
</objectgroup>
```

Object placements (both maps are 100×30 tiles, 16px per tile):

| Map   | Object position (Tiled) | `target_map`  | `spawn_tile_x` | `spawn_tile_y` |
|-------|-------------------------|---------------|----------------|----------------|
| map1  | x=1520, w=80 (right edge, 5 tiles) | `"map2.tmx"` | `7.0` | `4.0` |
| map2  | x=0, w=80 (left edge, 5 tiles)     | `"map1.tmx"` | `88.0` | `4.0` |

Spawn positions are placed well inside the destination map (7 tiles from left, 12 tiles from right edge) so the player never spawns inside a transition zone.

### Preventing Automatic Solid Object Colliders

`bevy_ecs_tiled`'s physics backend runs `collider_from_object` in `PreUpdate` via `MessageReader<TiledEvent<ObjectCreated>>`. By default (`TiledFilter::All`), it creates a **solid** `RigidBody::Static` + `Collider` for every object in every object layer — including our transition zones. This would block the player rather than letting them through.

Fix: attach `TiledPhysicsSettings` with `objects_layer_filter: TiledFilter::None` when spawning every `TiledMap` entity. This disables automatic object-layer collider generation while leaving tile-layer collider generation intact:

```rust
commands.spawn((
    TiledMap(map_handle),
    TilemapAnchor::Center,
    TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
        objects_layer_filter: TiledFilter::None,
        ..Default::default()
    },
))
```

This must be applied in both `setup_tilemap` (initial load) and `apply_transition` (subsequent loads).

### Observer: `setup_transition_colliders`

Registered as a **global observer** via `app.add_observer()`, not per-entity `.observe()`. Global registration means it automatically covers both the initial map load and every subsequent map loaded via `apply_transition`, with no need to re-attach per entity.

When a `TiledEvent<ObjectCreated>` fires:
- If the entity does not have `LevelTransition`, returns immediately (handles non-transition objects).
- If it has `LevelTransition`, adds Avian2D sensor components.

**Transform centering**: bevy_ecs_tiled places the entity `Transform` at the **top-left corner** of the Tiled rectangle. Avian2D centers `Collider` on the entity's `Transform`. The fix shifts the Transform to the rectangle's center: `+width/2` in X, `-height/2` in Y (world Y is up, Tiled Y is down).

**`CollisionEventsEnabled` is required**: Avian2D only writes `CollisionStart` messages for a contact pair if at least one entity has `CollisionEventsEnabled`. The player does not have it, so it must be on the zone entity.

```rust
pub fn setup_transition_colliders(
    ev: On<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    query: Query<(&TiledObject, Has<LevelTransition>, &Transform)>,
) {
    let entity = ev.event().origin;
    let Ok((obj, has_transition, transform)) = query.get(entity) else { return };
    if !has_transition { return; }

    let TiledObject::Rectangle { width, height } = obj else { return };
    let (width, height) = (*width, *height);

    // Transform is at top-left; Collider is centered — shift to actual center.
    let center = Transform::from_xyz(
        transform.translation.x + width / 2.0,
        transform.translation.y - height / 2.0,
        transform.translation.z,
    );

    commands.entity(entity).insert((
        center,
        RigidBody::Static,
        Sensor,
        Collider::rectangle(width, height),
        CollisionEventsEnabled,
    ));
}
```

### System: `trigger_transition`

Reads `CollisionStart` messages (one-shot per contact begin) each frame. Uses `MessageWriter<LevelTransitionEvent>`, not the legacy `EventWriter`.

A `TransitionCooldown` timer prevents duplicate events. The root cause of duplicates: `Position` writes take effect one physics tick after `apply_transition` runs, so the player may still geometrically overlap a zone on the new map for one frame and fire another `CollisionStart`. The cooldown (0.25s, named `TRANSITION_COOLDOWN_SECS`) masks this overlap.

```rust
pub fn trigger_transition(
    mut collision_reader: MessageReader<CollisionStart>,
    player_q: Query<Entity, With<Player>>,
    transition_q: Query<&LevelTransition>,
    mut writer: MessageWriter<LevelTransitionEvent>,
    mut cooldown: ResMut<TransitionCooldown>,
    time: Res<Time>,
) {
    cooldown.0.tick(time.delta());
    if !cooldown.0.is_finished() { return; }

    let Ok(player) = player_q.single() else { return };
    for event in collision_reader.read() {
        let other = if event.collider1 == player { event.collider2 }
                    else if event.collider2 == player { event.collider1 }
                    else { continue };
        if let Ok(t) = transition_q.get(other) {
            writer.write(LevelTransitionEvent { ... });
            cooldown.0.reset();
            break; // only one transition per frame
        }
    }
}
```

### Camera Snap on Transition

When the player teleports via `apply_transition`, the camera's lerp-based follow produces a slow scroll across the full map width. Fix: in `camera_follow`, use `lerp_factor = 1.0` (instant snap) when the camera is more than `CAMERA_SNAP_THRESHOLD = 300.0` pixels from the player. This threshold is never reached during normal gameplay and only triggers after a teleport.

### Plugin Registration Changes

```rust
app.register_type::<LevelTransition>()
    .init_resource::<TransitionCooldown>()
    .add_message::<LevelTransitionEvent>()
    .add_observer(setup_transition_colliders)  // global observer
    .add_systems(Startup, (setup_tilemap, spawn_level_bounds))
    .add_systems(Update, (
        trigger_transition,
        apply_transition.after(trigger_transition),
    ));
```

### Module Structure Changes

```
src/level/
    transition.rs   Add: LevelTransition, TransitionCooldown, setup_transition_colliders,
                         trigger_transition
                    Remove: debug_trigger_transition

src/camera.rs       Add: CAMERA_SNAP_THRESHOLD, snap logic in camera_follow

assets/
    map1.tmx        Add objectgroup with LevelTransition class property rectangle
    map2.tmx        Add objectgroup with LevelTransition class property rectangle

Cargo.toml          Add "user_properties" feature to bevy_ecs_tiled
```

## Implementation Notes

### bevy_ecs_tiled `user_properties` Feature Is Required

Without `features = ["user_properties"]` in `Cargo.toml`, the entire property deserialization block inside bevy_ecs_tiled is compiled out (`#[cfg(feature = "user_properties")]`). Properties are silently ignored — no error, no log, no crash — and `LevelTransition` will never be present on object entities. Always enable this feature when using Custom Class properties.

### `collider_from_object` Creates Solid Colliders for All Object-Layer Objects

The `TiledPhysicsPlugin` registers `collider_from_object` in `PreUpdate` which by default creates a solid `RigidBody::Static` + `Collider` for every object in every object layer. Adding `TiledPhysicsSettings { objects_layer_filter: TiledFilter::None }` to the `TiledMap` entity at spawn time is the correct way to opt out. This must be done on every `TiledMap` spawn (both initial and post-transition).

### Global Observer Required for Multi-Spawn Coverage

Per-entity `.observe(setup_transition_colliders)` on the `TiledMap` entity does **not** receive `TiledEvent<ObjectCreated>` triggers for child object entities — the observer must be on the object entity itself, not its map ancestor. `app.add_observer()` (global observer) fires for every trigger in the app regardless of which entity it targets, solving this cleanly.

### `CollisionEventsEnabled` Is Required on Sensor Zone Entities

Avian2D only writes `CollisionStart` messages when at least one entity in the pair has `CollisionEventsEnabled`. The sensor flag alone is not sufficient. The player entity does not have `CollisionEventsEnabled`, so the transition zone entity must have it.

### `Position` Not `Transform` for Player Repositioning

Avian2D owns the `Transform` of dynamic rigid bodies and overwrites it from `Position` each physics tick. Writing `Transform` directly in `apply_transition` would be overwritten by physics within one frame. Always write `Position` (Avian's component) to reposition physics-driven entities.

### Spawn Positions Must Clear All Transition Zones

If a player spawns inside a destination zone, `CollisionStart` fires immediately on the next physics frame and a second transition triggers. Place spawn positions far enough from zone edges that the player cannot reach the zone within `TRANSITION_COOLDOWN_SECS` seconds of the initial transition.

### bevy_ecs_tiled Transform Is at Top-Left, Avian Collider Is Centered

When bevy_ecs_tiled spawns a rectangle object entity, it places the `Transform` at the **top-left corner** of the Tiled rectangle. Avian2D's `Collider::rectangle(w, h)` is centered on the entity's `Transform`. To align the collider with the drawn rectangle, shift the Transform by `+width/2` in X and `-height/2` in Y (world Y is up, Tiled Y is down).

### Coordinate Formula Assumes Uniform Map Size

The player spawn formula (`TILEMAP_OFFSET_X + tile_x * TILE_SIZE + TILE_SIZE/2`) relies on compile-time constants derived from `LEVEL_WIDTH_IN_TILES` and `LEVEL_HEIGHT_IN_TILES`. This is correct only while all maps share those dimensions. If maps ever differ in size, these offsets must become per-map metadata carried in `LevelTransitionEvent`.

## Task Checklist

- [x] Add `LevelTransition` component (with `Reflect` derives) to `src/level/transition.rs`
- [x] Register `LevelTransition` in `LevelPlugin` with `app.register_type::<LevelTransition>()`
- [x] Enable `user_properties` feature in `Cargo.toml` for bevy_ecs_tiled property deserialization
- [x] Add object layer with `LevelTransition` class property rectangle to `assets/map1.tmx`
- [x] Add object layer with `LevelTransition` class property rectangle to `assets/map2.tmx`
- [x] Add `setup_transition_colliders` global observer to `src/level/transition.rs`
- [x] Register `setup_transition_colliders` with `app.add_observer` in `LevelPlugin`
- [x] Add `TiledPhysicsSettings { objects_layer_filter: TiledFilter::None }` to all `TiledMap` spawns
- [x] Add `trigger_transition` system with `TransitionCooldown` to `src/level/transition.rs`
- [x] Replace `debug_trigger_transition` with `trigger_transition` in `LevelPlugin`
- [x] Add camera snap logic (`CAMERA_SNAP_THRESHOLD`) to `src/camera.rs`
- [x] Verify `LevelTransition` is present on zone entities after map load
- [x] Verify zone entities have `Sensor` and `Collider`
- [x] Verify walking into each zone triggers the correct transition
- [x] Verify T and R keys no longer trigger transitions
- [x] Verify back-and-forth transitions work without loops or camera scrolling

## Related Specs

- **016 — Map Switching**: provides `CurrentMap`, `LevelTransitionEvent`, `apply_transition`, and `on_collider_created` that this spec builds on
- **014 — Collectibles**: establishes the sensor + `CollisionStart` detection pattern reused here

## Related Files

- `src/level/transition.rs` — `LevelTransition`, `TransitionCooldown`, `setup_transition_colliders`, `trigger_transition`
- `src/level/mod.rs` — plugin registration, `TiledPhysicsSettings` on map spawns
- `src/camera.rs` — camera snap on teleport
- `assets/map1.tmx` — object layer with transition zone
- `assets/map2.tmx` — object layer with transition zone
- `Cargo.toml` — `user_properties` feature for bevy_ecs_tiled
