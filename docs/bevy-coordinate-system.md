# Bevy Coordinate System Guide

## Overview
Bevy uses different coordinate systems for 2D and 3D, but both are right-handed coordinate systems.

## 2D Coordinate System (for our Metroidvania game)

### Axes
- **X-axis**: Horizontal axis
  - Positive X = Right
  - Negative X = Left
- **Y-axis**: Vertical axis
  - Positive Y = Up
  - Negative Y = Down
- **Z-axis**: Depth/layering axis
  - Higher Z values render on top (closer to camera)
  - Lower Z values render behind
  - Typical range: -999.9 to 999.9

### Origin Point
- Default origin (0, 0) is at the **center of the screen**
- Not at top-left like some 2D frameworks
- The camera's position determines what part of the world is visible

### Camera Coordinates
```rust
// Camera at (0, 0) shows world origin at screen center
Camera2d::default()

// Camera at (100, 200) shows that world position at screen center
Camera2d { translation: Vec3::new(100.0, 200.0, 0.0), .. }
```

### Sprite Positioning
```rust
// Sprite at origin (center of screen by default)
Transform::from_xyz(0.0, 0.0, 0.0)

// Sprite 100 pixels right, 50 pixels up from origin
Transform::from_xyz(100.0, 50.0, 0.0)

// Background layer (renders behind)
Transform::from_xyz(0.0, 0.0, -10.0)

// Foreground/UI layer (renders in front)
Transform::from_xyz(0.0, 0.0, 10.0)
```

### Common Gotchas

#### 1. Screen vs World Coordinates
- **Screen coordinates**: Pixels on your monitor, origin at center
- **World coordinates**: Position in game world, camera determines view
- Mouse/cursor positions need conversion from screen to world space

#### 2. Y-axis is Up (not down)
- Unlike some 2D frameworks where Y goes down
- Bevy's Y-axis points up, matching mathematical conventions
- Gravity should subtract from Y (velocity.y -= GRAVITY)

#### 3. Anchor Points
- Sprites are centered by default on their position
- A sprite at (100, 100) has its center at that position
- Use `Anchor` component to change this behavior

#### 4. Tile Coordinates vs World Coordinates

