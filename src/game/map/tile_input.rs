use crate::GameState;
use turbo::*;
use crate::game::util::point_in_bounds;
use crate::game::map::tile::Tile;

pub fn handle_tile_selection(
    state: &mut GameState,
    pointer: &mouse::ScreenMouse,
    pointer_xy: (i32, i32)
) {
    if state.selected_card.is_none() {
        return;
    }
    let card = state.selected_card.as_ref().unwrap().clone();
    let (_canvas_width, _canvas_height, tile_size, offset_x, offset_y) =
        state.get_board_layout(false);
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
