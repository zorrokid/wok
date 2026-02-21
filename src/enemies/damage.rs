use avian2d::prelude::*;
use bevy::prelude::*;

use crate::enemies::Enemy;
use crate::player::health::apply_damage;
use crate::player::{Health, InvincibilityTimer, NeedsRespawn, Player};

/// Deals damage to the player whenever they are touching an enemy without invincibility.
///
/// Runs every Update frame and queries the Avian2D `Collisions` system param for ongoing
/// contacts, rather than relying on `CollisionStart` events. This means damage is re-applied
/// each time the invincibility window expires while the player is still in contact — e.g.
/// standing on top of an enemy.
pub fn contact_damage(
    mut commands: Commands,
    collisions: Collisions,
    player_query: Query<(Entity, &Health, Has<InvincibilityTimer>), With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    let Ok((player_entity, health, is_invincible)) = player_query.single() else {
        return;
    };

    if is_invincible {
        return;
    }

    // Check whether any enemy is currently in contact with the player.
    let touching_enemy = enemy_query
        .iter()
        .any(|enemy_entity| collisions.contains(player_entity, enemy_entity));

    if !touching_enemy {
        return;
    }

    let new_health = Health { current: apply_damage(health.current), max: health.max };
    info!("Player hit by enemy: {} -> {} HP", health.current, new_health.current);

    let mut entity_commands = commands.entity(player_entity);
    entity_commands.insert(new_health);
    // Grant invincibility to prevent multiple hits within the same contact window.
    entity_commands.insert(InvincibilityTimer(Timer::from_seconds(1.0, TimerMode::Once)));

    if new_health.is_dead() {
        entity_commands.insert(NeedsRespawn);
    }
}
