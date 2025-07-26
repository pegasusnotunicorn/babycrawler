use turbo::*;

use crate::game::constants::GAME_PADDING;

/// Draws a message centered above the hand, in the same location as the turn label.
pub fn draw_text(text: &str) {
    let canvas_bounds = bounds::screen();
    let canvas_width = canvas_bounds.w();
    let canvas_height = canvas_bounds.h();
    let font_height = 12;
    let bar_height = font_height + GAME_PADDING;
    // Place text bar just above the cards, with only GAME_PADDING as the gap
    let text_y = canvas_height - bar_height - 8; // no idea why 8 is needed
    let text_x = GAME_PADDING * 2;

    let rect_x = GAME_PADDING;
    let rect_y = text_y - GAME_PADDING / 2;
    let rect_w = canvas_width - GAME_PADDING * 2;
    let rect_h = bar_height;
    let outline_color = 0xffffffff;

    rect!(x = rect_x, y = rect_y, w = rect_w, h = rect_h, color = outline_color, border_radius = 2);
    rect!(x = rect_x + 2, y = rect_y + 2, w = rect_w - 4, h = rect_h - 4, color = 0x222222ff);
    text!(text, x = text_x, y = text_y, font = "large", color = outline_color);
}

pub fn draw_menu() {
    let menu_items = ["Press SPACE to start"];
    if let Some(item) = menu_items.first() {
        draw_text(item);
    }
}

pub fn draw_turn_label(is_my_turn: bool, _game_state: &crate::GameState) {
    let turn_label = if is_my_turn { "Your turn!" } else { "Waiting for other player..." };
    draw_text(turn_label);
}

/// Draws a waiting message if no player is connected.
pub fn draw_waiting_for_players(_game_state: &crate::GameState) {
    draw_text("Waiting for players to connect...");
}
