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
```rust
// Tile coordinates (grid-based, usually integers)
let tile_x = 5;
let tile_y = 3;

// Convert to world coordinates
let world_x = tile_x as f32 * TILE_SIZE;
let world_y = tile_y as f32 * TILE_SIZE;

// Note: Tile (0, 0) might be at world (0, 0) or offset depending on level design
```

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
```rust
pub fn world_to_tile_coords(world_x: f32, world_y: f32) -> (i32, i32) {
    let tile_x = (world_x / TILE_SIZE).floor() as i32;
    let tile_y = (world_y / TILE_SIZE).floor() as i32;
    (tile_x, tile_y)
}
```

### Tile to World (center of tile)
```rust
pub fn tile_to_world_coords(tile_x: i32, tile_y: i32) -> (f32, f32) {
    let world_x = tile_x as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let world_y = tile_y as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    (world_x, world_y)
}
```

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
