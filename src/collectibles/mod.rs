pub mod collection;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::collectibles::collection::collect_items;
use crate::level::tile::MapDimensions;

// Tile-grid (column, row) positions for collectible placement.
// Temporary: spec 018 will replace these with Tiled object-layer entities.
const COLLECTIBLE_POSITIONS: &[(f32, f32)] = &[
    (5.0, 4.0),
    (10.0, 4.0),
    (15.0, 6.0),
    (20.0, 4.0),
    (25.0, 8.0),
];

#[derive(Component)]
pub struct Collectible;

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, collect_items);
    }
}

/// Spawns collectible items at tile-grid positions computed from `dims`.
///
/// Called from `on_map_created` on first map load only (guarded by a
/// `Query<(), With<Collectible>>` check). Temporary until spec 018 replaces
/// placement with Tiled object-layer entities.
pub fn spawn_collectibles_at(dims: &MapDimensions, commands: &mut Commands) {
    for &(col, row) in COLLECTIBLE_POSITIONS {
        let pos = dims.tile_to_world(col, row);

        commands.spawn((
            Collectible,
            Sprite {
                color: Color::srgb(1.0, 1.0, 0.0),
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Collider::rectangle(16.0, 16.0),
            Sensor,
            // Required by Avian2D to emit CollisionStart/CollisionEnd messages for this entity.
            CollisionEventsEnabled,
        ));
    }
}
