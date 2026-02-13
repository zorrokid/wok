# Bevy 0.18 Metroidvania Game Development Instructions

## Project Overview
This is a metroidvania-style platformer game built with Bevy 0.18 game engine in Rust.

## Bevy 0.18 Key Concepts

### Entity Component System (ECS)
- **Entities**: Unique identifiers for game objects
- **Components**: Data attached to entities (structs with `#[derive(Component)]`)
- **Systems**: Functions that operate on entities with specific components
- **Resources**: Global shared state (structs with `#[derive(Resource)]`)

### App Structure
```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (system1, system2))
        .run();
}
```

### System Scheduling
- Use `Startup` for initialization systems
- Use `Update` for per-frame logic
- Use `FixedUpdate` for physics/fixed timestep logic
- Chain systems with `.chain()` or use system sets for ordering

### Queries
```rust
fn system(query: Query<(&Transform, &mut Velocity), With<Player>>) {
    for (transform, mut velocity) in &mut query {
        // Process entities
    }
}
```

### Commands
Use `Commands` to spawn/despawn entities or modify components:
```rust
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::default(),
        Sprite { ... },
    ));
}
```

### Events
```rust
#[derive(Event)]
struct CollisionEvent;

fn send_events(mut events: EventWriter<CollisionEvent>) {
    events.send(CollisionEvent);
}

fn handle_events(mut events: EventReader<CollisionEvent>) {
    for event in events.read() {
        // Handle event
    }
}
```

## Metroidvania-Specific Patterns

### Player Movement
- Use `Transform` for position
- Implement velocity and acceleration components
- Handle keyboard input with `Res<ButtonInput<KeyCode>>`
- Apply gravity and collision detection

### Camera
- Use `Camera2d` for 2D games
- Implement camera follow system targeting player
- Consider camera bounds for rooms/areas

### Collision Detection
- Use Bevy's built-in rapier2d plugin OR
- Implement AABB collision with custom components
- Handle platforms, walls, enemies, collectibles

### Level Design
- Load tilemaps with bevy_ecs_tilemap crate
- Use scene files or procedural generation
- Implement room-based structure

### Abilities & Upgrades
- Create ability components (DoubleJump, WallJump, etc.)
- Use marker components for gating areas
- Track collected upgrades in a resource

### Game State
```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

// Add to app
.init_state::<GameState>()
.add_systems(OnEnter(GameState::Playing), setup_game)
```

## Best Practices

1. **Component Design**: Keep components small and focused
2. **System Organization**: Group related systems in plugins
3. **Performance**: Use `Changed<T>` filters to avoid unnecessary work
4. **Asset Loading**: Preload assets in startup or loading state
5. **Debug Tools**: Use `bevy_inspector_egui` for runtime inspection
6. **Testing**: Write unit tests for game logic separate from Bevy systems

## Common Dependencies for Metroidvania
```toml
bevy = "0.18.0"
bevy_ecs_tilemap = "0.15"  # For tilemap rendering
bevy_rapier2d = "0.28"     # For physics (optional)
bevy_inspector_egui = "0.28"  # For debugging
```

## Code Style
- Use descriptive component names (e.g., `Player`, `Velocity`, `Health`)
- Prefix marker components with `Is` or use noun form
- Keep systems focused on single responsibility
- Document complex game mechanics with comments

## Code Organization and Quality

