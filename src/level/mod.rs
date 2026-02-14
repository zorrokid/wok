pub mod tile;

use bevy::{
    asset::{AssetServer, Handle},
    camera::Camera2d,
    ecs::system::{Commands, Res},
    image::Image,
    transform::components::Transform,
    utils::default,
};
use bevy_ecs_tilemap::{
    TilemapBundle,
    anchor::TilemapAnchor,
    map::{TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize},
    tiles::{TileBundle, TilePos, TileStorage, TileTextureIndex},
};

// Level data: 0 = empty/air, 1 = solid platform
pub const LEVEL_WIDTH_IN_TILES: u32 = 20;
pub const LEVEL_HEIGHT_IN_TILES: u32 = 15;

/// Level layout: 2D array of tile types (0 = empty, 1 = solid)
/// currently left side of the level in world coordinates is at
/// x = -(LEVEL_WIDTH_IN_TILES * TILE_SIZE / 2.0)
/// e.g. for 20 tiles wide and 16 pixels per tile, left edge is at x = -160.0
/// and right side is at x = (LEVEL_WIDTH_IN_TILES * TILE_SIZE / 2.0)
pub const LEVEL_DATA: [[u32; LEVEL_WIDTH_IN_TILES as usize]; LEVEL_HEIGHT_IN_TILES as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];

pub fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Create tilemap entity
    let map_size = TilemapSize {
        x: LEVEL_WIDTH_IN_TILES,
        y: LEVEL_HEIGHT_IN_TILES,
    };
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = TilemapGridSize { x: 16.0, y: 16.0 };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    // Spawn tiles from level data
    // Note: Array index 0 is top of level, but Y=0 is bottom in tilemap
    for y in 0..LEVEL_HEIGHT_IN_TILES {
        for x in 0..LEVEL_WIDTH_IN_TILES {
            let tile_type = LEVEL_DATA[(LEVEL_HEIGHT_IN_TILES - 1 - y) as usize][x as usize];

            if tile_type > 0 {
                let tile_pos = TilePos { x, y };
                let tile_entity = commands
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(tile_type - 1),
                        ..default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    }

    // Load placeholder tileset texture
    let texture_handle: Handle<Image> = asset_server.load("tileset.png");

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::BottomLeft,
        transform: Transform::from_xyz(
            -(LEVEL_WIDTH_IN_TILES as f32 * 16.0) / 2.0,
            -(LEVEL_HEIGHT_IN_TILES as f32 * 16.0) / 2.0,
            0.0,
        ),
        ..default()
    });
}
