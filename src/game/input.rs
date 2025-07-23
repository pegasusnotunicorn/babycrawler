use crate::GameState;
use crate::game::hand::hovered_card_index;
use crate::game::tile::{ Tile, clear_highlights };
use turbo::*;

pub fn handle_input(state: &mut GameState, pointer: &mouse::ScreenMouse, pointer_xy: (i32, i32)) {
    handle_card_selection(state, pointer, pointer_xy);
    handle_tile_selection(state, pointer, pointer_xy);
}

fn highlight_selected_card_tiles(state: &mut GameState) {
    // Highlight tiles for the newly selected card
    clear_highlights(&mut state.tiles);
    if state.selected_cards.len() == 1 {
        let card = &state.selected_cards[0];
        if let Some(player) = state.get_local_player() {
            card.effect.highlight_tiles(player.position, &mut state.tiles);
        }
    }
}

pub fn handle_card_selection(
    state: &mut GameState,
    pointer: &mouse::ScreenMouse,
    pointer_xy: (i32, i32)
) {
    if let Some(player) = state.get_local_player() {
        let hand = &player.hand;
        let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
            state.get_board_layout(false);
        let (card_width, _card_height) = crate::game::hand::get_card_sizes(
            canvas_width,
            canvas_height
        );
        if let Some(idx) = hovered_card_index(hand, pointer_xy, canvas_width, canvas_height) {
            // Start drag on just_pressed
            if pointer.left.just_pressed() {
                let card = hand[idx].clone();
                // Card::toggle_selection(&mut state.selected_cards, &card);
                // highlight_selected_card_tiles(state);
                let (card_x, card_y) = crate::game::hand::get_card_position(idx, card_width);
                let offset = (pointer_xy.0 - (card_x as i32), pointer_xy.1 - (card_y as i32));
                state.dragged_card = Some(crate::DraggedCard {
                    card: card.clone(),
                    hand_index: idx,
                    offset,
                    pos: (card_x as f32, card_y as f32),
                    velocity: (0.0, 0.0),
                    dragging: true,
                });
            }
        }
        // If dragging, update position while pressed
        if let Some(drag) = &mut state.dragged_card {
            if pointer.left.pressed() && drag.dragging {
                let new_x = (pointer_xy.0 - drag.offset.0) as f32;
                let new_y = (pointer_xy.1 - drag.offset.1) as f32;
                drag.pos = (new_x, new_y);
            }
            // On release, stop dragging and start spring-back
            if pointer.left.just_released() && drag.dragging {
                drag.dragging = false;
                // velocity will be used for spring-back in update
            }
        }
    }
}

pub fn handle_tile_selection(
    state: &mut GameState,
    pointer: &mouse::ScreenMouse,
    pointer_xy: (i32, i32)
) {
    // Only if a card is selected
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
