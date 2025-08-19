use crate::GameState;
use crate::game::cards::card::Card;
use crate::game::cards::card_buttons::CardButton;
use crate::game::cards::CardRow;
use crate::game::constants::*;
use crate::game::cards::{ get_hand_y, get_card_sizes };
use crate::game::util::rects_intersect_outline_to_inner;
use turbo::*;
use crate::game::cards::card_effect::CardEffect;
use crate::game::animation::{ highlight_selected_card_tiles, AnimatedCardOrigin, AnimatedCard };
use crate::network::send::{ send_card_cancel, send_confirm_card };
use crate::game::map::clear_highlights;
use crate::game::cards::card_buttons::should_show_buttons;

// #region Helper functions

fn get_hand_slot_pos(hand_row: &CardRow, idx: usize) -> (f32, f32) {
    let (x, y) = hand_row.get_slot_position(idx);
    (x as f32, y as f32)
}

fn get_play_area_slot_pos(play_area_row: &CardRow, idx: usize) -> (f32, f32) {
    let (x, y) = play_area_row.get_slot_position(idx);
    (x as f32, y as f32)
}

fn play_area_intersect(
    play_area_row: &CardRow,
    card_width: u32,
    card_height: u32,
    pos: (f32, f32)
) -> Option<usize> {
    if let Some(idx) = play_area_row.leftmost_card_index(true) {
        let (slot_x, slot_y) = play_area_row.get_slot_position(idx);
        let border_width = GAME_PADDING;
        if
            rects_intersect_outline_to_inner(
                slot_x,
                slot_y,
                card_width,
                card_height,
                pos.0 as u32,
                pos.1 as u32,
                card_width,
                card_height,
                border_width
            )
        {
            return Some(idx);
        }
    }
    None
}

fn get_hand_row(state: &GameState) -> CardRow {
    let (canvas_width, canvas_height, _, _, _) = state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let hand_cards = state
        .get_local_player()
        .map(|p| p.hand.clone())
        .unwrap_or_default();
    CardRow::new(&hand_cards, get_hand_y() as u32, card_width, card_height)
}

fn get_play_area_row(state: &GameState) -> CardRow {
    let (canvas_width, canvas_height, _, _, _) = state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let hand_y = get_hand_y() as i32;
    let play_area_y = hand_y + (card_height as i32) + (GAME_PADDING as i32);
    CardRow::new(&state.play_area, play_area_y as u32, card_width, card_height)
}

// #endregion

pub fn handle_card_drag(
    state: &mut GameState,
    pointer: &mouse::ScreenMouse,
    pointer_xy: (i32, i32)
) {
    let selected_card = state.selected_card.clone();
    let hand_row = get_hand_row(state);
    let hand_slot_at_point = hand_row.slot_at_point(pointer_xy.0, pointer_xy.1);
    let play_area_row = get_play_area_row(state);

    let mut dragged = state.animated_card.take();
    if let Some(player) = state.get_local_player_mut() {
        // Only allow a new drag if no card is being dragged or animating
        let can_start_drag = dragged.as_ref().map_or(true, |d| !d.dragging && !d.animating);
        // Start drag from hand
        if selected_card.is_none() && can_start_drag {
            if let Some(idx) = hand_slot_at_point {
                if pointer.left.just_pressed() {
                    let hand_has_card = player.hand.get(idx).is_some();
                    if hand_has_card {
                        let card = player.hand[idx].clone();
                        if card.is_dummy() {
                            return; // Don't allow dragging dummy cards
                        }
                        player.hand[idx] = Card::dummy_card();
                        let origin_pos = get_hand_slot_pos(&hand_row, idx);
                        dragged = Some(AnimatedCard {
                            card: card.clone(),
                            pos: origin_pos,
                            velocity: (0.0, 0.0),
                            origin_row: AnimatedCardOrigin::Hand,
                            origin_row_index: idx,
                            origin_pos,
                            target_row: AnimatedCardOrigin::Hand,
                            target_row_index: idx,
                            target_pos: origin_pos,
                            dragging: true,
                            animating: false,
                        });
                    }
                }
            }
        }

        // Drag update
        if let Some(drag) = &mut dragged {
            if drag.dragging && pointer.left.pressed() && !drag.animating {
                let new_x = (pointer_xy.0 - ((hand_row.card_width / 2) as i32)) as f32;
                let new_y = (pointer_xy.1 - ((hand_row.card_height / 2) as i32)) as f32;
                drag.pos = (new_x, new_y);
                drag.target_pos = (new_x, new_y);
            }

            // On release, check for valid drop
            if drag.dragging && pointer.left.just_released() {
                drag.dragging = false;
                // Check if released over play area
                let w = play_area_row.card_width;
                let h = play_area_row.card_height;
                if let Some(idx) = play_area_intersect(&play_area_row, w, h, drag.pos) {
                    let target_pos = get_play_area_slot_pos(&play_area_row, idx);
                    drag.target_pos = target_pos;
                    drag.target_row = AnimatedCardOrigin::PlayArea;
                    drag.target_row_index = idx;
                    drag.animating = true;
                } else {
                    // Animate back to hand slot (original index)
                    let target_pos = get_hand_slot_pos(&hand_row, drag.origin_row_index);
                    drag.target_pos = target_pos;
                    drag.target_row = AnimatedCardOrigin::Hand;
                    drag.target_row_index = drag.origin_row_index;
                    drag.animating = true;
                }
            }
        }
    }
    // Put drag state back
    state.animated_card = dragged;
}

