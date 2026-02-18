pub mod tile;

use avian2d::prelude::RigidBody;
use bevy::{
    asset::{AssetServer, Handle},
    camera::Camera2d,
    ecs::{observer::On, system::{Commands, Res}},
};
use bevy_ecs_tiled::prelude::{
    ColliderCreated, TiledEvent, TiledMap, TiledMapAsset, TilemapAnchor,
};

pub fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // TiledMap triggers bevy_ecs_tiled to parse the .tmx file and spawn child
    // entities for each layer and tile. TilemapAnchor::Center positions the
    // tilemap so that its center aligns with the world origin (0, 0).
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map.tmx");
    commands
        .spawn((TiledMap(map_handle), TilemapAnchor::Center))
        // TiledPhysicsPlugin creates an Avian2D Collider for each solid tile,
        // but leaves the RigidBody type unset — the caller decides whether tiles
        // should be Static, Dynamic, or Kinematic. This observer fires once per
        // collider entity and marks it as Static (immovable terrain). Without
        // this, the colliders exist but Avian ignores them in collision resolution.
        .observe(
            |ev: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(ev.event().origin)
                    .insert(RigidBody::Static);
            },
        );
}