### Abstraction and Clean Code
- **Seek abstractions**: When adding new functionality, actively look for opportunities to make code more understandable through abstraction
- **Use established patterns**: Apply well-known design patterns (Strategy, Builder, State, etc.) when appropriate
- **Proactive suggestions**: Don't wait to be asked - suggest abstractions and patterns that would improve code quality
- **DRY (Don't Repeat Yourself)**: Always eliminate code duplication
  - Extract repeated logic into functions
  - Create shared types/structs for common data patterns
  - Use generics when the same logic applies to multiple types

### Function Size and Decomposition
- **50-line guideline**: When a single function grows over 50 lines, split it into smaller functions
- **Single Responsibility**: Each function should do one thing well
- **Meaningful names**: Extracted functions should have clear, descriptive names that explain their purpose
- **Logical grouping**: Group related functions in modules or impl blocks

### Pure Functions and Testing
- **Pure functions when appropriate**: 
  - Functions without side effects are easier to test and reason about
  - Extract business logic from Bevy systems into pure functions
  - Keep I/O and state changes at system boundaries
- **Test pure functions**: Write unit tests for:
  - Complex calculations and algorithms
  - Game rules and conditions
  - State machines and logic
  - Any function with conditional logic
- **Test coverage**: Aim for comprehensive tests on pure functions, minimal tests on Bevy systems

### Example: Good vs Bad
```rust
// ❌ BAD: Large function, duplicate logic, hard to test
pub fn player_movement(/* ... */) {
    // 80 lines of mixed concerns...
    let left_foot_x = transform.translation.x - (SPRITE_WIDTH / 2.0 - 3.0);
    let right_foot_x = transform.translation.x + (SPRITE_WIDTH / 2.0 - 3.0);
    let (left_tile_x, left_tile_y) = world_to_tile_coords(left_foot_x, check_y);
    // ... same logic repeated later ...
    let left_foot_x = transform.translation.x - (SPRITE_WIDTH / 2.0 - 3.0);
    let right_foot_x = transform.translation.x + (SPRITE_WIDTH / 2.0 - 3.0);
    // ... untestable Bevy-specific code mixed with game logic ...
}

// ✅ GOOD: Abstraction, DRY, testable
struct FeetPositions {
    left_x: f32,
    right_x: f32,
}

impl FeetPositions {
    fn from_center(center_x: f32) -> Self {
        Self {
            left_x: center_x - (SPRITE_WIDTH / 2.0 - 3.0),
            right_x: center_x + (SPRITE_WIDTH / 2.0 - 3.0),
        }
    }
}

// Pure, testable function
fn is_on_ground(feet: &FeetPositions, y: f32) -> bool {
    let check_y = y - 1.0;
    check_foot_collision(feet.left_x, check_y) 
        || check_foot_collision(feet.right_x, check_y)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_feet_positions() {
        let feet = FeetPositions::from_center(10.0);
        assert_eq!(feet.left_x, 3.0);
        assert_eq!(feet.right_x, 17.0);
    }
}

// System uses abstractions
pub fn player_movement(/* ... */) {
    let feet = FeetPositions::from_center(transform.translation.x);
    let is_grounded = is_on_ground(&feet, transform.translation.y);
    // Clean, readable, testable
}
```

## Advanced Patterns from Recent Refactoring

### Helper Struct Pattern for Reducing Duplication
When multiple functions compute the same data from the same inputs, extract into a helper struct:

```rust
// ❌ BAD: Duplicated computation in multiple functions
fn is_grounded(player_coord: &PlayerCoord, ...) -> bool {
    let check_y = player_coord.ground_check_y();
    let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord.feet_x_left, check_y);
    let (right_tile_x, right_tile_y) = world_to_tile_coords(player_coord.feet_x_right, check_y);
    let left_tile = get_tile_type_at(left_tile_x, left_tile_y);
    let right_tile = get_tile_type_at(right_tile_x, right_tile_y);
    // ... use tiles
}

fn ground_snap_y(player_coord: &PlayerCoord, ...) -> Option<f32> {
    // Same 5 lines repeated!
    let check_y = player_coord.ground_check_y();
    let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord.feet_x_left, check_y);
    // ...
}

// ✅ GOOD: Extract to helper struct and function
struct FeetTiles {
    left: Option<TileType>,
    right: Option<TileType>,
    left_tile_y: i32,
    right_tile_y: i32,
}

fn check_feet_tiles(
    player_coord: &PlayerCoord,
    world_to_tile_coords: impl Fn(f32, f32) -> (i32, i32),
    get_tile_type_at: impl Fn(i32, i32) -> Option<TileType>,
) -> FeetTiles {
    let check_y = player_coord.ground_check_y();
    let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord.feet_x_left, check_y);
    let (right_tile_x, right_tile_y) = world_to_tile_coords(player_coord.feet_x_right, check_y);
    FeetTiles {
        left: get_tile_type_at(left_tile_x, left_tile_y),
        right: get_tile_type_at(right_tile_x, right_tile_y),
        left_tile_y,
        right_tile_y,
    }
}

// Both functions now use the helper
fn is_grounded(player_coord: &PlayerCoord, ...) -> bool {
    let feet_tiles = check_feet_tiles(player_coord, world_to_tile_coords, get_tile_type_at);
    feet_tiles.left.map(is_solid_tile).unwrap_or(false)
        || feet_tiles.right.map(is_solid_tile).unwrap_or(false)
}
```

**Benefits:**
- Eliminates duplication (DRY)
- Single source of truth for the computation
- Easier to maintain and modify
- Naturally documents the relationship between functions

### Method Encapsulation for Constants
Don't just add constants—add methods that encapsulate calculations using those constants:

```rust
// ❌ BAD: Constant visible but calculation scattered
const GROUND_CHECK_OFFSET: f32 = 1.0;

// Multiple places do this:
let check_y = player_coord.feet_y - GROUND_CHECK_OFFSET;

// ✅ GOOD: Method encapsulates the calculation
const GROUND_CHECK_OFFSET: f32 = 1.0;

impl PlayerCoord {
    /// Get the Y position to check for ground (slightly below feet)
    pub fn ground_check_y(&self) -> f32 {
        self.feet_y - GROUND_CHECK_OFFSET
    }
}

// Usage is cleaner and self-documenting
let check_y = player_coord.ground_check_y();
```

**Benefits:**
- Self-documenting API
- Encapsulation allows changing implementation without affecting callers
- Easier to test (method can be unit tested)
- Prevents calculation mistakes

### Phased Refactoring with Testing Gates
Break large refactorings into phases, each with a clear testing gate:

```
Phase 1: Extract types → Test: compilation + gameplay
Phase 2: Add unit tests → Test: test suite passes
Phase 3: Reduce duplication → Test: gameplay unchanged
Phase 4: Add constants → Test: build succeeds
Phase 5: Add documentation → Test: cargo doc + final gameplay
```

**Benefits:**
- Easy rollback if a phase breaks something
- Incremental, visible progress
- Each phase adds value independently
- Testing gates catch regressions early
- Can stop at any phase if needed

**Guidelines:**
- Each phase should compile and run
- Test before moving to next phase
- Document phase completion in spec
- Keep phases small (< 1 hour each)

### Comprehensive Testing for Coordinate Math
Write thorough tests for coordinate/position calculations—these are bug-prone:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_specific_values() {
        // ✅ Use real values, not just 0 or 1
        let player_coord = PlayerCoord::new(Coord::new(100.0, 200.0));
        assert_eq!(player_coord.feet_y, 192.0); // 200.0 - 8.0
        assert_eq!(player_coord.feet_x_left, 95.0); // 100.0 - 5.0
        assert_eq!(player_coord.feet_x_right, 105.0); // 100.0 + 5.0
    }

    #[test]
    fn test_edge_cases() {
        // ✅ Test edge cases like origin, negative values
        let at_origin = PlayerCoord::new(Coord::new(0.0, 0.0));
        assert_eq!(at_origin.feet_y, -8.0);
    }

    #[test]
    fn test_consistency() {
        // ✅ Verify constants are applied correctly
        let player_coord = PlayerCoord::new(Coord::new(50.0, 100.0));
        let check_y = player_coord.ground_check_y();
        let expected = player_coord.feet_y - 1.0; // GROUND_CHECK_OFFSET
        assert_eq!(check_y, expected);
    }

    #[test]
    fn test_relationships() {
        // ✅ Test relationships between values
        let player_coord = PlayerCoord::new(Coord::new(50.0, 100.0));
        let feet_width = player_coord.feet_x_right - player_coord.feet_x_left;
        assert_eq!(feet_width, 10.0); // 2 * (SPRITE_WIDTH/2 - INSET)
    }
}
```

**Test categories to include:**
- Specific numeric values (not just trivial cases)
- Edge cases (zero, negative, boundaries)
- Consistency checks (verify constants work as expected)
- Relationship tests (verify derived values relate correctly)

### Feature-Based Module Structure
When a module grows beyond a single file, organize by **feature/concern** rather than by type:

```rust
// ✅ GOOD: Feature-based organization
src/
  player/
    mod.rs          // Public API, components, spawn
    movement.rs     // Movement logic (pure functions)
    combat.rs       // Combat logic (future)
  level/
    mod.rs          // Level setup and data
    tile.rs         // Tile-related types and utilities
  
