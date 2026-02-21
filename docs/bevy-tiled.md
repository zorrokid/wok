# Bevy 0.18 — Tilemap and Coordinates

Covers the coordinate system used in Bevy 2D, how `bevy_ecs_tiled` loads Tiled maps, and how world space and tile space relate to each other. Read this when working on level loading, tile layout, camera positioning, or any code that converts between world and tile coordinates.

---

## Coordinate System

### 2D Axes

Bevy uses a right-handed coordinate system where Y points **up** (unlike many 2D frameworks where Y points down).

```
      Y (up)
      ^
      |
      |     • (100, 200)
      |
------+-------> X (right)
(0,0) |
      |
      |  • (50, -100)
      v
```

- **X**: Positive = right, Negative = left
- **Y**: Positive = up, Negative = down
- **Z**: Depth layer. Higher Z renders on top. Typical range −999 to +999.
- **Origin (0, 0)**: Center of the screen by default.

Gravity subtracts from Y: `velocity.y -= GRAVITY * delta`.

### Transform Component

All positioned entities have a `Transform`:

```rust
pub struct Transform {
    pub translation: Vec3,  // Position (x, y, z)
    pub rotation: Quat,     // Rotation (rarely used in 2D)
    pub scale: Vec3,        // Scale (1.0 = normal)
}

Transform::from_xyz(100.0, 50.0, 0.0)
transform.translation.x += velocity.x * delta;
```

Sprites are **centered** on their `Transform` position by default. A 16×16 sprite at `(0, 0)` spans from `(-8, -8)` to `(8, 8)` in world space.

---

## Tilemap Coordinates

### Tile Grid

`bevy_ecs_tiled` (via `bevy_ecs_tilemap`) uses a tile grid where tile `(0, 0)` is the **bottom-left** tile — Y increases upward, consistent with Bevy's world space.

**Note on Tiled's Y axis:** The TMX file format stores rows top-to-bottom (row 0 = top of map). `bevy_ecs_tiled` handles this inversion automatically — tiles render correctly without manual flipping.

### TilemapAnchor

The `TilemapAnchor` controls which point of the tilemap aligns with the tilemap entity's `Transform` position:

| Anchor | Transform aligns to |
|---|---|
| `TilemapAnchor::Center` | Center of the entire tilemap |
| `TilemapAnchor::BottomLeft` | Bottom-left corner of the tilemap |
| `TilemapAnchor::None` (default) | Center of tile (0, 0) |

**This project uses `TilemapAnchor::Center`** with the map entity at the world origin, which means:

```
For a 20×15 map at 16px/tile:

Bottom-left corner = (-160, -120)
Tile (x, y) left edge   = -160 + x * 16
Tile (x, y) bottom edge = -120 + y * 16
Tile (x, y) center      = (-160 + x * 16 + 8, -120 + y * 16 + 8)
```

Choosing the wrong anchor causes a systematic offset between where tiles visually appear and where collision code expects them. `TilemapAnchor::None` (the default) offsets by half a tile, causing edge-case collision bugs.

### Coordinate Conversion API

`bevy_ecs_tilemap` provides built-in conversion that handles anchoring correctly:

```rust
// World position → tile position (returns None if out of bounds)
let tile_pos = TilePos::from_world_pos(
    &world_pos,   // Vec2
    &map_size,    // TilemapSize
    &grid_size,   // TilemapGridSize
    &tile_size,   // TilemapTileSize
    &map_type,    // TilemapType
    &anchor,      // TilemapAnchor
);

// Tile position → world center of that tile
let world_center: Vec2 = tile_pos.center_in_world(
    &map_size, &grid_size, &tile_size, &map_type, &anchor,
);
```

Prefer these built-in APIs over custom math — they stay correct if tile size, anchor, or map size changes.

---

## bevy_ecs_tiled

`bevy_ecs_tiled` (v0.11.2) bridges the Tiled map editor and Bevy's ECS. It parses `.tmx` files and spawns entities representing the map, layers, and tiles.

### Plugin Setup

```rust
use bevy_ecs_tiled::prelude::*;

app.add_plugins(TiledPlugin::default());

// Also add if you want automatic physics colliders from tile layers:
app.add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default());
```

**`user_properties` feature required for Tiled class properties:** If you use Custom Class properties to attach Rust components via bevy_ecs_tiled, you must enable the `user_properties` feature in `Cargo.toml`. Without it, the entire deserialization block is compiled out — properties are **silently ignored** with no error or warning:

```toml
bevy_ecs_tiled = { version = "0.11.2", features = ["avian", "user_properties"] }
```

### Loading a Map

