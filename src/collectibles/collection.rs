use avian2d::prelude::*;
use bevy::prelude::*;

use crate::collectibles::Collectible;
use crate::player::Player;

pub fn collect_items(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionStart>,
    player_query: Query<(), With<Player>>,
    collectible_query: Query<(), With<Collectible>>,
) {
    for event in collision_reader.read() {
        let (collectible_entity, other_entity) =
            if collectible_query.contains(event.collider1) {
                (event.collider1, event.collider2)
            } else if collectible_query.contains(event.collider2) {
                (event.collider2, event.collider1)
            } else {
                continue;
            };

        if player_query.contains(other_entity) {
            commands.entity(collectible_entity).despawn();
        }
    }
}
