use crate::GameState;
use turbo::*;

pub use crate::game::cards::card_input::{ handle_card_drag, handle_play_area_buttons };
pub use crate::game::map::tile_input::handle_tile_selection;

pub fn handle_input(state: &mut GameState, pointer: &mouse::ScreenMouse, pointer_xy: (i32, i32)) {
    handle_card_drag(state, pointer, pointer_xy);
    handle_play_area_buttons(state, pointer);
    handle_tile_selection(state, pointer, pointer_xy);
}
