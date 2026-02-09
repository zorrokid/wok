use bevy::{app::AppExit, ecs::message::MessageWriter, prelude::*};
use bevy_ecs_tilemap::prelude::*;

// Player marker component
#[derive(Component)]
struct Player;

// Level data: 0 = empty/air, 1 = solid platform
const LEVEL_WIDTH: u32 = 20;
const LEVEL_HEIGHT: u32 = 15;
const LEVEL_DATA: [[u32; LEVEL_WIDTH as usize]; LEVEL_HEIGHT as usize] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "WoK".into(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, (setup_tilemap, spawn_player))
        .add_systems(Update, exit_on_esc)
        .run();
}

fn setup_tilemap(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Create tilemap entity
    let map_size = TilemapSize {
        x: LEVEL_WIDTH,
        y: LEVEL_HEIGHT,
    };
    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = TilemapGridSize { x: 16.0, y: 16.0 };

    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    // Spawn tiles from level data
    // Note: Array index 0 is top of level, but Y=0 is bottom in tilemap
    for y in 0..LEVEL_HEIGHT {
        for x in 0..LEVEL_WIDTH {
            let tile_type = LEVEL_DATA[(LEVEL_HEIGHT - 1 - y) as usize][x as usize];
            
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
        transform: Transform::from_xyz(
            -(LEVEL_WIDTH as f32 * 16.0) / 2.0,
            -(LEVEL_HEIGHT as f32 * 16.0) / 2.0,
            0.0,
        ),
        ..default()
    });
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load player sprite
    let player_texture = asset_server.load("player.png");
    
    // Calculate spawn position: 3 tiles from left, on top of ground (3 tile rows)
    // Ground starts at y=0, is 3 tiles (48 pixels) tall
    // Player sprite center should be at ground + half player height
    let spawn_x = -(LEVEL_WIDTH as f32 * 16.0) / 2.0 + (3.0 * 16.0);
    let spawn_y = -(LEVEL_HEIGHT as f32 * 16.0) / 2.0 + (3.0 * 16.0) + 8.0;
    
    commands.spawn((
        Player,
        Sprite::from_image(player_texture),
        Transform::from_xyz(spawn_x, spawn_y, 10.0),
    ));
}

fn exit_on_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit_writer.write(AppExit::Success);
    }
}
