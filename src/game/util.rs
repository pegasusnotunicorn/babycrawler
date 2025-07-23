use turbo::Bounds;
use crate::game::cards::Card;
use crate::game::cards::CardRow;
use crate::game::cards::{ get_hand_y, get_card_sizes };
use crate::game::constants::GAME_PADDING;

pub fn point_in_bounds(x: i32, y: i32, bounds: &Bounds) -> bool {
    let bx = bounds.x();
    let by = bounds.y();
    let bw = bounds.w() as i32;
    let bh = bounds.h() as i32;

    x >= bx && x < bx + bw && y >= by && y < by + bh
}

pub fn lerp_color(from: u32, to: u32, t: f64) -> u32 {
    let from_r = ((from >> 24) & 0xff) as f64;
    let from_g = ((from >> 16) & 0xff) as f64;
    let from_b = ((from >> 8) & 0xff) as f64;
    let from_a = (from & 0xff) as f64;

    let to_r = ((to >> 24) & 0xff) as f64;
    let to_g = ((to >> 16) & 0xff) as f64;
    let to_b = ((to >> 8) & 0xff) as f64;
    let to_a = (to & 0xff) as f64;

    let r = ((1.0 - t) * from_r + t * to_r) as u32;
    let g = ((1.0 - t) * from_g + t * to_g) as u32;
    let b = ((1.0 - t) * from_b + t * to_b) as u32;
    let a = ((1.0 - t) * from_a + t * to_a) as u32;

    (r << 24) | (g << 16) | (b << 8) | a
}

/// Returns true if the drag card's outline rect intersects the slot's inner rect
pub fn rects_intersect_outline_to_inner(
    slot_x: u32,
    slot_y: u32,
    slot_w: u32,
    slot_h: u32,
    drag_x: u32,
    drag_y: u32,
    drag_w: u32,
    drag_h: u32,
    border_width: u32
) -> bool {
    let inset = border_width / 2;
    let slot_inner_x = slot_x + inset;
    let slot_inner_y = slot_y + inset;
    let slot_inner_w = slot_w.saturating_sub(border_width);
    let slot_inner_h = slot_h.saturating_sub(border_width);
    let slot_bounds = turbo::Bounds::new(slot_inner_x, slot_inner_y, slot_inner_w, slot_inner_h);
    let drag_bounds = turbo::Bounds::new(drag_x, drag_y, drag_w, drag_h);
    slot_bounds.intersects(&drag_bounds)
}

/// Sets up a spring-back animation for a card from (from_x, from_y) to the hand slot at (to_x, to_y).
pub fn spring_back_card(
    state: &mut crate::GameState,
    card: Card,
    hand_index: usize,
    from_x: u32,
    from_y: u32
) {
    state.dragged_card = Some(crate::DraggedCard {
        card,
        hand_index,
        offset: (0, 0),
        pos: (from_x as f32, from_y as f32),
        velocity: (0.0, 0.0),
        dragging: false,
        released: true,
    });
}

/// Moves a card from the play area to the first empty hand slot, updates state, and sets up spring-back animation.
pub fn move_card_play_area_to_hand_with_spring_back(
    state: &mut crate::GameState,
    play_area_idx: usize,
    selected: &crate::game::card::Card
) -> bool {
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
            return true;
        }
    }
    false
}

pub struct CardButtonGeometry {
    pub button_w: u32,
    pub button_h: u32,
    pub button_y: u32,
    pub inset: u32,
}

pub fn get_card_button_geometry(y: u32, w: u32, h: u32, padding: u32) -> CardButtonGeometry {
    let inset = padding / 2;
    let button_w = (w - padding * 3) / 2;
    let button_h = button_w;
    let button_y = y + h - inset - button_h;
    CardButtonGeometry { button_w, button_h, button_y, inset }
}
