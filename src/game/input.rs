use crate::game::tile::Tile;
use crate::GameState;
use turbo::*;

pub fn handle_input(state: &mut GameState, pointer: &mouse::ScreenMouse, pointer_xy: (i32, i32)) {
    // --- Card selection logic ---
    if let Some(player) = state.get_local_player() {
        let hand = &player.hand;
        let canvas_width = bounds::screen().w() - crate::game::constants::GAME_PADDING * 2;
        let canvas_height = bounds::screen().h();
        if
            let Some(idx) = crate::game::hand::hovered_card_index(
                hand,
                pointer_xy,
                canvas_width,
                canvas_height
            )
        {
            if pointer.just_pressed() {
                let card = hand[idx].clone();
                crate::game::card::Card::toggle_selection(&mut state.selected_cards, &card);

                // Highlight tiles for the newly selected card
                crate::game::tile::clear_highlights(&mut state.tiles);
                if state.selected_cards.len() == 1 {
                    let card = &state.selected_cards[0];
                    if let Some(player) = state.get_local_player() {
                        card.effect.highlight_tiles(player.position, &mut state.tiles);
                    }
                }
                return;
            }
        }
    }
    // --- Tile/cell input logic (only if a card is selected) ---
    if state.selected_cards.len() != 1 {
        return;
    }
    let card = state.selected_cards[0].clone();
    let (_canvas_width, _canvas_height, tile_size, offset_x, offset_y) =
        state.get_board_layout(false);
    for (i, _tile) in state.tiles.iter().enumerate() {
        let (tx, ty) = Tile::screen_position(i, tile_size, offset_x, offset_y);
        let bounds = Bounds::new(tx, ty, tile_size, tile_size);
        let mx = pointer_xy.0;
        let my = pointer_xy.1;
        if crate::game::util::point_in_bounds(mx, my, &bounds) {
            if pointer.just_pressed() {
                card.effect.apply_effect(state, i);
            }
            break;
        }
    }
}
