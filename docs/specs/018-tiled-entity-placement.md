# Spec 018 — Tiled Entity Placement

## Status: Pending

## Overview

Migrates collectibles and enemies from hardcoded Rust startup systems to Tiled object layer definitions. Each entity type is a registered Rust component; bevy_ecs_tiled deserializes the matching Tiled class property and inserts the component automatically onto the spawned object entity. A setup system in each plugin reads `TiledEvent<ObjectCreated>` messages and adds the visual and physics components. Because bevy_ecs_tiled spawns objects as children of the map entity, collectibles and enemies are automatically cleaned up when the map despawns during a transition — no explicit per-map tracking needed.

## Requirements

1. `Collectible` and `Enemy` derive `Reflect` and are registered in the type registry so bevy_ecs_tiled can deserialize them from Tiled class properties.
2. Each map's object layer contains rectangle objects with either a `wok::collectibles::Collectible` or `wok::enemies::Enemy` class property.
3. `CollectiblesPlugin` registers a `setup_collectible_objects` system that reads `TiledEvent<ObjectCreated>` messages, detects entities with `Collectible`, and adds `Sprite`, `Sensor`, `Collider`, and `CollisionEventsEnabled`.
4. `EnemiesPlugin` registers a `setup_enemy_objects` system that reads `TiledEvent<ObjectCreated>` messages, detects entities with `Enemy`, and adds `Sprite`, `RigidBody::Static`, and `Collider`.
5. The hardcoded `COLLECTIBLE_POSITIONS`, `ENEMY_POSITIONS`, `spawn_collectibles`, and `spawn_enemies` are removed.
6. The `collect_items` and `contact_damage` systems are unchanged — they still query by `Collectible` and `Enemy` components.
7. Collectibles and enemies are defined per-map in the Tiled editor. Both maps can have different sets.

## Acceptance Criteria

- [ ] Collectibles appear at positions defined in the Tiled map, not hardcoded positions.
- [ ] Enemies appear at positions defined in the Tiled map.
- [ ] Walking into a collectible still removes it.
- [ ] Walking into an enemy still deals damage.
- [ ] Transitioning to a new map removes the previous map's collectibles and enemies automatically.
- [ ] Each map can have its own independent set of collectibles and enemies.
- [ ] No panics or errors on transition or repeated collection.

## Implementation Plan

### Component Changes

Both `Collectible` and `Enemy` gain `Reflect` derives and type registration. Since they are marker components with no fields, the Tiled class type has no fields either — just the type name is enough for bevy_ecs_tiled to insert the component:

```rust
// src/collectibles/mod.rs
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Collectible;

// src/enemies/mod.rs
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Enemy;
```

### Tiled Setup

In each map's **object layer** (e.g. named `"Entities"`), place rectangle objects:

- For a collectible: assign class property `wok::collectibles::Collectible` (no fields)
- For an enemy: assign class property `wok::enemies::Enemy` (no fields)

The rectangle's position and dimensions in Tiled define the entity's world position and collider/sprite size. This means the level designer controls placement visually in the Tiled editor.

### `setup_collectible_objects`

Reads `TiledEvent<ObjectCreated>` messages each frame. When an entity with `Collectible` is found, adds the visual and physics components. Sprite size and collider dimensions are taken from the Tiled rectangle so the designer controls the size:

```rust
fn setup_collectible_objects(
    mut commands: Commands,
    mut reader: MessageReader<TiledEvent<ObjectCreated>>,
    query: Query<(&TiledObject, Has<Collectible>)>,
) {
    for ev in reader.read() {
        let entity = ev.origin;
        let Ok((obj, has_collectible)) = query.get(entity) else { continue };
        if !has_collectible { continue; }

        let TiledObject::Rectangle { width, height } = obj else { continue };
        commands.entity(entity).insert((
            Sprite {
                color: Color::srgb(1.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(*width, *height)),
                ..default()
            },
            Collider::rectangle(*width, *height),
            Sensor,
            // Required for Avian2D to emit CollisionStart events for this entity.
            CollisionEventsEnabled,
        ));
    }
}
```

### `setup_enemy_objects`

Same pattern for enemies — reads messages, checks for `Enemy`, adds a red sprite and solid physics:

```rust
fn setup_enemy_objects(
    mut commands: Commands,
    mut reader: MessageReader<TiledEvent<ObjectCreated>>,
    query: Query<(&TiledObject, Has<Enemy>)>,
) {
    for ev in reader.read() {
        let entity = ev.origin;
        let Ok((obj, has_enemy)) = query.get(entity) else { continue };
        if !has_enemy { continue; }

        let TiledObject::Rectangle { width, height } = obj else { continue };
        commands.entity(entity).insert((
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(*width, *height)),
                ..default()
            },
            RigidBody::Static,
            Collider::rectangle(*width, *height),
        ));
    }
}
```

### Why `MessageReader` Instead of Entity Observers

bevy_ecs_tiled fires `TiledEvent<ObjectCreated>` both as an entity-specific trigger (observer) and as a global message. Entity observers must be attached to the `TiledMap` entity at spawn time, which would require `LevelPlugin` to import and reference every entity type — coupling level loading to gameplay systems.

`MessageReader<TiledEvent<ObjectCreated>>` is readable in any system from any plugin. Each plugin handles its own entity type independently, keeping modules decoupled. `LevelPlugin` needs no knowledge of collectibles or enemies.

