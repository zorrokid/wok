use avian2d::prelude::*;
use bevy::prelude::*;

use crate::level::tile::MapDimensions;
use crate::player::PLAYER_MAX_HP;
use crate::player::spawn::initial_spawn_pos;

/// Current and maximum hit points for the player.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn full(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

/// Present only during the invincibility window after taking damage.
/// Inserted by `enemy_contact`; removed here when the timer expires.
/// SparseSet is efficient for components that toggle frequently.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct InvincibilityTimer(pub Timer);

/// One-frame signal: something triggered a player death. Consumed by `respawn_player`.
/// Both the kill zone (fall) and enemy contact (HP=0) insert this, keeping respawn logic
/// in one place regardless of the death cause.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct NeedsRespawn;

/// Advances the invincibility countdown and removes the component when it expires.
pub fn tick_invincibility(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut InvincibilityTimer)>,
) {
    for (entity, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).remove::<InvincibilityTimer>();
        }
    }
}

/// Resets position, velocity, health, and invincibility for any player marked `NeedsRespawn`.
///
/// Uses `Position` (not `Transform`) because Avian2D owns the transform for dynamic
/// rigid bodies. Writing to `Transform` directly would be overwritten by the physics step.
pub fn respawn_player(
    mut commands: Commands,
    map_dims: Res<MapDimensions>,
    mut query: Query<(Entity, &mut Position, &mut LinearVelocity, &mut Health), With<NeedsRespawn>>,
) {
    for (entity, mut position, mut velocity, mut health) in &mut query {
        let spawn_pos = initial_spawn_pos(&map_dims);
        *position = Position::from_xy(spawn_pos.x, spawn_pos.y);
        *velocity = LinearVelocity::ZERO;
        *health = Health::full(PLAYER_MAX_HP);
        commands
            .entity(entity)
            .remove::<InvincibilityTimer>()
            .remove::<NeedsRespawn>();
        info!("Player respawned with full health ({} HP)", PLAYER_MAX_HP);
    }
}

/// Returns the resulting health after applying one point of damage, clamped to zero.
pub fn apply_damage(current_hp: i32) -> i32 {
    (current_hp - 1).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_full_sets_current_to_max() {
        let h = Health::full(3);
        assert_eq!(h.current, 3);
        assert_eq!(h.max, 3);
    }

    #[test]
    fn health_is_dead_at_zero() {
        let h = Health { current: 0, max: 3 };
        assert!(h.is_dead());
    }

    #[test]
    fn health_is_not_dead_above_zero() {
        let h = Health { current: 1, max: 3 };
        assert!(!h.is_dead());
    }

    #[test]
    fn apply_damage_decrements_by_one() {
        assert_eq!(apply_damage(3), 2);
        assert_eq!(apply_damage(1), 0);
    }

    #[test]
    fn apply_damage_does_not_go_below_zero() {
        assert_eq!(apply_damage(0), 0);
    }
}
