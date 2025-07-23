use crate::game::constants::{ HAND_SIZE, PLAY_AREA_COLOR, GAME_PADDING };
use crate::game::hand::{ get_card_sizes, get_hand_y };
use crate::game::card::Card;
use crate::game::card_effect::CardEffect;
use crate::GameState;
use crate::game::card::CardVisualState;

pub fn draw_play_area(state: &GameState) {
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let y = get_hand_y() + card_height + GAME_PADDING;

    // Determine which play area card is hovered by the dragged card (if any)
    let mut hovered_index: Option<usize> = None;
    let mut max_intersection = 0;
    if let Some(drag) = &state.dragged_card {
        let drag_bounds = turbo::Bounds::new(
            drag.pos.0 as u32,
            drag.pos.1 as u32,
            card_width,
            card_height
        );
        for i in 0..HAND_SIZE {
            let x = GAME_PADDING + (i as u32) * (card_width + GAME_PADDING);
            let card_bounds = turbo::Bounds::new(x, y, card_width, card_height);
            // Compute intersection area
            let ix0 = drag_bounds.x().max(card_bounds.x());
            let iy0 = drag_bounds.y().max(card_bounds.y());
            let ix1 = (drag_bounds.x() + (drag_bounds.w() as i32)).min(
                card_bounds.x() + (card_bounds.w() as i32)
            );
            let iy1 = (drag_bounds.y() + (drag_bounds.h() as i32)).min(
                card_bounds.y() + (card_bounds.h() as i32)
            );
            let iw = (ix1 - ix0).max(0);
            let ih = (iy1 - iy0).max(0);
            let area = iw * ih;
            if area > max_intersection {
                max_intersection = area;
                hovered_index = Some(i);
            }
        }
    }

    for i in 0..HAND_SIZE {
        let x = GAME_PADDING + (i as u32) * (card_width + GAME_PADDING);
        // Create a dummy Card just for drawing the rectangle with the correct color
        let dummy_card = Card {
            id: 0,
            name: String::new(),
            effect: CardEffect::MoveOneTile,
            color: PLAY_AREA_COLOR,
        };
        let mut visual_state = CardVisualState::DUMMY;
        if Some(i) == hovered_index {
            visual_state |= CardVisualState::HOVERED;
        }
        dummy_card.draw(x, y, card_width, card_height, PLAY_AREA_COLOR, true, visual_state, None);
    }
}
