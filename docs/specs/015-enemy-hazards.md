# Spec 015 — Enemy Hazards & Player Health

## Status: Pending

## Overview

Introduces stationary enemy hazards and a player health system. Enemies are solid red squares placed at fixed positions during level startup. Walking into one deals 1 HP of damage; a brief invincibility window prevents rapid drain. When HP reaches zero the player respawns at the start position with full health. The kill zone fall respawn is unified with the HP-death respawn so both paths reset health consistently.

## Requirements

1. Player has a `Health` component (3 HP max), spawned at full health.
2. Stationary enemy entities are placed at hardcoded positions on startup, rendered as red 16×16 squares.
3. Enemies are physically solid (block the player's movement).
4. Touching an enemy deals 1 HP damage, unless the player is currently invincible.
5. After taking damage the player is invincible for 1 second.
6. When HP reaches 0 the player respawns at the spawn position with full HP and no invincibility.
7. Falling into the kill zone also resets HP to full (same respawn path).
8. Enemy logic lives in `src/enemies/` and registers via `EnemiesPlugin`.
9. Health and invincibility logic lives in `src/player/health.rs`.

## Acceptance Criteria

- [ ] Player starts with 3 HP (verified via debug log or future HUD).
- [ ] Walking into a red square reduces HP by 1 and grants 1 second of invincibility.
- [ ] While invincible, additional enemy contacts deal no damage.
- [ ] Reaching 0 HP respawns the player at the start with full health.
- [ ] Falling below the kill zone also respawns with full health.
- [ ] Enemy squares physically block the player's path.
- [ ] No panics or errors during normal play or on repeated deaths.

## Implementation Plan

### New Components

**`Health { current: i32, max: i32 }`** — Attached to the player on spawn. Tracks current and maximum hit points.

**`InvincibilityTimer(Timer)`** — SparseSet marker on the player. Present only during the invincibility window; its absence means the player can be damaged. Inserted by the damage system; removed by `tick_invincibility` when the timer expires.

**`NeedsRespawn`** — SparseSet marker on the player. Inserted by either `kill_zone` (fall death) or the damage system (HP=0). A dedicated `respawn_player` system consumes it and performs the full reset in one place, keeping both death paths consistent.

**`Enemy`** — Marker component on enemy entities.

### Respawn Flow

Using `NeedsRespawn` as a one-frame signal unifies the two death triggers:

1. `kill_zone` detects `position.y < KILL_ZONE_Y` → inserts `NeedsRespawn` (instead of directly resetting).
2. `enemy_contact` detects `health.current <= 0` after a hit → inserts `NeedsRespawn`.
3. `respawn_player` sees `NeedsRespawn` → resets `Position`, `LinearVelocity`, `Health`; removes `InvincibilityTimer` and `NeedsRespawn`.

`respawn_player` is chained after `kill_zone` in the player system chain so fall deaths are handled in the same frame. HP-zero deaths from `enemy_contact` (registered separately in `EnemiesPlugin`) are resolved one frame later — imperceptible in practice.

### Enemy Entities

Each enemy is spawned with:
- `Enemy` marker
- `Sprite` with `color: Color::srgb(1.0, 0.0, 0.0)` and `custom_size: Some(Vec2::splat(16.0))`
- `RigidBody::Static` — immovable terrain-like obstacle
- `Collider::rectangle(16.0, 16.0)` — solid, blocks the player
- `CollisionEventsEnabled` — opts the entity into Avian2D collision event emission (no `Sensor`; enemies are solid)

No `Sensor` is needed because enemies should physically block the player. `CollisionEventsEnabled` alone is sufficient for `CollisionStart` events to fire.

### Damage System (`src/enemies/damage.rs`)

`enemy_contact` reads `EventReader<CollisionStart>` each frame:

1. For each event, determine if one entity is an `Enemy` and the other is the `Player`.
2. If the player currently has `InvincibilityTimer`, skip (already invincible).
3. Otherwise: decrement `health.current` by 1, insert `InvincibilityTimer(Timer::from_seconds(1.0, TimerMode::Once))`.
4. If `health.current <= 0` after the decrement: insert `NeedsRespawn`.

### Invincibility Tick (`src/player/health.rs`)

`tick_invincibility` runs every Update frame:
- Advances `InvincibilityTimer` by `time.delta()`.
- When the timer finishes, removes `InvincibilityTimer` from the player.

### System Ordering

**`PlayerPlugin` chain (Update):**
```
tick_invincibility → update_grounded → player_movement → kill_zone → respawn_player
```

**`EnemiesPlugin` (Update, no explicit ordering):**
```
enemy_contact   (independent of player chain)
```

### Spawn Positions

Defined as tile-grid coordinates in `src/enemies/mod.rs`, converted to world space using `TILEMAP_OFFSET_X`, `TILEMAP_OFFSET_Y`, and `TILE_SIZE` — the same pattern as collectibles.

```rust
const ENEMY_POSITIONS: &[(f32, f32)] = &[
    (8.0, 4.0),
    (18.0, 4.0),
    (30.0, 5.0),
];
```

### Module Structure

```
src/enemies/
    mod.rs          EnemiesPlugin, Enemy marker, ENEMY_POSITIONS, spawn_enemies
    damage.rs       enemy_contact system

src/player/
    health.rs       Health, InvincibilityTimer, NeedsRespawn components;
                    tick_invincibility, respawn_player systems
    mod.rs          export new components; add PLAYER_MAX_HP constant;
                    add tick_invincibility, respawn_player to system chain
    spawn.rs        add Health to player bundle; kill_zone inserts NeedsRespawn
                    instead of directly resetting position

src/main.rs         mod enemies; add EnemiesPlugin
```

## Notes

- **`Position` not `Transform` for respawn**: Avian2D owns the transform for dynamic bodies. The existing `kill_zone` already uses `Position` — `respawn_player` must do the same.
- **`CollisionEventsEnabled` on enemy, not player**: One entity in a pair having `CollisionEventsEnabled` is sufficient for Avian2D to emit `CollisionStart` events. The player already works this way with collectibles.
- **No visual invincibility feedback**: Sprite blinking or opacity change is deferred to a later spec. The invincibility window is still functional without it.
- **Enemy count**: Three enemies for initial playtesting. Adjust positions after verifying the level layout.
- **Future extensibility**: `Health` can be added to enemies in a later spec for killable enemies. `Enemy` can grow variant fields for different hazard types.

## Task Checklist

- [ ] Add `Health`, `InvincibilityTimer`, `NeedsRespawn` components to `src/player/health.rs`
- [ ] Add `tick_invincibility` and `respawn_player` systems to `src/player/health.rs`
- [ ] Add `PLAYER_MAX_HP` constant and export new components in `src/player/mod.rs`
- [ ] Update `src/player/spawn.rs`: add `Health` to player bundle; refactor `kill_zone` to insert `NeedsRespawn`
- [ ] Update `PlayerPlugin` system chain to include `tick_invincibility` and `respawn_player`
- [ ] Create `src/enemies/mod.rs` with `Enemy`, `EnemiesPlugin`, `ENEMY_POSITIONS`, `spawn_enemies`
- [ ] Create `src/enemies/damage.rs` with `enemy_contact` system
- [ ] Wire `EnemiesPlugin` into `main.rs`
- [ ] Verify enemies appear as red squares at correct positions
- [ ] Verify enemy contact reduces HP and grants invincibility
- [ ] Verify HP=0 triggers respawn with full health
- [ ] Verify fall death also resets health
- [ ] Mark spec complete after user verification

## Related Specs

- **014 — Collectibles**: establishes the `CollisionStart` + `CollisionEventsEnabled` contact detection pattern this spec reuses
- **013 — Architecture Refactor**: plugin and module conventions followed throughout

## Related Files

- `src/main.rs` — add `mod enemies`, `EnemiesPlugin`
- `src/enemies/mod.rs` — new file
- `src/enemies/damage.rs` — new file
- `src/player/health.rs` — new file
- `src/player/mod.rs` — new components, constant, system chain update
- `src/player/spawn.rs` — `Health` in bundle, `kill_zone` refactor
