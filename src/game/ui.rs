use crate::game::constants::{ PLAYER_1_COLOR, PLAYER_2_COLOR };
use turbo::*;

pub fn draw_turn_label(is_my_turn: bool, tile_size: u32) {
    let canvas_bounds = bounds::screen();
    let canvas_width = canvas_bounds.w();
    let canvas_height = canvas_bounds.h();

    let turn_label = if is_my_turn { "Your turn!" } else { "Waiting for other player..." };

    let text_x = canvas_width / 2 - ((turn_label.len() as u32) * 6) / 2; // ~6px per char
    let text_y = canvas_height - tile_size * 2; // adjust vertically to sit above hand

    text!(turn_label, x = text_x, y = text_y, font = "large", color = 0xffffffff);
}