pub fn handle_play_area_buttons(state: &mut GameState, pointer: &mouse::ScreenMouse) {
    let play_area_row = get_play_area_row(state);
    let pointer_xy = (pointer.x, pointer.y);
    let just_pressed = pointer.left.just_pressed();

    for (i, slot) in play_area_row.slots.iter().enumerate() {
        let (x, y) = play_area_row.get_slot_position(i);
        let w = play_area_row.card_width;
        let h = play_area_row.card_height;
        let geometry = CardButton::new(y, w, h);
        let button_positions = geometry.get_button_positions(x, w);
        let pointer_bounds = turbo::Bounds::new(pointer_xy.0 as u32, pointer_xy.1 as u32, 1, 1);

        let show_buttons = if let Some(card) = &slot.card {
            should_show_buttons(card, state.selected_card.as_ref())
        } else {
            false
        };

        if show_buttons {
            let button_specs = [
                ("B", button_positions[0]),
                ("A", button_positions[1]),
            ];

            for (label, (bx, by)) in button_specs {
                let bounds = turbo::Bounds::new(bx, by, geometry.button_w, geometry.button_h);
                let hovered = bounds.contains(&pointer_bounds);
                if hovered && just_pressed {
                    if label == "B" {
                        if let Some(selected) = state.selected_card.take() {
                            if let Some(idx) = state.play_area.iter().position(|c| c == &selected) {
                                handle_card_cancel(state, idx, &selected);
                            }
                        }
                    } else if label == "A" && !slot.card.as_ref().unwrap().hide_confirm_button {
                        confirm_card(state);
                    }
                }
            }
        }
    }
}

/// Moves a card from the play area back to its original hand slot, updates state, and sets up spring-back animation.
pub fn handle_card_cancel(state: &mut GameState, play_area_idx: usize, selected: &Card) {
    log!("🔍 [CANCEL] Card: {:?}, hand_index: {:?}", selected.name, selected.hand_index);

    if let CardEffect::RotateCard = selected.effect {
        CardEffect::revert_tile_rotations(&mut state.tiles);
    }

    if let CardEffect::SwapCard = selected.effect {
        CardEffect::revert_tile_positions(state);
    }

    let play_area_row = get_play_area_row(state);
    let (from_x, from_y) = play_area_row.get_slot_position(play_area_idx);

    // Find the original hand slot where this card came from
    if let Some(original_hand_idx) = selected.hand_index {
        if let Some(player) = state.get_local_player_mut() {
            if original_hand_idx < player.hand.len() {
                send_card_cancel(original_hand_idx);

                // Clear the play area slot first
                state.play_area[play_area_idx] = Card::dummy_card();

                let hand_row = get_hand_row(state);
                let (to_x, to_y) = hand_row.get_slot_position(original_hand_idx);

                // Set up AnimatedCard for spring-back animation to the original hand slot
                state.animated_card = Some(AnimatedCard {
                    card: selected.clone(),
                    pos: (from_x as f32, from_y as f32),
                    velocity: (0.0, 0.0),
                    origin_row: AnimatedCardOrigin::PlayArea,
                    origin_row_index: play_area_idx,
                    origin_pos: (from_x as f32, from_y as f32),
                    target_row: AnimatedCardOrigin::Hand,
                    target_row_index: original_hand_idx,
                    target_pos: (to_x as f32, to_y as f32),
                    dragging: false,
                    animating: true,
                });
                highlight_selected_card_tiles(state);
            }
        }
    }
}

pub fn confirm_card(state: &mut GameState) {
    if let Some(selected_card) = &state.selected_card {
        send_confirm_card(selected_card);
        state.selected_card = None;
        clear_highlights(&mut state.tiles);
    }
}
