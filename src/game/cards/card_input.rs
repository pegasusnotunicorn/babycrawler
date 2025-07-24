use crate::GameState;
use crate::game::cards::card::Card;
use crate::game::cards::CardRow;
use crate::game::constants::*;
use crate::game::cards::{ get_hand_y, get_card_sizes };
use crate::game::util::{
    rects_intersect_outline_to_inner,
    spring_back_card,
    get_card_button_geometry,
};
use turbo::*;
use crate::game::cards::card_effect::CardEffect;

pub fn handle_card_drag(
    state: &mut GameState,
    pointer: &mouse::ScreenMouse,
    pointer_xy: (i32, i32)
) {
    let selected_card = state.selected_card.clone();
    let play_area_cards = state.play_area.clone();
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let hand_cards = if let Some(player) = state.get_local_player() {
        player.hand.clone()
    } else {
        Vec::new()
    };
    let (w, h) = get_card_sizes(canvas_width, canvas_height);
    let card_width = w as i32;
    let card_height = h as i32;
    let hand_row = CardRow::new(
        &hand_cards,
        get_hand_y() as u32,
        card_width as u32,
        card_height as u32
    );
    let play_area_row = CardRow::new(
        &play_area_cards,
        ((get_hand_y() as i32) + card_height + (GAME_PADDING as i32)) as u32,
        card_width as u32,
        card_height as u32
    );
    let hand_slot_at_point = hand_row.slot_at_point(pointer_xy.0, pointer_xy.1);

    let mut dragged = state.dragged_card.take();
    if let Some(player) = state.get_local_player_mut() {
        let (w, h) = get_card_sizes(canvas_width, canvas_height);
        let card_width = w as i32;
        let card_height = h as i32;

        // Start drag from hand
        if selected_card.is_none() {
            if let Some(idx) = hand_slot_at_point {
                if pointer.left.just_pressed() {
                    let hand_has_card = player.hand.get(idx).is_some();
                    if hand_has_card {
                        let card = player.hand[idx].clone();
                        let (card_x, card_y) = hand_row.get_slot_position(idx);
                        let offset = (
                            pointer_xy.0 - (card_x as i32),
                            pointer_xy.1 - (card_y as i32),
                        );
                        dragged = Some(crate::DraggedCard {
                            card: card.clone(),
                            hand_index: idx,
                            offset,
                            pos: (card_x as f32, card_y as f32),
                            velocity: (0.0, 0.0),
                            dragging: true,
                            released: false,
                        });
                    }
                }
            }
        }

        // Drag update
        if let Some(drag) = &mut dragged {
            if pointer.left.pressed() && drag.dragging {
                let new_x = (pointer_xy.0 - drag.offset.0) as f32;
                let new_y = (pointer_xy.1 - drag.offset.1) as f32;
                drag.pos = (new_x, new_y);
            }

            // On release, check for valid drop
            if pointer.left.just_released() && drag.dragging {
                drag.dragging = false;
                drag.released = true;

                let hand_card_opt = player.hand.get(drag.hand_index).cloned();
                let from_hand = hand_card_opt
                    .as_ref()
                    .map(|c| c == &drag.card)
                    .unwrap_or(false);

                if from_hand {
                    let card = hand_card_opt.unwrap();
                    if let Some(target_idx) = play_area_row.leftmost_card_index(true) {
                        // Only allow drop if the dragged card's outline intersects the slot's inner rect
                        let (slot_x, slot_y) = play_area_row.get_slot_position(target_idx);
                        let border_width = GAME_PADDING;
                        if
                            rects_intersect_outline_to_inner(
                                slot_x,
                                slot_y,
                                card_width as u32,
                                card_height as u32,
                                drag.pos.0 as u32,
                                drag.pos.1 as u32,
                                card_width as u32,
                                card_height as u32,
                                border_width
                            )
                        {
                            // Remove from hand
                            player.hand[drag.hand_index] = Card::dummy_card();
                            // Insert into play_area
                            state.play_area.insert(target_idx, card.clone());
                            state.play_area.truncate(HAND_SIZE);
                            // Set selected_card to the dropped card (after all borrows)
                            state.selected_card = Some(card);
                            highlight_selected_card_tiles(state);
                            // Drop drag state
                            state.dragged_card = None;
                            return;
                        }
                    }
                } else {
                    if from_hand {
                        spring_back_card(
                            state,
                            drag.card.clone(),
                            drag.hand_index,
                            drag.pos.0 as u32,
                            drag.pos.1 as u32
                        );
                    }
                }
            }
        }
    }
    // Put drag state back
    state.dragged_card = dragged;
}