// ❌ BAD: Type-based organization
src/
  systems.rs        // All systems together
  components.rs     // All components together
  helpers.rs        // All helper functions together
```

**Benefits:**
- Related code stays together
- Clear boundaries between features
- Easier to find and modify specific functionality
- Natural place for feature-specific tests

### Coordinate Abstractions
Create semantic types for positions and coordinates instead of using raw floats:

```rust
// ✅ GOOD: Clear intent, type safety
struct PlayerCoord {
    center: Coord,
    feet_y: f32,
    feet_x_left: f32,
    feet_x_right: f32,
}

impl From<Transform> for PlayerCoord {
    fn from(transform: Transform) -> Self {
        PlayerCoord::new(transform.into())
    }
}

// Usage
let player_coord: PlayerCoord = (*transform).into();
let is_grounded = check_ground(&player_coord);

// ❌ BAD: Magic number calculations scattered everywhere
let feet_y = transform.translation.y - 8.0;
let left_x = transform.translation.x - 5.0;
```

**Benefits:**
- Self-documenting code
- Centralized calculation logic
- Type safety prevents mixing up coordinates
- Easy to extend with more derived positions

### Dependency Injection for Testability
Use function parameters to inject dependencies, making functions testable without Bevy:

```rust
// ✅ GOOD: Pure function with injected dependencies
fn is_grounded(
    player_coord: &PlayerCoord,
    world_to_tile_coords: impl Fn(f32, f32) -> (i32, i32),
    get_tile_type_at: impl Fn(i32, i32) -> Option<TileType>,
    is_solid_tile: impl Fn(TileType) -> bool,
) -> bool {
    // Logic here can be tested with mock functions
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_grounded_with_mocks() {
        let player_coord = PlayerCoord::new(/* ... */);
        let mock_world_to_tile = |x, y| (0, 0);
        let mock_get_tile = |x, y| Some(TileType::Solid);
        let mock_is_solid = |t| t == TileType::Solid;
        
        assert!(is_grounded(&player_coord, mock_world_to_tile, mock_get_tile, mock_is_solid));
    }
}

