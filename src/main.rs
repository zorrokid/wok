use bevy::{app::AppExit, ecs::message::MessageWriter, prelude::*};

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
        .add_systems(Update, exit_on_esc)
        .run();
}

fn exit_on_esc(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit_writer: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit_writer.write(AppExit::Success);
    }
}
