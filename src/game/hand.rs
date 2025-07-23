use crate::game::card::Card;
use crate::game::constants::{ GAME_PADDING, HAND_SIZE, MAP_SIZE };
use turbo::*;
use crate::GameState;
use crate::game::card::CardVisualState;

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
        if crate::game::util::point_in_bounds(mx, my, &bounds) {
            return Some(i);
        }
    }
    None
}

pub fn draw_hand(state: &GameState, hand: &[Card], selected_cards: &[Card], frame: f64) {
    if hand.is_empty() {
        return;
    }

    let pointer = mouse::screen();
    let pointer_xy = (pointer.x, pointer.y);
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let hovered = hovered_card_index(hand, pointer_xy, canvas_width, canvas_height);

    // If dragging, get dragged card info
    let (dragged_index, dragged_card, dragged_pos) = if let Some(drag) = &state.dragged_card {
        (Some(drag.hand_index), Some(&drag.card), Some(drag.pos))
    } else {
        (None, None, None)
    };

    for (i, card) in hand.iter().enumerate() {
        let (x, y) = get_card_position(i, card_width);
        let is_hovered = hovered == Some(i);
        let is_selected = selected_cards.contains(card);

        let mut visual_state = CardVisualState::NONE;
        if is_hovered && !dragged_index.is_some() {
            visual_state |= CardVisualState::HOVERED;
        }
        if is_selected {
            visual_state |= CardVisualState::SELECTED;
        }
        if Some(i) == dragged_index {
            visual_state |= CardVisualState::DUMMY;
        }
        card.draw(x, y, card_width, card_height, card.color, true, visual_state, Some(frame));
    }

    // Draw the dragged card
    if let (Some(card), Some((dx, dy))) = (dragged_card, dragged_pos) {
        card.draw(
            dx as u32,
            dy as u32,
            card_width,
            card_height,
            card.color,
            true,
            CardVisualState::NONE,
            Some(frame)
        );
    }
}