// ❌ BAD: Hard-coded global dependencies
fn is_grounded(player_coord: &PlayerCoord) -> bool {
    // Directly calls world_to_tile_coords() - hard to test
}
```

### Extract Domain Logic to Pure Functions
Separate complex logic from Bevy systems by extracting into pure functions:

```rust
// ✅ GOOD: System delegates to pure functions
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    // System extracts data
    let player_coord: PlayerCoord = (*transform).into();
    
    // Pure function calls with injected dependencies
    let is_grounded = is_grounded(&player_coord, world_to_tile_coords, get_tile_type_at, is_solid_tile);
    let target_velocity_x = get_target_velocity_x(is_left, is_right);
    
    velocity.0.x = apply_horizontal_acceleration(
        velocity.0.x, target_velocity_x, delta,
        PLAYER_ACCELERATION, PLAYER_DECELERATION, PLAYER_MAX_SPEED
    );
    
    velocity.0.y = get_velocity_y(is_grounded, velocity.0.y, delta, JUMP_VELOCITY, is_jump);
    
    // Apply results back
    transform.translation.x += velocity.0.x * delta;
    transform.translation.y += velocity.0.y * delta;
    
    if let Some(snap_y) = ground_snap_y(&player_coord_after, /* deps */) {
        transform.translation.y = snap_y;
    }
}

// Each extracted function is testable
fn apply_horizontal_acceleration(/* ... */) -> f32 { /* ... */ }
fn get_velocity_y(/* ... */) -> f32 { /* ... */ }
fn ground_snap_y(/* ... */) -> Option<f32> { /* ... */ }
```

**Pattern:**
1. System handles Bevy-specific concerns (queries, input, time)
2. System extracts data into simple types (PlayerCoord)
3. System calls pure functions with data
4. System applies results back to components

### Type Safety with Enums
Replace magic numbers with type-safe enums:

```rust
// ✅ GOOD: Type-safe enum
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TileType {
    Solid = 1,
    Empty = 0,
}

impl From<u32> for TileType {
    fn from(value: u32) -> Self {
        match value {
            1 => TileType::Solid,
            _ => TileType::Empty,
        }
    }
}

pub fn is_solid_tile(tile_type: TileType) -> bool {
    tile_type == TileType::Solid
}

