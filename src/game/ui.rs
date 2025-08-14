use turbo::*;

use crate::game::constants::{ GAME_PADDING, FONT_HEIGHT };
use crate::network::send::send_end_turn;

const BUTTON_WIDTH: u32 = 100;

/// Helper function to draw a text box with outline, fill, and text
pub fn draw_text_box(
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    text: &str,
    text_color: u32,
    fill_color: u32
) {
    let outline_color = 0xffffffff;

    // Draw outline rect
    rect!(
        x = x as i32,
        y = y as i32,
        w = width,
        h = height,
        color = outline_color,
        border_radius = 2
    );

    // Draw fill rect (slightly smaller for border effect)
    rect!(
        x = (x + 2.0) as i32,
        y = (y + 2.0) as i32,
        w = width - 4,
        h = height - 4,
        color = fill_color,
        border_radius = 2
    );

    // Position text with fixed offset from top, similar to card text positioning
    let text_x = x + 16.0;
    let text_y = y + 8.0;

    // Draw text
    text!(text, x = text_x as i32, y = text_y as i32, font = "large", color = text_color);
}

// Draw a button to end the turn
pub fn draw_end_turn_button() {
    let font_height = FONT_HEIGHT;
    let button_height = font_height + GAME_PADDING;

    // flush right
    let button_x = bounds::screen().w() - BUTTON_WIDTH - GAME_PADDING;
    let button_y = bounds::screen().h() - button_height - GAME_PADDING;

    // when hovered, the fill color should be black
    // otherwise normally fill color is light red
    let mut fill_color = 0x600000ff;
    let pointer = mouse::screen();
    let button_bounds = turbo::Bounds::new(button_x, button_y, BUTTON_WIDTH, button_height);
    let pointer_bounds = turbo::Bounds::new(pointer.x as u32, pointer.y as u32, 1, 1);

    if button_bounds.contains(&pointer_bounds) {
        fill_color = 0x222222ff;
    }

    draw_text_box(
        button_x as f32,
        button_y as f32,
        BUTTON_WIDTH,
        button_height,
        "End Turn",
        0xffffffff,
        fill_color
    );

    // if the button is pressed, send end turn
    if button_bounds.contains(&pointer_bounds) && pointer.just_pressed() {
        send_end_turn();
    }
}

/// Draws a message centered above the hand, in the same location as the turn label.
pub fn draw_text(text: &str, is_my_turn: bool) {
    let canvas_bounds = bounds::screen();
    let canvas_width = canvas_bounds.w();
    let canvas_height = canvas_bounds.h();
    let font_height = FONT_HEIGHT;
    let bar_height = font_height + GAME_PADDING;
    let text_y = canvas_height - bar_height - 8; // no idea why 8 is needed

    let rect_x = GAME_PADDING;
    let rect_y = text_y - GAME_PADDING / 2;
    // use full screen width if not my turn
    let rect_w = if is_my_turn {
        canvas_width - GAME_PADDING * 3 - BUTTON_WIDTH
    } else {
        canvas_width - GAME_PADDING * 2
    };
    let rect_h = bar_height;

    draw_text_box(rect_x as f32, rect_y as f32, rect_w, rect_h, text, 0xffffffff, 0x222222ff);
}

pub fn draw_menu(game_over: bool) {
    let menu_items = if game_over {
        ["Press SPACE to return to menu"]
    } else {
        ["Press SPACE to start"]
    };
    if let Some(item) = menu_items.first() {
        draw_text(item, false);
    }
}

pub fn draw_turn_label(is_my_turn: bool, _game_state: &crate::GameState) {
    let turn_label = if is_my_turn { "It's your turn!" } else { "Please wait for your turn..." };
    draw_text(turn_label, is_my_turn);
    if is_my_turn {
        draw_end_turn_button();
    }
}

/// Draws a waiting message if no player is connected.
pub fn draw_waiting_for_players(_game_state: &crate::GameState) {
    draw_text("Waiting for players...", false);
}

/// Draws the game over screen with winner/loser information
pub fn draw_game_over_screen(winner_id: &str, current_user_id: &str) {
    draw_menu(true);
    let canvas_bounds = bounds::screen();
    let canvas_width = canvas_bounds.w();
    let canvas_height = canvas_bounds.h();
    let is_winner = current_user_id == winner_id;

    // Draw main game over box using draw_text_box
    let box_width = canvas_width - GAME_PADDING * 2;
    let box_height = FONT_HEIGHT + GAME_PADDING;
    let box_x = GAME_PADDING as f32;
    let box_y = (canvas_height - box_height * 2 - GAME_PADDING * 2) as f32;

    // Draw title
    let title = if is_winner { "CONGRATS, YOU WON!" } else { "GAMEOVER, YOU LOST!" };
    let fill_color = if is_winner { 0x118811ff } else { 0xff2222ff };

    draw_text_box(
        box_x,
        box_y,
        box_width,
        box_height,
        title,
        0xffffffff, // White text
        fill_color // Game background color
    );
}