See the dedicated [Tilemap Coordinate System](#tilemap-coordinate-system-bevy_ecs_tilemap) section below for how `bevy_ecs_tilemap` handles tile-to-world mapping and how our collision system stays in sync.

## 3D Coordinate System (for reference)

### Axes (Right-handed system)
- **X-axis**: Points right
- **Y-axis**: Points up
- **Z-axis**: Points toward the camera (out of screen)
  - Negative Z is away from camera
  - Positive Z is toward camera

This follows OpenGL conventions.

## Transform Component

The `Transform` component is how positions are represented:

```rust
pub struct Transform {
    pub translation: Vec3,  // Position (x, y, z)
    pub rotation: Quat,     // Rotation (rarely used in 2D)
    pub scale: Vec3,        // Scale (1.0 = normal size)
}
```

### Common Operations
```rust
// Moving right
transform.translation.x += 10.0;

// Moving up
transform.translation.y += 5.0;

// Checking if above a position
if transform.translation.y > ground_level {
    // In the air
}

// Distance between two entities
let distance = transform1.translation.distance(transform2.translation);
```

## Practical Examples for Platformers

### Ground Level
```rust
// Ground is typically at a negative Y value
const GROUND_Y: f32 = -100.0;

// Player spawns above ground
Transform::from_xyz(0.0, GROUND_Y + 50.0, 0.0)
```

### Gravity
```rust
// Gravity pulls down (negative Y direction)
velocity.y -= GRAVITY * delta_time;
transform.translation.y += velocity.y * delta_time;
```

### Platform Collision
```rust
// Player is above platform if their Y is greater
if player.translation.y > platform.translation.y {
    // Player is on top
}
```

### Camera Follow
```rust
// Camera follows player by matching Y position
camera.translation.x = player.translation.x;
camera.translation.y = player.translation.y;
// Camera Z should stay at a positive value to see the world
camera.translation.z = 999.9;
```

## Coordinate Conversion Helpers

### World to Tile

Our `world_to_tile_coords` assumes tiles are corner-aligned (left/bottom edge at `TILEMAP_OFFSET + tile_index * TILE_SIZE`):

```rust
pub fn world_to_tile_coords(world_x: f32, world_y: f32) -> (i32, i32) {
    let tile_x = ((world_x - TILEMAP_OFFSET_X) / TILE_SIZE).floor() as i32;
    let tile_y = ((world_y - TILEMAP_OFFSET_Y) / TILE_SIZE).floor() as i32;
    (tile_x, tile_y)
}
```

This works correctly **only** when the tilemap uses `TilemapAnchor::BottomLeft`, which aligns tile edges with the collision grid. See [Tilemap Coordinate System](#tilemap-coordinate-system-bevy_ecs_tilemap) for details.

### Tile to World (top of tile for ground snapping)
```rust
// Top edge of a tile (used for ground_snap_y)
let tile_top_y = TILEMAP_OFFSET_Y + ((tile_y + 1) as f32 * TILE_SIZE);
// Player center snaps to tile top + half sprite height
let snap_y = tile_top_y + SPRITE_HEIGHT / 2.0;
```

## Tilemap Coordinate System (bevy_ecs_tilemap)

### How bevy_ecs_tilemap Positions Tiles

`bevy_ecs_tilemap` positions each tile by computing its **center** in tilemap-local space. For a square tilemap:

```
Tile (x, y) center = (grid_size.x * x, grid_size.y * y)
```

The tilemap's `Transform` component positions the tilemap in world space, and the `TilemapAnchor` controls which point of the tilemap sits at the transform position.

### TilemapAnchor and Why It Matters

The anchor determines the relationship between the tilemap's `Transform` position and where tiles actually render:

| Anchor | Transform position corresponds to |
|---|---|
| `None` (default) | Center of tile (0,0) |
| `BottomLeft` | Bottom-left corner of the tilemap |
| `Center` | Center of the entire tilemap |

**We use `TilemapAnchor::BottomLeft`** so that the tilemap's transform position equals `TILEMAP_OFFSET`, and tile edges align with the collision grid:

```
Tilemap transform = (TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y)

Tile (x, y) left edge  = TILEMAP_OFFSET_X + x * TILE_SIZE
Tile (x, y) right edge = TILEMAP_OFFSET_X + (x + 1) * TILE_SIZE
Tile (x, y) bottom edge = TILEMAP_OFFSET_Y + y * TILE_SIZE
Tile (x, y) top edge    = TILEMAP_OFFSET_Y + (y + 1) * TILE_SIZE
```

This matches exactly what `world_to_tile_coords` computes with `floor()`.

### Why Not TilemapAnchor::None (the default)?

With the default `TilemapAnchor::None`, tile (0,0) is **centered** at the transform position. This means tile edges are offset by half a tile from where `world_to_tile_coords` expects them, causing an 8-pixel misalignment between visual tiles and collision detection. The player would fall off platform edges too early on one side.

### Visual Layout

```
With TilemapAnchor::BottomLeft and transform at (-160, -120):

World X:  -160      -144      -128      -112
           |         |         |         |
           | Tile 0  | Tile 1  | Tile 2  |  ...
           |         |         |         |
           ^         ^         ^
         left edge  left edge  left edge

Collision grid boundaries match tile visual edges exactly.
```

### LEVEL_DATA Array Mapping

The `LEVEL_DATA` 2D array uses array index 0 as the **top** of the level, but `bevy_ecs_tilemap` uses Y=0 as the **bottom**. The conversion is:

```
array_y = (LEVEL_HEIGHT_IN_TILES - 1) - tile_y

LEVEL_DATA[0][..]  → top of level    (tile_y = 14)
LEVEL_DATA[14][..] → bottom of level (tile_y = 0)
```

### Built-in Coordinate Conversion APIs

`bevy_ecs_tilemap` provides built-in methods for coordinate conversion that handle anchoring automatically:

```rust
// World position → tile position (returns None if out of bounds)
let tile_pos = TilePos::from_world_pos(
    &world_pos,    // Vec2
    &map_size,     // TilemapSize
    &grid_size,    // TilemapGridSize
    &tile_size,    // TilemapTileSize
    &map_type,     // TilemapType
    &anchor,       // TilemapAnchor
);

// Tile position → world center
let world_center: Vec2 = tile_pos.center_in_world(
    &map_size, &grid_size, &tile_size, &map_type, &anchor,
);
```

We currently use our own `world_to_tile_coords` for simplicity (it doesn't require passing all the tilemap parameters), but these built-in APIs are the canonical way to convert coordinates and are guaranteed to match the rendering.

## Visualization

```
      Y (up)
      ^
      |
      |     • (100, 200)
      |
      |
------+-------> X (right)
(0,0) |
      |
      |  • (50, -100)
      |
      v
   Y (down in other frameworks)

Z-axis (not shown): Points out of screen toward you
```

## Resources
- [Bevy Transform Docs](https://docs.rs/bevy/latest/bevy/transform/components/struct.Transform.html)
- [Bevy Coordinate System](https://bevyengine.org/learn/book/getting-started/ecs/)