// ❌ BAD: Magic numbers
fn is_solid_tile(tile_type: u32) -> bool {
    tile_type == 1  // What does 1 mean?
}
```

**Benefits:**
- Self-documenting (TileType::Solid vs 1)
- Compile-time safety (can't pass wrong type)
- Easy to extend (add TileType::Ladder, etc.)
- Pattern matching support

## Testing and Architecture

### Hybrid Approach: Core Logic Separation
**Philosophy:** Separate complex logic into pure functions for testability, while keeping systems in Bevy ECS for integration.

**What to separate (extract to pure functions):**
- Complex calculations (physics formulas, jump height, damage calculation)
- Game rules and conditions (can player jump? is grounded? collision detection algorithms)
- State machines and logic (player state transitions, AI behavior)
- Algorithms (pathfinding, procedural generation)

**What to keep in Bevy systems:**
- Component queries and iteration
- Input handling (reading ButtonInput)
- Rendering logic
- Simple wiring between components
- System scheduling and ordering

### Example Pattern
```rust
// Pure function - easy to unit test
pub fn should_jump(is_grounded: bool, has_double_jump: bool, jump_count: u8) -> bool {
    is_grounded || (has_double_jump && jump_count < 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_should_jump_when_grounded() {
        assert!(should_jump(true, false, 0));
    }
    
    #[test]
    fn test_cannot_double_jump_without_ability() {
        assert!(!should_jump(false, false, 1));
    }
}

// System uses pure function
pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Transform, &mut Velocity, &JumpState), With<Player>>,
) {
    for (transform, mut velocity, jump_state) in &mut query {
        let is_grounded = check_ground(transform.translation.y);
        
        if keyboard.just_pressed(KeyCode::Z) 
            && should_jump(is_grounded, jump_state.has_double_jump, jump_state.count) {
            velocity.0.y = JUMP_VELOCITY;
        }
    }
}
```

### Testing Guidelines
1. **Write unit tests for pure functions** - Fast, isolated, test game logic
2. **Use Bevy integration tests sparingly** - For system interactions when needed
3. **Don't over-architect** - If logic is simple (< 5 lines), keep it in the system
4. **Test when sensible:**
   - Complex calculations definitely need tests
   - Simple getters/setters usually don't
   - Game rules and conditions should be tested
   - Collision algorithms should be tested

### Module Organization
```rust
// src/physics.rs - Pure functions
pub fn calculate_jump_velocity(gravity: f32, height: f32) -> f32 { /* ... */ }
pub fn apply_acceleration(velocity: f32, accel: f32, delta: f32) -> f32 { /* ... */ }

// src/player.rs - Bevy systems
pub fn player_movement(/* Bevy queries */) {
    // Use pure functions from physics module
    let new_velocity = apply_acceleration(velocity, ACCEL, delta);
}
```

### When NOT to Separate
- Simple component queries
- Direct input-to-action mappings
- Straightforward rendering logic
- One-liner calculations
- Code that's already clear and simple

## Spec-Driven Development

### Project Structure
```
/docs
  /specs              # Feature specs with integrated plans and tasks
    001-player-movement.md
    002-combat-system.md
    003-abilities.md