fn highlight_selected_card_tiles(state: &mut GameState) {
    let selected_card = state.selected_card.clone();
    // Highlight tiles for the newly selected card
    crate::game::map::tile::clear_highlights(&mut state.tiles);
    if let Some(card) = &selected_card {
        if let Some(player) = state.get_local_player() {
            card.effect.highlight_tiles(player.position, &mut state.tiles);
        }
    }
}

pub fn handle_play_area_buttons(state: &mut GameState, pointer: &mouse::ScreenMouse) {
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let hand_y = get_hand_y() as i32;
    let play_area_y = hand_y + (card_height as i32) + (GAME_PADDING as i32);
    let play_area_row = CardRow::new(
        &state.play_area,
        play_area_y as u32,
        card_width as u32,
        card_height as u32
    );
    let pointer_xy = (pointer.x, pointer.y);
    let just_pressed = pointer.left.just_pressed();
    for (i, slot) in play_area_row.slots.iter().enumerate() {
        let (x, y) = play_area_row.get_slot_position(i);
        let w = play_area_row.card_width;
        let h = play_area_row.card_height;
        let geom = get_card_button_geometry(y, w, h, GAME_PADDING);
        let pointer_bounds = turbo::Bounds::new(pointer_xy.0 as u32, pointer_xy.1 as u32, 1, 1);
        let show_buttons = {
            if let Some(card) = &slot.card {
                if !card.is_dummy() {
                    play_area_row.leftmost_card_index(false) == Some(i)
                } else {
                    false
                }
            } else {
                false
            }
        };
        if show_buttons {
            let button_specs = [
                ("B", 0xff2222ffu32, x + geom.inset + GAME_PADDING / 2),
                ("A", 0x22cc22ffu32, x + w - geom.inset - GAME_PADDING / 2 - geom.button_w),
            ];
            for (label, _color, bx) in button_specs {
                let bounds = turbo::Bounds::new(bx, geom.button_y, geom.button_w, geom.button_h);
                let hovered = bounds.contains(&pointer_bounds);
                if hovered && just_pressed {
                    if label == "B" {
                        if let Some(selected) = state.selected_card.take() {
                            // If the selected card is a rotate card, revert tile rotations
                            if let CardEffect::RotateCard = selected.effect {
                                CardEffect::revert_tile_rotations(&mut state.tiles);
                            }
                            if let Some(idx) = state.play_area.iter().position(|c| c == &selected) {
                                handle_card_cancel(state, idx, &selected);
                            }
                        }
                    } else if label == "A" {
                        // TODO: Implement button A
                    }
                }
            }
        }
    }
}

/// Moves a card from the play area to the first empty hand slot, updates state, and sets up spring-back animation.
pub fn handle_card_cancel(state: &mut GameState, play_area_idx: usize, selected: &Card) -> bool {
    // Compute all needed values before mutating state
    let play_area_cards = state.play_area.clone();
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let hand_y = get_hand_y() as i32;
    let play_area_y = hand_y + (card_height as i32) + (GAME_PADDING as i32);
    let play_area_row = CardRow::new(
        &play_area_cards,
        play_area_y as u32,
        card_width as u32,
        card_height as u32
    );
    let (from_x, from_y) = play_area_row.get_slot_position(play_area_idx);
    if let Some(player) = state.get_local_player_mut() {
        let empty_idx = player.hand.iter().position(|c| c.id == 0);
        if let Some(empty_idx) = empty_idx {
            player.hand[empty_idx] = selected.clone();
            if let Some(slot) = state.play_area.get_mut(play_area_idx) {
                *slot = Card::dummy_card();
            }
            spring_back_card(state, selected.clone(), empty_idx, from_x, from_y);
            highlight_selected_card_tiles(state);
            return true;
        }
    }
    false
}
