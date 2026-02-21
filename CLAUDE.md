# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WoK is a Metroidvania-style 2D platformer built with **Bevy 0.18** (ECS game engine) in Rust. The project uses a spec-driven development methodology documented in `docs/specs/`.

## Build Commands

```bash
cargo run          # Run the game (dev build)
cargo check        # Verify compilation without producing a binary (preferred)
cargo build        # Compile only (dev build) — heavier than cargo check
cargo test         # Run all tests
cargo test <name>  # Run a specific test by name
```

**Prefer `cargo check` over `cargo build`** when only verifying that code compiles — it skips code generation and is significantly lighter on CPU and RAM. Only use `cargo build` when a runnable binary is actually needed.

**Never use `cargo build --release`** during development — it's extremely slow. **Avoid `cargo clean`** unless absolutely necessary; Bevy's initial build takes 2–3 minutes but incremental builds are fast.

## Architecture

### ECS Structure

`src/main.rs` is the entry point. It wires up all plugins and systems:
- **Startup systems**: `setup_tilemap_new` (level), `spawn_player`
- **Update systems**: `player_movement`, `camera_follow`, `exit_on_esc`

### Module Organization

Code is organized by **feature**, not by type:

- `src/player/` — Player components/constants (`mod.rs`), spawning (`spawn.rs`), movement physics (`movement.rs`), coordinate abstractions (`coord.rs`)
- `src/level/` — Tilemap setup (`mod.rs`), tile utilities (`tile.rs`)
- `src/camera.rs` — Smooth camera follow (lerp)
- `src/keyboard.rs` — ESC to exit

### Key Patterns

**Coordinate abstractions**: The `PlayerCoord` struct in `src/player/coord.rs` wraps raw float positions into semantic types (`feet_y`, `feet_x_left`, `feet_x_right`). Use/extend this instead of scattering raw offset calculations.

**Pure functions for testability**: Complex game logic (collision, physics math) is extracted into pure functions that can be unit tested without Bevy. Systems act as thin wrappers that read input, call pure functions, and apply results back to components.

**Helper struct pattern**: When multiple functions compute the same derived data, extract to a shared struct (e.g., `FeetTiles` computed once and used by both `is_grounded()` and `ground_snap_y()`).

### Current State (Spec 011 in progress)

The project is migrating from a custom tilemap/physics system to:
- **Tiled editor** via `bevy_ecs_tiled` for level loading (`assets/map.tmx`)
- **Avian2D** for physics-based collision (replacing custom tile collision code)

The old custom system (`FeetTiles`, `check_feet_tiles`, manual `Velocity` component, `world_to_tile_coords`) is being removed. The new system uses `RigidBody`, `Collider`, `ShapeCaster`, and `LinearVelocity` from Avian2D.

### Assets

- `assets/map.tmx` — Level map (Tiled format)
- `assets/tileset.tsx` / `assets/tileset.png` — Tile definitions
- `assets/player.png` — 16×16 player sprite

### Bevy Source Code

All Bevy 0.18 source is vendored in `vendor/`. Search it for implementation details:
```bash
grep -r "struct Transform" vendor/bevy_transform/
grep -r "LinearVelocity" vendor/avian2d/
```

## Spec-Driven Development

Features are defined in numbered spec files (`docs/specs/NNN-name.md`). Each spec contains: Overview, Requirements, Acceptance Criteria, Implementation Plan, and a task checklist. A spec is not complete until all tasks are checked off. Specs are implemented incrementally (not as one large diff), and updated to reflect what was actually built if the approach changes.

### Workflow

1. **Write the spec** — draft the spec file and present it to the user for review.
2. **Wait for user approval** — do not begin any implementation until the user explicitly confirms the spec.
3. **Create a branch** — when implementation starts, create a git branch named `NNN-short-name` matching the spec number and slug (e.g. `012-level-bounds`).
4. **Implement** — work through the spec's task checklist on that branch.
5. **User verifies** — the user tests the feature in-game and confirms it works.
6. **Mark spec complete** — set `Status: Complete` and check off all remaining tasks in the spec file.
7. **Open a pull request** — create a PR from the feature branch into `main`.
