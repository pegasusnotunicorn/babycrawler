use crate::GameState;
use turbo::*;

pub fn draw_debug(state: &GameState) {
    if !state.debug {
        return;
    }

    let canvas_height = bounds::screen().h();
    let debug_y = canvas_height / 2 + 15;
    let y_spacer = 10;

    let id = &state.user;
    let debug_str = format!("Player ID: {}", id);
    text!(&debug_str, x = 8, y = debug_y, font = "small", color = 0xffffffff);

    let current_turn_player_id = state.current_turn
        .as_ref()
        .map(|turn| turn.player_id.clone())
        .unwrap_or("None".to_string());
    let debug_str = format!("Current Turn Player ID: {}", current_turn_player_id);
    text!(&debug_str, x = 8, y = debug_y + y_spacer, font = "small", color = 0xffffffff);

    let selected_card = &state.selected_card;
    let debug_str = format!(
        "Selected Card: {:?}",
        selected_card.as_ref().map(|c| c.name.clone())
    );
    text!(&debug_str, x = 8, y = debug_y + y_spacer * 2, font = "small", color = 0xffffffff);

    let local_player = state.get_local_player();
    let debug_str = format!("Local Player: {:?}", local_player);
    text!(&debug_str, x = 8, y = debug_y + y_spacer * 3, font = "small", color = 0xffffffff);
}
