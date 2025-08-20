use crate::game::cards::hand::{ get_card_sizes, get_hand_y };
use crate::GameState;
use crate::game::cards::card_row::CardRow;
use crate::game::constants::{ GAME_PADDING, CARD_DUMMY_COLOR };

use turbo::mouse;
use crate::game::cards::card::{ Card, CardVisualState };
use crate::game::cards::card_buttons::{ draw_card_buttons, should_show_buttons };

pub fn fill_with_dummies(vec: &mut Vec<Card>, size: usize) {
    while vec.len() < size {
        vec.push(Card::dummy_card());
    }
    vec.truncate(size);
}

pub fn draw_play_area(state: &GameState, frame: f64) {
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let y = get_hand_y() + card_height + GAME_PADDING;

    let play_area_row = CardRow::new(&state.play_area, y, card_width, card_height);

    let pointer = mouse::screen();
    let pointer_xy = (pointer.x, pointer.y);

    // Draw play area row with buttons on the leftmost real card
    for (i, slot) in play_area_row.slots.iter().enumerate() {
        let (x, y) = play_area_row.get_slot_position(i);
        let outline_color = slot.card
            .as_ref()
            .map(|c| c.color)
            .unwrap_or(CARD_DUMMY_COLOR);
        let show_buttons = if let Some(card) = &slot.card {
            should_show_buttons(card, state.selected_card.as_ref())
        } else {
            false
        };

        if let Some(card) = &slot.card {
            let mut visual_state = slot.visual_state;
            if card.id == 0 {
                visual_state |= CardVisualState::DUMMY;
            }
            if show_buttons {
                let w = play_area_row.card_width;
                let h = play_area_row.card_height;
                visual_state |= CardVisualState::SELECTED;
                card.draw(x, y, w, h, outline_color, true, visual_state, Some(frame));
                draw_card_buttons(x, y, w, h, pointer_xy, card.hide_confirm_button);
            } else {
                card.draw(
                    x,
                    y,
                    play_area_row.card_width,
                    play_area_row.card_height,
                    outline_color,
                    true,
                    visual_state,
                    Some(frame)
                );
            }
        } else {
            let dummy = Card::dummy_card();
            dummy.draw(
                x,
                y,
                play_area_row.card_width,
                play_area_row.card_height,
                outline_color,
                true,
                slot.visual_state | CardVisualState::DUMMY,
                Some(frame)
            );
        }
    }
}
