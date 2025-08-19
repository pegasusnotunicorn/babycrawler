use crate::GameState;
use crate::network::send::send_reset_game;
use turbo::*;

pub use crate::game::cards::card_input::{ handle_card_drag, handle_play_area_buttons };
pub use crate::game::map::tile_input::handle_tile_selection;

pub fn handle_input(state: &mut GameState) {
    let pointer = mouse::screen();
    let pointer_xy = (pointer.x, pointer.y);
    handle_card_drag(state, &pointer, pointer_xy);
    handle_play_area_buttons(state, &pointer);
    handle_tile_selection(state, &pointer, pointer_xy);

    let gp = gamepad::get(0);
    if gp.a.just_pressed() {
        send_reset_game();
    }
}
