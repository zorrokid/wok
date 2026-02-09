use bevy::{
    app::AppExit,
    ecs::{message::MessageWriter, system::Res},
    input::{ButtonInput, keyboard::KeyCode},
};

pub fn exit_on_esc(keyboard: Res<ButtonInput<KeyCode>>, mut exit_writer: MessageWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit_writer.write(AppExit::Success);
    }
}
