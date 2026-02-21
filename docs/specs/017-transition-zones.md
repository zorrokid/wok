# Spec 017 — Transition Zones

## Status: Pending

## Overview

Replaces the temporary keyboard trigger from spec 016 with physics-based transition zones defined in the Tiled map. Each map has an object layer containing a rectangle whose transition destination is stored as a Tiled Custom Class property. bevy_ecs_tiled automatically deserializes the property into a `LevelTransition` component on the spawned object entity. An observer adds `Sensor` and `Collider` to complete the trigger zone. A `trigger_transition` system detects when the player enters the zone and fires the existing `LevelTransitionEvent`, which `apply_transition` (unchanged from spec 016) handles.

## Requirements

1. `LevelTransition { target_map: String, spawn_tile_x: f32, spawn_tile_y: f32 }` is a Bevy component that derives `Reflect` and is registered in the type registry.
2. Each map has an object layer with at least one rectangle object carrying a `LevelTransition` custom class property.
3. An observer on `TiledEvent<ObjectCreated>` detects entities that have a `LevelTransition` component (added automatically by bevy_ecs_tiled) and inserts `Sensor` and `Collider::rectangle` to make the zone physically detectable.
4. A `trigger_transition` system reads Avian2D `CollisionStart` events and sends `LevelTransitionEvent` when the player contacts a `LevelTransition` entity.
5. The temporary `debug_trigger_transition` system (T / R keys) from spec 016 is removed.
6. `apply_transition` and the rest of the map switching infrastructure from spec 016 are unchanged.

## Acceptance Criteria

- [ ] `LevelTransition` component is present on transition zone entities after map load (confirming bevy_ecs_tiled deserialization works).
- [ ] Transition zone entities have `Sensor` and `Collider` (confirming the observer fires).
- [ ] Walking into the right-edge zone on map1 transitions to map2 and places the player at the correct position.
- [ ] Walking into the left-edge zone on map2 transitions back to map1 and places the player at the correct position.
- [ ] The T and R debug keys no longer trigger transitions.
- [ ] No panics or errors on repeated back-and-forth transitions.

## Implementation Plan

### `LevelTransition` Component

Derives `Reflect` so bevy_ecs_tiled can deserialize it from Tiled class properties and insert it automatically onto spawned object entities:

```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct LevelTransition {
    /// Asset path of the target map, e.g. "map2.tmx".
    pub target_map: String,
    /// Tile column for the player spawn position in the target map.
    pub spawn_tile_x: f32,
    /// Tile row for the player spawn position in the target map.
    pub spawn_tile_y: f32,
}
```

`target_map` is `String` (not `&'static str`) because it is deserialized from Tiled at runtime.

### Tiled Setup: Custom Class Property

bevy_ecs_tiled's property system supports **Class** type properties backed by registered Rust types. Plain string/int/float properties are not supported and are silently skipped. The setup:

1. In the Tiled **Project** settings, define a new Custom Property Type (Class) named exactly: `wok::level::transition::LevelTransition`.
2. Add three fields to the class:
   - `target_map` — string
   - `spawn_tile_x` — float
   - `spawn_tile_y` — float
3. In each map's object layer, place a rectangle object and add a property of this class type with the appropriate values.

When the map loads, bevy_ecs_tiled matches the class name against the Bevy type registry and inserts the deserialized `LevelTransition` component onto the spawned object entity automatically.

Example object placements:

| Map   | Object position       | `target_map`  | `spawn_tile_x` | `spawn_tile_y` |
|-------|-----------------------|---------------|----------------|----------------|
| map1  | Right edge of map     | `"map2.tmx"`  | `2.0`          | `4.0`          |
| map2  | Left edge of map      | `"map1.tmx"`  | `28.0`         | `4.0`          |

> **Type path note**: the name in Tiled must match the full Rust type path exactly. Verify with `println!("{}", std::any::type_name::<LevelTransition>())` if deserialization silently fails.

### Observer: `setup_transition_colliders`

Fires for every spawned map object. If the entity already has a `LevelTransition` component (inserted by bevy_ecs_tiled), adds the Avian2D components needed to make the zone detectable by the player's collider:

```rust
fn setup_transition_colliders(
    ev: On<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    query: Query<(&TiledObject, Has<LevelTransition>)>,
) {
    let entity = ev.event().origin;
    let Ok((obj, has_transition)) = query.get(entity) else { return };
    if !has_transition { return; }

    let TiledObject::Rectangle { width, height } = obj else { return };
    // Sensor: player passes through rather than being physically blocked.
    // Collider dimensions match the rectangle drawn in Tiled.
    commands.entity(entity).insert((
        Sensor,
        Collider::rectangle(*width, *height),
    ));
}
```

