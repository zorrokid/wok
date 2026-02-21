pub mod damage;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::enemies::damage::contact_damage;
use crate::level::tile::{TILE_SIZE, TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y};

/// Marker component for stationary enemy hazard entities.
#[derive(Component)]
pub struct Enemy;

// Tile-grid (column, row) positions for enemy placement.
// Converted to world space the same way collectibles are placed.
const ENEMY_POSITIONS: &[(f32, f32)] = &[
    (8.0, 4.0),
    (18.0, 4.0),
    (30.0, 5.0),
];

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemies)
            .add_systems(Update, contact_damage);
    }
}

fn spawn_enemies(mut commands: Commands) {
    for &(col, row) in ENEMY_POSITIONS {
        let x = TILEMAP_OFFSET_X + col * TILE_SIZE + TILE_SIZE / 2.0;
        let y = TILEMAP_OFFSET_Y + row * TILE_SIZE + TILE_SIZE / 2.0;

        commands.spawn((
            Enemy,
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.0),
            RigidBody::Static,
            Collider::rectangle(16.0, 16.0),
            // CollisionEventsEnabled opts this entity into Avian2D's CollisionStart/CollisionEnd
            // event stream. One entity in a pair having this is sufficient — no Sensor needed
            // because enemies are solid and should physically block the player.
            CollisionEventsEnabled,
        ));
    }
}
