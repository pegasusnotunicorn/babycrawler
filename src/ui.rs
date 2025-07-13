use turbo::*;

use crate::constants::{ PLAYER_1_COLOR, PLAYER_2_COLOR };

pub fn draw_turn_label(current_turn: usize, tile_size: u32) {
    let canvas_bounds = bounds::screen();
    let canvas_width = canvas_bounds.w();
    let canvas_height = canvas_bounds.h();

    let turn_label = format!("Player {}'s Turn", current_turn + 1);

    let text_x = canvas_width / 2 - ((turn_label.len() as u32) * 6) / 2; // ~6px per char
    let text_y = canvas_height - tile_size * 2; // adjust vertically to sit above hand

    text!(
        &turn_label,
        x = text_x,
        y = text_y,
        font = "large",
        color = if current_turn == 0 { PLAYER_1_COLOR } else { PLAYER_2_COLOR }
    );
}