```

### Naming Convention
- Specs are numbered with 3-digit prefix: `001-`, `002-`, etc.
- Use descriptive kebab-case names after the prefix
- Example: `001-basic-game-window.md`

### Spec File Structure
Each spec file is self-contained with:
1. **Specification** - What to build (requirements, behavior, acceptance criteria)
2. **Plan** - How to build it (approach, technical decisions)
3. **Tasks** - Implementation checklist (concrete steps)

**Important:** Specs should NOT include complete code implementations. Focus on requirements, approach, and tasks. Code snippets are acceptable for clarification (e.g., function signatures, API usage examples), but avoid full implementations.

### Spec Completion Guidelines
**A spec is NOT complete until all tasks are checked off.**

When implementing a spec:
1. **Implement in steps** - Break implementation into logical parts, make changes incrementally
2. **Explain each step** - Briefly describe what each code change does and why
3. **Check off tasks progressively** - Mark implementation tasks complete as you go
4. **Wait for user testing confirmation** - Never mark testing tasks complete without user confirmation
5. **Verify implementation matches plan** - After implementation, confirm the actual approach matches the spec's plan
   - If implementation differs from plan (bug fixes, alternative approaches, additional steps):
     - Update the spec to document what was actually done
     - Add notes explaining why implementation differs
     - Get user confirmation on spec updates before marking complete
6. If a task cannot be completed:
   - Update the spec to explain why (add note)
   - Remove or modify the task to reflect reality
   - Never leave incomplete tasks unchecked in a "finished" spec
7. Before moving to the next spec, verify all checkboxes are marked

**Implementation approach:**
- **NOT acceptable**: One massive code diff with all changes at once
- **Acceptable**: Multiple smaller changes with explanations:
  1. "Adding helper function for coordinate conversion..."
  2. "Now modifying player_movement to use tile detection..."
  3. "Removing the old GROUND_LEVEL constant..."

**Spec accuracy:**
- Specs should reflect reality, not just initial plans
- If bugs were found and fixed, document them in the spec
- If approach changed during implementation, update the spec
- Future developers should understand what actually works, not just what was planned

Example of handling impossible tasks:
```markdown
- [x] Calculate screen boundaries - removed in spec 005 (camera follow replaced this)
- [x] Add feature X - not needed, alternative approach used (see notes)
```

### Spec Template
```markdown
# [Feature Name]

## Overview
Brief description of the feature from player/game perspective.

## Requirements
- Player can do X
- System behaves Y when Z

## Acceptance Criteria
- [ ] Criteria 1
- [ ] Criteria 2

---

## Implementation Plan

### Approach
How we'll implement this technically.

### Components & Systems
- `ComponentName` - Description
- `system_name()` - What it does

### Tasks
- [ ] Create component structs
- [ ] Implement input system
- [ ] Add physics/collision
- [ ] Test and validate

### Notes
Technical considerations, edge cases, dependencies.
Can include small code snippets for clarity (e.g., function signatures, API calls), but NOT complete implementations.
```

## Local Bevy Source Code

All Bevy 0.18 source code is available in the `vendor/` directory via `cargo vendor`.

### Searching Bevy Source
Use grep to search through Bevy's implementation:
```bash
# Find specific types or functions
grep -r "struct Transform" vendor/bevy*/

# Search in specific Bevy modules
grep -r "Query" vendor/bevy_ecs/

# Find examples of patterns
grep -r "impl Plugin" vendor/bevy*/
```

### Key Bevy Crates in Vendor
- `vendor/bevy/` - Main Bevy crate re-exports
- `vendor/bevy_ecs/` - Entity Component System
- `vendor/bevy_app/` - App and plugin system
- `vendor/bevy_transform/` - Transform and hierarchy
- `vendor/bevy_render/` - Rendering system
- `vendor/bevy_sprite/` - 2D sprites
- `vendor/bevy_input/` - Input handling
- `vendor/bevy_time/` - Time and timer utilities
- `vendor/bevy_asset/` - Asset loading

### Using Vendor Code as Reference
When implementing features, search vendor for examples:
```bash
# How does Bevy implement camera follow?
grep -r "camera" vendor/bevy_render/

# Find collision examples
grep -r "collision" vendor/bevy*/
```

## Resources
- Official Bevy Book: https://bevyengine.org/learn/book/
- Bevy Examples: https://bevyengine.org/examples/
- API Docs: https://docs.rs/bevy/0.18.0/bevy/
- Local Source: Search `vendor/` directory for Bevy implementation details

## Development Workflow
- Use `cargo run` for regular development
- Use `cargo build` when you just need to check compilation
- Avoid release builds during development

## Build Guidelines

### Build Type Selection
- **Always use dev builds (`cargo build` or `cargo run`)** for development and testing
- **Never use release builds (`cargo build --release`)** during development - they are extremely slow
- Release builds are only for final distribution, not for iterative development

### Build Optimization
- **Avoid `cargo clean` unless absolutely necessary** - Bevy builds are very slow (2-3 minutes)
- Only use `cargo clean` when:
  - Compiler errors seem inconsistent or cached
  - Switching between major dependency versions
  - Build artifacts are genuinely corrupted
- For most compilation errors, regular `cargo build` is sufficient
- Incremental builds are much faster (~5-30 seconds vs 2-3 minutes)

**Note**: Advanced optimizations like LLD linker and dynamic linking can speed up incremental builds, but require full rebuilds to set up. For this project, stick with regular dev builds.
