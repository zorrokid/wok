pub mod tile;
pub mod transition;

use avian2d::prelude::{Collider, RigidBody};
use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{Assets, AssetServer, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        observer::On,
        query::With,
        system::{Commands, Query, Res},
    },
    prelude::IntoScheduleConfigs,
    transform::components::Transform,
};
use bevy_ecs_tiled::prelude::{
    ColliderCreated, MapCreated, TiledEvent, TiledFilter, TiledMap, TiledMapAsset,
    TilemapAnchor, TiledPhysicsAvianBackend, TiledPhysicsSettings,
};

use crate::collectibles::{Collectible, spawn_collectibles_at};
use crate::enemies::{Enemy, spawn_enemies_at};
use crate::level::tile::{MapDimensions, map_dimensions_from_asset};
use crate::level::transition::{
    CurrentMap, LevelTransition, LevelTransitionEvent, TransitionCooldown,
    apply_transition, setup_transition_colliders, trigger_transition,
};
use crate::player::{Player, spawn::spawn_player_at};


pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LevelTransition>()
            .init_resource::<TransitionCooldown>()
            .init_resource::<MapDimensions>()
            .add_message::<LevelTransitionEvent>()
            .add_observer(setup_transition_colliders)
            .add_observer(on_map_created)
            .add_systems(Startup, setup_tilemap)
            .add_systems(
                Update,
                (
                    trigger_transition,
                    apply_transition.after(trigger_transition),
                ),
            );
    }
}

/// Marker component for the invisible static colliders at the map's left and
/// right edges. Tagged so they can be despawned and re-spawned each time a
/// new map loads (possibly with different dimensions).
#[derive(Component)]
pub struct LevelBoundsWall;

/// Global observer: fires once per `TiledEvent<MapCreated>` for every map load.
///
/// Responsibilities:
/// 1. Reads the map asset to compute `MapDimensions` and inserts it as a resource.
/// 2. Despawns any old level-bounds walls and spawns fresh ones sized to the new map.
/// 3. On the **first** map load only (guarded by empty-query checks), spawns the
///    player, collectibles, and enemies. Subsequent loads (transitions) skip this
///    because the entities already exist.
pub fn on_map_created(
    ev: On<TiledEvent<MapCreated>>,
    mut commands: Commands,
    assets: Res<Assets<TiledMapAsset>>,
    asset_server: Res<AssetServer>,
    walls_q: Query<Entity, With<LevelBoundsWall>>,
    player_q: Query<(), With<Player>>,
    collectible_q: Query<(), With<Collectible>>,
    enemy_q: Query<(), With<Enemy>>,
) {
    let Some(asset) = ev.event().get_map_asset(&assets) else {
        return;
    };
    let dims = map_dimensions_from_asset(asset);

    // Update the MapDimensions resource so all systems see the new values.
    // Note: commands are deferred — the local `dims` variable holds the live
    // data and is used directly for spawning below.
    commands.insert_resource(dims);

    // Despawn old walls and spawn new ones sized to the loaded map.
    for e in walls_q.iter() {
        commands.entity(e).despawn();
    }
    let half_width = dims.width_px() / 2.0;
    let height = dims.height_px();
    commands.spawn((
        LevelBoundsWall,
        Collider::rectangle(1.0, height),
        RigidBody::Static,
        Transform::from_xyz(-half_width, 0.0, 0.0),
    ));
    commands.spawn((
        LevelBoundsWall,
        Collider::rectangle(1.0, height),
        RigidBody::Static,
        Transform::from_xyz(half_width, 0.0, 0.0),
    ));

    // First-time-only spawns: skip on map transitions where the entities exist.
    if player_q.is_empty() {
        spawn_player_at(&dims, &mut commands, &asset_server);
    }
    if collectible_q.is_empty() {
        spawn_collectibles_at(&dims, &mut commands);
    }
    if enemy_q.is_empty() {
        spawn_enemies_at(&dims, &mut commands);
    }
}

/// Named function for the ColliderCreated observer so it can be reattached
/// to new map entities spawned during transitions (see `apply_transition`).
///
/// TiledPhysicsPlugin creates an Avian2D Collider for each solid tile but
/// leaves the RigidBody type unset. This observer fires once per collider
/// entity and marks it as Static so Avian includes it in collision resolution.
pub fn on_collider_created(ev: On<TiledEvent<ColliderCreated>>, mut commands: Commands) {
    commands.entity(ev.event().origin).insert(RigidBody::Static);
}

pub fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map1.tmx");
    let entity = commands
        .spawn((
            TiledMap(map_handle),
            TilemapAnchor::Center,
            // Disable automatic object-layer collider creation. Without this,
            // collider_from_object creates a solid collider for every object
            // (including transition zones), blocking the player. We create
            // transition zone colliders manually in setup_transition_colliders.
            TiledPhysicsSettings::<TiledPhysicsAvianBackend> {
                objects_layer_filter: TiledFilter::None,
                ..Default::default()
            },
        ))
        .observe(on_collider_created)
        .id();

    commands.insert_resource(CurrentMap(entity));
}
