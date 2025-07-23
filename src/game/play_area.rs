use crate::game::hand::{ get_card_sizes, get_hand_y };
use crate::GameState;
use crate::game::card_row::CardRow;
use crate::game::constants::{
    GAME_PADDING,
    CARD_DUMMY_COLOR,
    CARD_BUTTON_A_COLOR,
    CARD_BUTTON_B_COLOR,
};
use crate::game::util::rects_intersect_outline_to_inner;
use turbo::mouse;
use crate::game::card::{ Card, CardVisualState };

fn draw_card_buttons(x: u32, y: u32, w: u32, h: u32, pointer_xy: (i32, i32)) {
    let border_width = GAME_PADDING;
    let inset = border_width / 2;
    let button_w = (w - GAME_PADDING * 3) / 2;
    let button_h = button_w;
    let button_y = y + h - inset - button_h;
    let pointer_bounds = turbo::Bounds::new(pointer_xy.0 as u32, pointer_xy.1 as u32, 1, 1);
    let button_specs = [
        ("B", CARD_BUTTON_A_COLOR, x + inset + GAME_PADDING / 2),
        ("A", CARD_BUTTON_B_COLOR, x + w - inset - GAME_PADDING / 2 - button_w),
    ];
    for (label, color, bx) in button_specs {
        let bounds = turbo::Bounds::new(bx, button_y, button_w, button_h);
        let hovered = bounds.contains(&pointer_bounds);
        Card::draw_button(label, bx, button_y, button_w, button_h, color, hovered);
    }
}

pub fn draw_play_area(state: &GameState, frame: f64) {
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    let y = get_hand_y() + card_height + GAME_PADDING;

    let mut play_area_row = CardRow::new(&state.play_area, y, card_width, card_height);

    // Highlight leftmost empty slot if dragging from hand
    if let Some(drag) = &state.dragged_card {
        if !drag.released {
            let dragging_from_hand = state.play_area
                .get(drag.hand_index)
                .map(|c| c != &drag.card)
                .unwrap_or(true);
            if dragging_from_hand {
                if let Some(idx) = play_area_row.leftmost_card_index(true) {
                    let (slot_x, slot_y) = play_area_row.get_slot_position(idx);
                    let border_width = GAME_PADDING;
                    if
                        rects_intersect_outline_to_inner(
                            slot_x,
                            slot_y,
                            card_width,
                            card_height,
                            drag.pos.0 as u32,
                            drag.pos.1 as u32,
                            card_width,
                            card_height,
                            border_width
                        )
                    {
                        play_area_row.slots[idx].visual_state |= CardVisualState::HOVERED;
                    }
                }
            }
        }
    }
    let pointer = mouse::screen();
    let pointer_xy = (pointer.x, pointer.y);

    // Draw play area row with buttons on the leftmost real card
    for (i, slot) in play_area_row.slots.iter().enumerate() {
        let (x, y) = play_area_row.get_slot_position(i);
        let outline_color = slot.card
            .as_ref()
            .map(|c| c.color)
            .unwrap_or(CARD_DUMMY_COLOR);
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
        if let Some(card) = &slot.card {
            let mut visual_state = slot.visual_state;
            if card.id == 0 {
                visual_state |= CardVisualState::DUMMY;
            }
            if show_buttons {
                let w = play_area_row.card_width;
                let h = play_area_row.card_height;
                card.draw(x, y, w, h, outline_color, true, visual_state, Some(frame));
                draw_card_buttons(x, y, w, h, pointer_xy);
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
