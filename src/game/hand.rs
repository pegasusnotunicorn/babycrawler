use crate::game::card::Card;
use crate::game::constants::{ GAME_PADDING, HAND_SIZE, MAP_SIZE };
use turbo::*;
use crate::GameState;

/// Returns (card_width, card_height) for the hand layout
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

/// Returns the (x, y) position for a card in the hand given its index and card size
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

    // Use the shared function for hover detection and layout
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let hovered = hovered_card_index(hand, pointer_xy, canvas_width, canvas_height);

    for (i, card) in hand.iter().enumerate() {
        let (x, y) = get_card_position(i, card_width);
        let is_hovered = hovered == Some(i);
        let is_selected = selected_cards.contains(card);

        card.draw_rect(
            x,
            y,
            card_width,
            card_height,
            card.color,
            true,
            is_hovered,
            is_selected,
            Some(frame)
        );

        // Card label
        let border_width = crate::game::constants::GAME_PADDING;
        let inset = border_width / 2;
        let label_x = x + inset + 4;
        let label_y = y + inset + 4;
        text!(&card.name, x = label_x, y = label_y, font = "large", color = 0x000000ff);
    }
}
