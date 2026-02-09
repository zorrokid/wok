# Basic Tilemap System

## Overview
Implement a simple tile-based level rendering system using bevy_ecs_tilemap. Tiles are 16x16 pixels and defined in hand-coded Rust arrays for quick prototyping. The system supports a single layer for rendering solid platform tiles.

## Requirements
- Add bevy_ecs_tilemap dependency to the project
- Define a simple tileset (empty, solid platform tile)
- Create a hand-coded level with tile data as Rust array
- Render the tilemap on screen
- Tiles should be 16x16 pixels
- Camera should show the level clearly

## Acceptance Criteria
- [ ] bevy_ecs_tilemap crate is added to Cargo.toml
- [ ] Tilemap renders on screen with visible tiles
- [ ] Level is defined as a 2D array in code
- [ ] At least two tile types exist (empty/air and solid platform)
- [ ] Tiles are 16x16 pixels each
- [ ] Camera is positioned to view the tilemap

---

## Implementation Plan

### Approach
Use bevy_ecs_tilemap crate for efficient tile rendering. Create a simple prototype level with hand-coded tile data stored in a constant array. Start with a minimal tileset (just empty and solid tiles) that can be expanded later. Use colored rectangles as placeholder graphics until proper sprites are added.

### Components & Systems
**Plugin:**
- `TilemapPlugin` from bevy_ecs_tilemap - handles tile rendering

**Resources:**
- Level data as constant 2D array (e.g., `const LEVEL_1: [[u32; WIDTH]; HEIGHT]`)

**Systems:**
- `setup_tilemap()` - Startup system to spawn tilemap from level data
- Configures tile size, map dimensions, tileset texture

**Entities/Components:**
- `TilemapBundle` - Main tilemap entity
- `TileStorage` - Stores tile entity references
- Individual tile entities with positions

### Tasks
- [ ] Add bevy_ecs_tilemap to Cargo.toml (check compatible version for Bevy 0.18)
- [ ] Create placeholder tileset texture (2 tiles: empty=transparent, solid=colored)
- [ ] Define level data as 2D constant array (e.g., 20x15 tiles)
- [ ] Implement setup_tilemap system to spawn tiles from array
- [ ] Configure camera position/scale to view level
- [ ] Test: Run and verify tilemap renders correctly
- [ ] Test: Verify different tile types are visually distinct

### Notes
- bevy_ecs_tilemap version must be compatible with Bevy 0.18 (likely v0.15.x)
- Tile coordinates: (0,0) is typically bottom-left in bevy_ecs_tilemap
- For placeholder graphics, can use simple colored squares or load a tileset image
- Level array format example: `[[0, 1, 1, 1, 0], [0, 0, 0, 0, 0]]` where 0=empty, 1=solid
- Consider tile types: 0=Empty/Air, 1=Solid Platform
- Camera may need adjustment: use orthographic projection for 2D
