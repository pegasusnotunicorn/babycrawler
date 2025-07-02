use turbo::*;

use crate::constants::{ PLAYER_1_COLOR, PLAYER_2_COLOR };

pub fn draw_turn_label(current_turn: usize) {
    let turn_label = format!("Player {}'s Turn", current_turn + 1);
    text!(
        &turn_label,
        x = 5,
        y = 5,
        font = "small",
        color = if current_turn == 0 { PLAYER_1_COLOR } else { PLAYER_2_COLOR }
    );
}
