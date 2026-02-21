pub mod damage;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::enemies::damage::contact_damage;
use crate::level::tile::MapDimensions;

/// Marker component for stationary enemy hazard entities.
#[derive(Component)]
pub struct Enemy;

// Tile-grid (column, row) positions for enemy placement.
// Temporary: spec 018 will replace these with Tiled object-layer entities.
const ENEMY_POSITIONS: &[(f32, f32)] = &[
    (8.0, 4.0),
    (18.0, 4.0),
    (30.0, 5.0),
];

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, contact_damage);
    }
}

/// Spawns stationary enemy hazards at tile-grid positions computed from `dims`.
///
/// Called from `on_map_created` on first map load only (guarded by a
/// `Query<(), With<Enemy>>` check). Temporary until spec 018 replaces
/// placement with Tiled object-layer entities.
pub fn spawn_enemies_at(dims: &MapDimensions, commands: &mut Commands) {
    for &(col, row) in ENEMY_POSITIONS {
        let pos = dims.tile_to_world(col, row);

        commands.spawn((
            Enemy,
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.0),
            RigidBody::Static,
            Collider::rectangle(16.0, 16.0),
            // CollisionEventsEnabled opts this entity into Avian2D's CollisionStart/CollisionEnd
            // event stream. One entity in a pair having this is sufficient — no Sensor needed
            // because enemies are solid and should physically block the player.
            CollisionEventsEnabled,
        ));
    }
}
