pub mod collection;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::level::tile::{TILE_SIZE, TILEMAP_OFFSET_X, TILEMAP_OFFSET_Y};

use crate::collectibles::collection::collect_items;

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
        app.add_systems(Startup, spawn_collectibles)
            .add_systems(Update, collect_items);
    }
}

fn spawn_collectibles(mut commands: Commands) {
    for &(col, row) in COLLECTIBLE_POSITIONS {
        let x = TILEMAP_OFFSET_X + col * TILE_SIZE + TILE_SIZE / 2.0;
        let y = TILEMAP_OFFSET_Y + row * TILE_SIZE + TILE_SIZE / 2.0;

        commands.spawn((
            Collectible,
            Sprite {
                color: Color::srgb(1.0, 1.0, 0.0),
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.0),
            Collider::rectangle(16.0, 16.0),
            Sensor,
            // Required by Avian2D to emit CollisionStart/CollisionEnd messages for this entity.
            CollisionEventsEnabled,
        ));
    }
}