Spawn an entity with the `TiledMap` component holding an asset handle. The plugin detects it and spawns the full map hierarchy as child entities:

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    commands.spawn((
        TiledMap(asset_server.load("map.tmx")),
        TilemapAnchor::Center,
    ));
}
```

### Lifecycle Events

The plugin emits events as it processes the map. Each event is sent both as a **message** (readable globally by any system) and as an **entity observer trigger**. Use whichever fits the architecture:

```rust
// MessageReader — global, readable in any system, any plugin.
// Correct for ColliderCreated, LayerCreated — events that each system
// needs to react to independently.
fn on_object_spawned(mut reader: MessageReader<TiledEvent<ObjectCreated>>) {
    for ev in reader.read() {
        let entity = ev.origin;
        // entity now has TiledObject and TiledName components
    }
}

// Global app observer (app.add_observer) — fires for every trigger in the app.
// Use this for ObjectCreated when you need to react to child object entities:
// per-entity .observe() on a TiledMap entity does NOT receive triggers
// for its child object entities — the trigger must be global.
app.add_observer(setup_zones);

fn setup_zones(ev: On<TiledEvent<ObjectCreated>>, mut commands: Commands, ...) {
    let entity = ev.event().origin;
    // entity is the object child, not the TiledMap entity
}
```

**Do not use `EventReader`** — these are bevy_ecs_tiled messages, not standard Bevy events. `MessageReader` is the correct type.

**Per-entity `.observe()` on `TiledMap` does not receive `ObjectCreated` for child objects.** When you call `.observe(my_fn)` on a `TiledMap` entity, the observer only fires for triggers sent directly to that entity — not for triggers sent to its children. `ObjectCreated` is triggered on the object entity itself (a child), so only a global observer (`app.add_observer`) will see it.

Available events: `MapCreated`, `LayerCreated`, `TilemapCreated`, `TileCreated`, `ObjectCreated`, `ColliderCreated`.

### Physics Collider Generation

When `TiledPhysicsPlugin` is added with the Avian backend, solid tiles automatically receive `RigidBody::Static` and `Collider` components — no manual collider placement needed.

**`collider_from_object` creates solid colliders for ALL object-layer objects by default.** The physics backend runs two systems in `PreUpdate`: `collider_from_tiles_layer` (for tile layers) and `collider_from_object` (for object layers). Both use `TiledFilter::All` by default, which matches everything. This means every rectangle object in every object layer gets a solid `RigidBody::Static` + `Collider` — including objects you intend as sensor zones or spawn markers. If you have object-layer objects that should **not** be solid physics bodies, disable object collider generation:

```rust
commands.spawn((
    TiledMap(asset_server.load("map.tmx")),
    TilemapAnchor::Center,
    TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
        // Disable automatic solid colliders for object-layer objects.
        // Create colliders manually in an observer for objects that need them.
        objects_layer_filter: TiledFilter::None,
        ..Default::default()
    },
));
```

This setting must be added **at spawn time**. If you omit it, `initialize_settings_for_maps` inserts the default (all-matching) settings in `PreUpdate` before `collider_from_object` reads the events.

Control which layers generate tile colliders via `tiles_layer_filter`:

```rust
TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
    tiles_layer_filter: TiledFilter::Names(vec!["Ground".into()]),
    objects_layer_filter: TiledFilter::None,
    ..Default::default()
}
```

The `TiledPhysicsAvianBackend` collider shape options:
- `Polyline` — Aggregate line strings (efficient for flat ground)
- `Triangulation` — Triangulated polygons (for complex shapes)
- `LineStrip` — One collider per line string segment

### Object Layers

Object layers in Tiled spawn as ECS entities. Each spawned object entity has:

- **`TiledObject`** — describes the shape:
  ```rust
  enum TiledObject {
      Point,
      Rectangle { width: f32, height: f32 },
      Ellipse   { width: f32, height: f32 },
      Polygon   { vertices: Vec<Vec2> },
      Polyline  { vertices: Vec<Vec2> },
      Tile      { width: f32, height: f32 },
      Text,
  }
  ```
  `width` and `height` are in pixels, matching the rectangle drawn in the Tiled editor.

- **`TiledName`** — a `String` wrapper holding the object's name from Tiled.
- **`Transform`** — world position of the object's **top-left corner** (not center). This matches Tiled's coordinate convention (origin at top-left of each object). Avian2D's `Collider::rectangle(w, h)` is centered on the entity's `Transform`, so if you add a collider using the object's dimensions, you must offset the Transform by `+width/2` in X and `-height/2` in Y to center the collider on the drawn rectangle:

```rust
// bevy_ecs_tiled places Transform at the top-left of the Tiled rectangle.
// Avian2D centers Collider on Transform.
// Shift Transform to the rectangle center before inserting the collider.
let center = Transform::from_xyz(
    transform.translation.x + width / 2.0,
    transform.translation.y - height / 2.0,  // Y negated: world Y is up, Tiled Y is down
    transform.translation.z,
);
commands.entity(entity).insert((
    center,
    Collider::rectangle(width, height),
));
```

React to spawned objects via `MessageReader<TiledEvent<ObjectCreated>>`:

```rust
fn setup_objects(
    mut reader: MessageReader<TiledEvent<ObjectCreated>>,
    mut commands: Commands,
    query: Query<(&TiledObject, &TiledName, Has<MyComponent>)>,
) {
    for ev in reader.read() {
        let entity = ev.origin;
        let Ok((obj, name, has_my_comp)) = query.get(entity) else { continue };

        // Option A: filter by name
        if name.0 == "some_object_name" { ... }

        // Option B: filter by component (added via custom class property)
        if has_my_comp { ... }

        // Use Tiled rectangle dimensions for collider and sprite
        if let TiledObject::Rectangle { width, height } = obj {
            commands.entity(entity).insert(Collider::rectangle(*width, *height));
        }
    }
}
```

Objects are spawned as children of the map entity hierarchy. When the map entity is despawned, all its objects are despawned automatically.

---

### Custom Class Properties

bevy_ecs_tiled can automatically insert Rust components onto spawned entities from Tiled **Custom Class** properties. This is the recommended way to attach game data (entity type, behaviour settings) to map objects.

**What is supported:** `ClassValue` properties only — a Tiled Custom Property Type (Class) whose name exactly matches a registered Rust type path.

**What is NOT supported:** Plain string, int, float, bool, or color properties on objects. These generate a warning log and are silently skipped.

#### Workflow

1. **Define the Rust component** with `Reflect` and register it:

```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct MyMarker;

