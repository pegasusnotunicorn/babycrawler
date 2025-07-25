use crate::game::cards::card::{ Card, CardVisualState };
use crate::game::constants::{ GAME_PADDING, HAND_SIZE, MAP_SIZE };
use turbo::*;
use crate::GameState;
use crate::game::cards::card_row::CardRow;
use crate::game::util::point_in_bounds;

pub fn get_card_sizes(canvas_width: u32, canvas_height: u32) -> (u32, u32) {
    let card_width = (canvas_width - GAME_PADDING * ((HAND_SIZE as u32) + 1)) / (HAND_SIZE as u32);
    let card_height = card_width.min(canvas_height / 5);
    (card_width, card_height)
}

pub fn get_hand_y() -> u32 {
    let canvas_bounds = bounds::screen();
    let canvas_width = canvas_bounds.w();
    let tile_size = canvas_width / (MAP_SIZE as u32);
    tile_size * (MAP_SIZE as u32) + GAME_PADDING
}

pub fn get_card_position(index: usize, card_width: u32) -> (u32, u32) {
    let x = GAME_PADDING + (index as u32) * (card_width + GAME_PADDING);
    let y = get_hand_y();
    (x, y)
}

pub fn hovered_card_index(
    hand: &[Card],
    mouse_xy: (i32, i32),
    canvas_width: u32,
    canvas_height: u32
) -> Option<usize> {
    if hand.is_empty() {
        return None;
    }
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let mx = mouse_xy.0;
    let my = mouse_xy.1;
    for (i, _card) in hand.iter().enumerate() {
        let (x, y) = get_card_position(i, card_width);
        let bounds = Bounds::new(x, y, card_width, card_height);
        if point_in_bounds(mx, my, &bounds) {
            return Some(i);
        }
    }
    None
}

pub fn draw_hand(state: &GameState, frame: f64) {
    let hand = &state.get_local_player().unwrap().hand;
    if hand.is_empty() {
        return;
    }
    let selected_card = state.selected_card.clone();
    let is_my_turn = state.is_my_turn();

    let pointer = mouse::screen();
    let pointer_xy = (pointer.x, pointer.y);
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let hovered = hovered_card_index(hand, pointer_xy, canvas_width, canvas_height);

    // Build CardRow for the hand
    let y = get_hand_y();
    let mut row = CardRow::new(hand, y, card_width, card_height);
    for (i, slot) in row.slots.iter_mut().enumerate() {
        let mut visual_state = CardVisualState::NONE;
        if
            is_my_turn &&
            hovered == Some(i) &&
            state.animated_card.is_none() &&
            selected_card.is_none() &&
            slot.card
                .as_ref()
                .map(|c| !c.is_dummy())
                .unwrap_or(false)
        {
            visual_state |= CardVisualState::HOVERED;
        }
        if let (Some(selected), Some(card)) = (selected_card.as_ref(), slot.card.as_ref()) {
            if selected == card {
                visual_state |= CardVisualState::SELECTED;
            }
        }
        slot.visual_state = visual_state;
    }
    row.draw(frame);

    // Draw the animated card on top, if any
    if let Some(drag) = &state.animated_card {
        let (dx, dy) = drag.pos;
        drag.card.draw(
            dx as u32,
            dy as u32,
            card_width,
            card_height,
            drag.card.color,
            true,
            CardVisualState::NONE,
            Some(frame)
        );
    }
}
