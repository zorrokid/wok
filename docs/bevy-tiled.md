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

The plugin emits events as it processes the map. React to them to run code once the map is ready:

```rust
// Using EventReader
fn on_map_loaded(mut events: EventReader<TiledEvent<MapCreated>>) {
    for event in events.read() {
        // Map is fully spawned; safe to query tile storage, spawn player, etc.
    }
}

// Using observer (immediate, entity-specific)
commands.entity(map_entity).observe(|trigger: Trigger<TiledEvent<MapCreated>>| {
    // Runs immediately when this specific map entity finishes loading
});
```

Available events: `MapCreated`, `LayerCreated`, `TileCreated`, `ObjectCreated`.

### Physics Collider Generation

When `TiledPhysicsPlugin` is added with the Avian backend, solid tiles automatically receive `RigidBody::Static` and `Collider` components — no manual collider placement needed.

Control which layers generate colliders via `TiledPhysicsSettings`:

```rust
commands.spawn((
    TiledMap(asset_server.load("map.tmx")),
    TilemapAnchor::Center,
    TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
        tiles_layer_filter: TiledFilter::include_layer("Ground"),
        ..default()
    },
));
```

The `TiledPhysicsAvianBackend` collider shape options:
- `Polyline` — Aggregate line strings (efficient for flat ground)
- `Triangulation` — Triangulated polygons (for complex shapes)
- `LineStrip` — One collider per line string segment

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