This observer is attached to every `TiledMap` entity (both initial spawn and post-transition) alongside `on_collider_created`.

### System: `trigger_transition`

Reads `CollisionStart` events (one-shot per contact begin — fires exactly once when the player enters the zone, unlike the `Collisions` system param which fires every frame):

```rust
pub fn trigger_transition(
    mut collision_reader: MessageReader<CollisionStart>,
    player_q: Query<Entity, With<Player>>,
    transition_q: Query<&LevelTransition>,
    mut writer: EventWriter<LevelTransitionEvent>,
) {
    let Ok(player) = player_q.single() else { return };
    for event in collision_reader.read() {
        let other = if event.collider1 == player { event.collider2 }
                    else if event.collider2 == player { event.collider1 }
                    else { continue };
        if let Ok(t) = transition_q.get(other) {
            writer.write(LevelTransitionEvent {
                target_map: t.target_map.clone(),
                spawn_tile_x: t.spawn_tile_x,
                spawn_tile_y: t.spawn_tile_y,
            });
        }
    }
}
```

### Plugin Registration Changes

In `LevelPlugin::build`, replace `debug_trigger_transition` with `trigger_transition`, register `LevelTransition`, and attach `setup_transition_colliders` to map spawns:

```rust
app
    .register_type::<LevelTransition>()
    .add_event::<LevelTransitionEvent>()         // already registered in spec 016
    .add_systems(Startup, (setup_tilemap, spawn_level_bounds))
    .add_systems(Update, (
        trigger_transition,
        apply_transition.after(trigger_transition),
    ));
```

`setup_tilemap` and `apply_transition` are updated to attach `setup_transition_colliders` as a second observer on every `TiledMap` entity spawn.

### Module Structure Changes

```
src/level/
    transition.rs   Add: LevelTransition, setup_transition_colliders, trigger_transition
                    Remove: debug_trigger_transition

assets/
    map1.tmx        Add object layer with LevelTransition class property rectangle
    map2.tmx        Add object layer with LevelTransition class property rectangle
```

## Notes

- **bevy_ecs_tiled deserialization is the risk**: If the type path in Tiled doesn't match the registered Rust type exactly, bevy_ecs_tiled logs an error and skips the property — no crash, but `LevelTransition` won't be present on the entity and the zone will be inert. Check the log for deserialization errors if zones don't work.
- **`CollisionStart` vs `Collisions`**: `MessageReader<CollisionStart>` fires once per contact begin — correct for a one-shot trigger. The `Collisions` system param (used by the damage system) fires every frame while in contact, which would cause repeated transitions.
- **Both observers on every map spawn**: `on_collider_created` and `setup_transition_colliders` must both be attached when spawning a `TiledMap` entity — in `setup_tilemap` and in `apply_transition`.
- **`CollisionEventsEnabled` not needed**: The player's dynamic collider and the transition zone's sensor collider generate `CollisionStart` events without `CollisionEventsEnabled` because the sensor relationship is sufficient for Avian2D to emit the event.

## Task Checklist

- [ ] Add `LevelTransition` component (with `Reflect` derives) to `src/level/transition.rs`
- [ ] Register `LevelTransition` in `LevelPlugin` with `app.register_type::<LevelTransition>()`
- [ ] Define `LevelTransition` Custom Property Type in Tiled project settings
- [ ] Add object layer with `LevelTransition` class property rectangle to `assets/map1.tmx`
- [ ] Add object layer with `LevelTransition` class property rectangle to `assets/map2.tmx`
- [ ] Add `setup_transition_colliders` observer to `src/level/transition.rs`
- [ ] Attach `setup_transition_colliders` to `TiledMap` spawn in `setup_tilemap` and `apply_transition`
- [ ] Add `trigger_transition` system to `src/level/transition.rs`
- [ ] Replace `debug_trigger_transition` with `trigger_transition` in `LevelPlugin`
- [ ] Verify `LevelTransition` is present on zone entities after map load
- [ ] Verify zone entities have `Sensor` and `Collider`
- [ ] Verify walking into each zone triggers the correct transition
- [ ] Verify T and R keys no longer trigger transitions
- [ ] Mark spec complete after user verification

## Related Specs

- **016 — Map Switching**: provides `CurrentMap`, `LevelTransitionEvent`, `apply_transition`, and `on_collider_created` that this spec builds on
- **014 — Collectibles**: establishes the sensor + `CollisionStart` detection pattern reused here

## Related Files

- `src/level/transition.rs` — add `LevelTransition`, `setup_transition_colliders`, `trigger_transition`; remove `debug_trigger_transition`
- `src/level/mod.rs` — register `LevelTransition` type; attach `setup_transition_colliders` observer; swap system registration
- `assets/map1.tmx` — add object layer with transition zone
- `assets/map2.tmx` — add object layer with transition zone
