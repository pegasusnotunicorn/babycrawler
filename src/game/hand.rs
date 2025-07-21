use crate::game::card::Card;
use crate::game::constants::FLASH_SPEED;
use turbo::*;
use crate::GameState;

/// Returns (card_width, card_height, y, offset_x) for the hand layout
pub fn hand_card_layout(
    hand_len: usize,
    canvas_width: u32,
    canvas_height: u32
) -> (u32, u32, u32, u32) {
    use crate::game::constants::GAME_PADDING;
    let card_width = canvas_width / (hand_len as u32);
    let card_height = card_width.min(canvas_height / 5);
    let y = canvas_height - card_height - GAME_PADDING;
    let offset_x = GAME_PADDING;
    (card_width, card_height, y, offset_x)
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
    let (card_width, card_height, y, offset_x) = hand_card_layout(
        hand.len(),
        canvas_width,
        canvas_height
    );
    let mx = mouse_xy.0;
    let my = mouse_xy.1;
    for (i, _card) in hand.iter().enumerate() {
        let x = (i as u32) * card_width + offset_x;
        let bounds = Bounds::new(x, y, card_width, card_height);
        if crate::game::util::point_in_bounds(mx, my, &bounds) {
            return Some(i);
        }
    }
    None
}

pub fn draw_hand(
    state: &GameState,
    hand: &[Card],
    selected_cards: &[Card],
    tile_size: u32,
    frame: f64
) {
    if hand.is_empty() {
        return;
    }

    let pointer = mouse::screen();
    let pointer_xy = (pointer.x, pointer.y);
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(true);

    // Use the shared function for hover detection and layout
    let hovered = hovered_card_index(hand, pointer_xy, canvas_width, canvas_height);
    let (card_width, card_height, y, offset_x) = hand_card_layout(
        hand.len(),
        canvas_width,
        canvas_height
    );

    let border_width = tile_size / 4;
    let border_radius = tile_size / 16;
    let inset = border_width / 2;

    for (i, card) in hand.iter().enumerate() {
        let x = (i as u32) * card_width + offset_x;
        let is_hovered = hovered == Some(i);
        let is_selected = selected_cards.contains(card);

        // Inset dimensions to make room for overlay-style border
        let inner_x = x + inset;
        let inner_y = y + inset;
        let inner_w = card_width.saturating_sub(border_width);
        let inner_h = card_height.saturating_sub(border_width);

        if is_selected {
            let t: f64 = (frame * FLASH_SPEED).sin() * 0.5 + 0.5;
            let alpha = (t * 255.0) as u32;
            let flash_color: u32 = (0xff << 24) | (0xff << 16) | (0xff << 8) | alpha;

            rect!(
                x = x,
                y = y,
                w = card_width,
                h = card_height,
                color = flash_color,
                border_radius = border_radius
            );
        }

        // Background fill
        rect!(
            x = inner_x,
            y = inner_y,
            w = inner_w,
            h = inner_h,
            color = card.color,
            border_radius = border_radius
        );

        // Hover overlay
        if is_hovered {
            let highlight_color = 0xffffff80;
            rect!(
                x = inner_x,
                y = inner_y,
                w = inner_w,
                h = inner_h,
                color = highlight_color,
                border_radius = border_radius
            );
        }

        // Card label
        text!(&card.name, x = inner_x + 4, y = inner_y + 4, font = "large", color = 0x000000ff);
    }
}
