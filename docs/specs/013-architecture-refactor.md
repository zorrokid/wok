# Spec 013 — Code Architecture Refactor

## Status: Complete

## Overview

After completing specs 011 and 012, an architectural review identified 8 issues ranging from
a silent correctness risk (duplicated spawn position) to structural concerns that will become
maintenance burdens as the game grows. This spec addresses all of them before new features
are added.

## Requirements

1. Spawn position must be defined once and shared between `spawn.rs` and `kill_zone`.
2. `movement.rs` must not import anything from `crate::level`.
3. Each feature module must own its Bevy system registrations via a `Plugin` impl.
4. Window size must be defined once and shared between `main.rs` and `camera.rs`.
5. `Camera2d` must be spawned in the camera module, not the level module.
6. `kill_zone` must live in `spawn.rs`, not `movement.rs`.
7. The `Update`/`FixedUpdate` scheduling trade-off must be documented with a known fix.
8. The ShapeCaster radius shrink must be a named constant.

## Acceptance Criteria

- [ ] `cargo build` — clean compile, zero warnings
- [ ] `cargo test` — all existing tests pass
- [ ] `movement.rs` has zero imports from `crate::level`
- [ ] No system is registered in more than one place
- [ ] Game behaviour unchanged: movement, camera, level bounds, kill zone all work

## Task Checklist

- [x] Write spec file
- [x] Create branch `013-architecture-refactor`
- [x] Add `PLAYER_SPAWN_X`, `PLAYER_SPAWN_Y`, `KILL_ZONE_Y`, `SHAPE_CASTER_RADIUS` to `player/mod.rs`
- [x] Add `PlayerPlugin` to `player/mod.rs`
- [x] Update `spawn.rs` to use shared constants and receive `kill_zone`
- [x] Update `movement.rs` to use shared constants, drop `level::tile` import, remove `kill_zone`
- [x] Add `LevelPlugin` to `level/mod.rs`, remove `Camera2d` spawn
- [x] Add `setup_camera`, `CameraPlugin`, import `WINDOW_WIDTH` in `camera.rs`
- [x] Add `WINDOW_WIDTH`/`WINDOW_HEIGHT` to `main.rs`, switch to plugin registration
- [x] `cargo build` passes with no warnings
- [x] `cargo test` passes
- [x] User verifies game behaviour in-game