### Plugin Registration

```rust
// CollectiblesPlugin
app.register_type::<Collectible>()
   .add_systems(Update, (setup_collectible_objects, collect_items));

// EnemiesPlugin
app.register_type::<Enemy>()
   .add_systems(Update, (setup_enemy_objects, contact_damage));
```

`spawn_collectibles` and `spawn_enemies` startup systems are removed. `COLLECTIBLE_POSITIONS`, `ENEMY_POSITIONS`, and the `src/level/tile` imports they used are also removed from both modules.

### Automatic Cleanup

bevy_ecs_tiled spawns object entities as children of the map entity hierarchy. When `apply_transition` despawns the old `TiledMap` entity and its children, all collectibles and enemies in that map are despawned along with it. No explicit per-map tracking is needed.

### Module Structure

```
src/collectibles/
    mod.rs          Collectible gains Reflect; remove COLLECTIBLE_POSITIONS and
                    spawn_collectibles; add setup_collectible_objects; register type

src/enemies/
    mod.rs          Enemy gains Reflect; remove ENEMY_POSITIONS and spawn_enemies;
                    add setup_enemy_objects; register type
    damage.rs       Unchanged

assets/
    map1.tmx        Add/update object layer with collectible and enemy rectangles
    map2.tmx        Add object layer with its own collectibles and enemies (can differ)
```

## Notes

- **Sprite `Transform` from Tiled**: bevy_ecs_tiled sets the `Transform` on spawned object entities from the object's position in the map. No manual world coordinate calculation is needed — the existing `TILEMAP_OFFSET_X/Y` conversion logic in `spawn_collectibles` and `spawn_enemies` is replaced entirely by Tiled's own coordinate system.
- **`TiledObject::Rectangle` only**: the setup systems skip non-rectangle objects silently. If a collectible or enemy object is accidentally placed as a point or polygon in Tiled, it will have the component but no physics — easy to spot and fix in the editor.
- **Collect items and contact damage unchanged**: `collect_items` queries `With<Collectible>` and `contact_damage` uses the `Collisions` system param — both work regardless of how the entity was spawned.
- **`CollisionEventsEnabled` on collectibles only**: collectibles use `MessageReader<CollisionStart>` which requires `CollisionEventsEnabled`. Enemies use the `Collisions` system param which does not require it, so it is intentionally omitted from `setup_enemy_objects`.
- **Per-map entity sets**: map1 and map2 can have completely different collectibles and enemies — just place different objects in each map's Tiled file. No Rust code change needed to adjust placement.
- **Future enemy types**: when moving enemies are added, a new component (e.g. `Patrol`) is defined, registered, and placed as an additional class property on the same rectangle object. `setup_enemy_objects` can be extended to check for `Patrol` and set up `RigidBody::Dynamic` accordingly. The `Enemy` marker stays as the shared identifier for damage detection.

## Task Checklist

- [ ] Add `Reflect` derives and `#[reflect(Component, Default)]` to `Collectible` in `src/collectibles/mod.rs`
- [ ] Add `Reflect` derives and `#[reflect(Component, Default)]` to `Enemy` in `src/enemies/mod.rs`
- [ ] Register `Collectible` type in `CollectiblesPlugin` with `app.register_type::<Collectible>()`
- [ ] Register `Enemy` type in `EnemiesPlugin` with `app.register_type::<Enemy>()`
- [ ] Add `setup_collectible_objects` system to `src/collectibles/mod.rs`
- [ ] Add `setup_enemy_objects` system to `src/enemies/mod.rs`
- [ ] Remove `COLLECTIBLE_POSITIONS`, `spawn_collectibles`, and related imports from `src/collectibles/mod.rs`
- [ ] Remove `ENEMY_POSITIONS`, `spawn_enemies`, and related imports from `src/enemies/mod.rs`
- [ ] Define `wok::collectibles::Collectible` Custom Property Type in Tiled project settings
- [ ] Define `wok::enemies::Enemy` Custom Property Type in Tiled project settings
- [ ] Place collectible and enemy rectangle objects in `assets/map1.tmx` object layer
- [ ] Place collectible and enemy rectangle objects in `assets/map2.tmx` object layer
- [ ] Verify collectibles appear at Tiled-defined positions on map load
- [ ] Verify enemies appear at Tiled-defined positions on map load
- [ ] Verify transitioning to map2 removes map1's collectibles and enemies
- [ ] Verify gameplay (collection, damage) works as before
- [ ] Mark spec complete after user verification

## Related Specs

- **017 — Transition Zones**: establishes the Tiled class property + `TiledEvent<ObjectCreated>` pattern this spec extends to collectibles and enemies
- **014 — Collectibles**: original implementation being replaced
- **015 — Enemy Hazards**: original implementation being replaced
- **016 — Map Switching**: provides the `apply_transition` despawn that makes automatic entity cleanup work

## Related Files

- `src/collectibles/mod.rs` — Reflect derives, type registration, remove spawn system, add setup system
- `src/enemies/mod.rs` — Reflect derives, type registration, remove spawn system, add setup system
- `assets/map1.tmx` — add/update object layer with collectible and enemy objects
- `assets/map2.tmx` — add object layer with its own entity set
