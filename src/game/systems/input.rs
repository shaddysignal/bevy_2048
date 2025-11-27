use bevy::prelude::*;
use crate::game::components::*;
use crate::game::states::*;

pub fn generate_direction_messages(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut direction_message: MessageWriter<DirectionMessage>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        direction_message.write(DirectionMessage(Direction::Left));
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        direction_message.write(DirectionMessage(Direction::Right));
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        direction_message.write(DirectionMessage(Direction::Down));
    } else if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        direction_message.write(DirectionMessage(Direction::Up));
    }

    if keyboard_input.any_just_pressed([
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowDown,
        KeyCode::ArrowUp,
    ]) {
        game_state.set(GameState::Process);
    }
}