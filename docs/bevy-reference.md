# Bevy 0.18 — Reference Index

Technical reference for the engine stack used in this project. Split into three focused documents — read only what is relevant to the task at hand.

---

## [bevy-ecs.md](bevy-ecs.md) — ECS Core

The foundational "how Bevy works" reference. Covers ECS architecture (entities, components, systems, resources, events, plugins), app structure, system scheduling and ordering, queries, commands, assets, and input.

**Read when:** designing new components or systems, reasoning about system ordering, working on any Bevy feature, or onboarding to the codebase.

---

## [bevy-tiled.md](bevy-tiled.md) — Tilemap and Coordinates

Covers Bevy's 2D coordinate system (Y-up, world origin, Transform), tilemap coordinates (tile grid, TilemapAnchor, world↔tile conversion), and the `bevy_ecs_tiled` plugin (map loading, lifecycle events, physics collider generation, TMX format).

**Read when:** working on level loading, tile layout, camera positioning, player spawn position, coordinate conversions, or the `bevy_ecs_tiled` integration.

---

## [avian2d.md](avian2d.md) — Physics

Covers the Avian2D physics engine: rigid bodies, colliders, velocity, constraints, gravity, player entity setup, ground detection with `ShapeCaster`, the movement pattern (write velocity, never touch Transform), and collision layers.

**Read when:** working on player movement, collision, jumping, ground detection, moving platforms, or any feature involving physical simulation.

---

## Vendor Source

When a document above lacks enough detail — exact function signatures, available component fields, internal behaviour — read the source directly:

- `vendor/avian2d/` — Physics engine
- `vendor/bevy_ecs_tiled/` — Tilemap plugin
- `vendor/bevy_ecs_tilemap/` — Underlying tilemap rendering
- `vendor/bevy_*/` — Core Bevy crates
