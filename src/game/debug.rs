use crate::GameState;
use turbo::*;

pub fn draw_debug(state: &GameState) {
    if !state.debug {
        return;
    }

    let font_size = "small";

    let canvas_height = bounds::screen().h();
    let debug_y = canvas_height / 2 + 15;
    // let y_spacer = 10;

    let id = &state.user;
    let debug_str = format!("Player ID: {}", id);
    text!(&debug_str, x = 8, y = debug_y, font = font_size, color = 0xffffffff);
}
