# Player Module Refactoring

## Overview
Refactor the player module to improve code organization, reduce duplication, enhance testability, and make the code more maintainable. This builds on the existing movement system without changing gameplay behavior.

## Requirements
- Maintain all existing player movement behavior (no gameplay changes)
- Reduce code duplication in collision checking logic
- Improve organization of coordinate types and constants
- Enhance testability of pure functions
- Make code more readable and maintainable

## Acceptance Criteria
- [ ] Player movement behavior is unchanged (jumping, walking, collision work identically)
- [ ] Code duplication in collision checks is eliminated
- [ ] PlayerCoord has its own module with tests
- [ ] Collision detection logic is extracted and reusable
- [ ] Constants are better organized
- [ ] All tests pass

---

## Implementation Plan

### Approach
Extract common patterns into reusable abstractions:
1. Move `PlayerCoord` and `Coord` to dedicated `coord.rs` module
2. Extract duplicated tile collision checking logic into helper functions
3. Create collision detection helpers to reduce code duplication
4. Add unit tests for pure functions
5. Improve constant organization

### Current Issues
1. **Duplication**: `is_grounded()` and `ground_snap_y()` both check tiles beneath feet with nearly identical code
2. **Module organization**: `PlayerCoord` is in `mod.rs` but used primarily by `movement.rs`
3. **Magic numbers**: Constants like `1.0` (ground check offset) and `3.0` (foot edge inset) lack named constants
4. **No tests**: Pure functions lack unit tests

### Components & Files

**New Files:**
- `src/player/coord.rs` - Coordinate types (Coord, PlayerCoord)
- `src/player/collision.rs` - Collision detection helpers (optional, if enough logic to extract)

**Modified Files:**
- `src/player/mod.rs` - Re-export coord types, cleaner constant organization
- `src/player/movement.rs` - Use new helpers, reduced duplication

### Tasks

#### Phase 1: Extract Coordinate Types
- [x] Create `src/player/coord.rs` module
- [x] Move `Coord` struct to coord.rs
- [x] Move `PlayerCoord` struct to coord.rs
- [x] Add `FOOT_EDGE_INSET` constant (currently hardcoded as 3.0)
- [x] Add `ground_check_y()` method to PlayerCoord (encapsulate the -1.0 offset)
- [x] Update mod.rs to re-export coord types
- [x] Update movement.rs imports
- [x] Test: Verify compilation and gameplay unchanged

**Status: Complete** ✓
- Created `src/player/coord.rs` with Coord and PlayerCoord types
- Added `FOOT_EDGE_INSET` (3.0) and `GROUND_CHECK_OFFSET` (1.0) constants
- Implemented `ground_check_y()` method to encapsulate offset calculation
- Updated `mod.rs` to re-export types via `pub use coord::{Coord, PlayerCoord}`
- Updated `is_grounded()` and `ground_snap_y()` to use new method
- Build successful with expected warnings only
- Gameplay behavior unchanged (pure refactoring)

#### Phase 2: Add Tests for Coordinate Types
- [ ] Add test for `PlayerCoord::new()` feet position calculations
- [ ] Add test for `ground_check_y()` method
- [ ] Run tests with `cargo test`

#### Phase 3: Extract Common Collision Patterns
- [ ] Identify shared logic between `is_grounded()` and `ground_snap_y()`
- [ ] Create helper function `check_feet_tiles()` that returns tile info for both feet
- [ ] Refactor `is_grounded()` to use helper
- [ ] Refactor `ground_snap_y()` to use helper
- [ ] Test: Verify gameplay unchanged

#### Phase 4: Add Constants for Magic Numbers
- [ ] Add `GROUND_CHECK_OFFSET` constant (currently 1.0)
- [ ] Replace hardcoded 1.0 with constant in collision checks
- [ ] Document what each constant represents

#### Phase 5: Cleanup and Documentation
- [ ] Add doc comments to public functions in coord.rs
- [ ] Add doc comments to collision helper functions
- [ ] Verify all constants have explanatory comments
- [ ] Final gameplay test: jumping, walking, platform edges

### Notes

**Key Principle: No Behavior Changes**
- This is pure refactoring - gameplay must remain identical
- Test frequently to ensure no regressions
- If any behavior changes, roll back and investigate

**Testing Strategy**
- Unit tests for coordinate calculations
- Manual gameplay testing after each phase
- Compare behavior before/after refactoring

**Duplication Example**
Both functions have this pattern:
```rust
let check_y = player_coord.feet_y - 1.0;
let (left_tile_x, left_tile_y) = world_to_tile_coords(player_coord.feet_x_left, check_y);
let (right_tile_x, right_tile_y) = world_to_tile_coords(player_coord.feet_x_right, check_y);
let left_tile = get_tile_type_at(left_tile_x, left_tile_y);
let right_tile = get_tile_type_at(right_tile_x, right_tile_y);
```

Could be extracted to:
```rust
struct FeetTiles {
    left: Option<TileType>,
    right: Option<TileType>,
    left_tile_y: i32,
    right_tile_y: i32,
}

fn check_feet_tiles(player_coord: &PlayerCoord, ...) -> FeetTiles
```

**Coordinate Module Benefits**
- Encapsulates magic numbers (3.0 for foot inset, 1.0 for ground check)
- Makes PlayerCoord easily testable
- Provides single source of truth for coordinate calculations
- Can add more coordinate helpers in future (head position, center, etc.)

**Optional Future Enhancements** (not in this spec)
- Create `TileCollisionContext` trait to eliminate repetitive `impl Fn` parameters
- Extract physics constants to config struct
- Add more comprehensive collision tests

## Implementation Order
1. Start with Phase 1 (extract coordinates) - low risk, high value
2. Add tests (Phase 2) - ensures coordinate calculations are correct
3. Reduce duplication (Phase 3) - more complex, but coordinates are now tested
4. Polish with constants and docs (Phases 4-5)

## Success Criteria
- All checkboxes marked complete
- `cargo test` passes
- Manual gameplay test shows identical behavior
- Code is more readable and maintainable
- Less duplication in collision logic