// In plugin:
app.register_type::<MyMarker>();
```

2. **In Tiled**, open Project → Custom Types and create a new Class. The class name must be the **exact Rust type path**: `mycrate::mymodule::MyMarker`.

   For a struct with fields, add matching field names and types:
   ```rust
   #[derive(Component, Reflect, Default)]
   #[reflect(Component, Default)]
   pub struct SpawnConfig {
       pub speed: f32,
       pub count: i32,
   }
   ```
   The Tiled class would have fields `speed` (float) and `count` (int).

   > **Tiled 1.11 note:** The Custom Types UI may not be accessible in all Tiled 1.11 builds. If the menu is unavailable, edit the `.tmx` file directly (see TMX format below).

3. **In the Tiled map**, select an object and add a property of the class type. bevy_ecs_tiled reads this on load and inserts the deserialized component onto the spawned entity — no observer or extra code needed for the component itself.

   If adding properties via Tiled's GUI is not possible, add the XML directly to the `.tmx` file. The `propertytype` attribute must match the full Rust type path exactly:

   ```xml
   <object id="1" x="100" y="0" width="80" height="480">
     <properties>
       <property name="mycrate::mymodule::SpawnConfig"
                 type="class"
                 propertytype="mycrate::mymodule::SpawnConfig">
         <properties>
           <property name="speed" type="float" value="5.0"/>
           <property name="count" type="int" value="3"/>
         </properties>
       </property>
     </properties>
   </object>
   ```

#### Type Path Requirement

The type path includes the crate name and full module path, e.g. `wok::level::transition::LevelTransition`. If the name in Tiled doesn't match exactly, bevy_ecs_tiled logs an error and skips the property — no crash, but the component won't be present. Verify the path with:

```rust
println!("{}", std::any::type_name::<MyComponent>());
```

#### Supported Rust Field Types

Primitive fields supported in class properties: `bool`, `i8`–`i128`, `u8`–`u128`, `f32`, `f64`, `String`, `char`. Enums (unit variants) map to Tiled string values. Nested structs map to nested Tiled class values.

---

### Entity Hierarchy and Map Cleanup

bevy_ecs_tiled builds a parent–child hierarchy under the `TiledMap` entity:

```
TiledMap entity
└── Layer entity (one per layer)
    ├── Tile entity (one per tile with data)
    └── Object entity (one per object in object layers)
```

To despawn a map and all its tiles, objects, and colliders:

```rust
commands.entity(map_entity).despawn_children();
commands.entity(map_entity).despawn();
```

`despawn_children()` despawns all child entities recursively. The separate `despawn()` call then removes the map entity itself. Both calls are deferred (applied at the next command flush), so there is one frame where both the old and new map coexist if you spawn the replacement map in the same system.

This is the correct pattern for level transitions. Gameplay entities (enemies, collectibles) that are spawned as object children are cleaned up automatically.

---

### TMX Map Format

The `.tmx` format is XML. Key attributes:

```xml
<map orientation="orthogonal" width="20" height="15"
     tilewidth="16" tileheight="16" infinite="0">
  <tileset firstgid="1" source="tileset.tsx"/>
  <layer name="Ground" width="20" height="15">
    <data encoding="csv">
      <!-- 20×15 grid of tile IDs; 0 = empty, 1+ = tile from tileset -->
    </data>
  </layer>
</map>
```

Tile ID `0` means empty. Non-zero IDs reference the tileset (1-indexed via `firstgid`).
