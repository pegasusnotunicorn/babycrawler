use crate::tile::Tile;
use crate::{ GameState };
use crate::util::point_in_bounds;
use turbo::*;

pub fn handle_input(
    state: &mut GameState,
    pointer: &mouse::ScreenMouse,
    pointer_xy: (i32, i32),
    tile_size: u32,
    offset_x: u32,
    offset_y: u32
) {
    if state.selected_cards.len() != 1 {
        return;
    }

    let card = state.selected_cards[0].clone();

    for (i, _tile) in state.tiles.iter().enumerate() {
        let (tx, ty) = Tile::screen_position(i, tile_size, offset_x, offset_y);
        let bounds = Bounds::new(tx, ty, tile_size, tile_size);
        let mx = pointer_xy.0;
        let my = pointer_xy.1;

        if point_in_bounds(mx, my, &bounds) {
            if pointer.just_pressed() {
                card.effect.apply_effect(state, i);
            }
            break;
        }
    }
}
