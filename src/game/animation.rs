use crate::GameState;
use crate::game::hand::get_card_sizes;
use crate::game::hand::get_card_position;

/// Updates the spring-back animation for the dragged card. Returns true if the dragged card should be cleared.
pub fn update_spring_back_dragged_card(state: &mut GameState) -> bool {
    let mut clear_dragged = false;
    if let Some(drag) = &mut state.dragged_card {
        if !drag.dragging {
            let hand_index = drag.hand_index;
            let (px, py) = drag.pos;
            let (vx, vy) = drag.velocity;
            let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
                state.get_board_layout(false);
            let (card_width, _card_height) = get_card_sizes(canvas_width, canvas_height);
            let (target_x, target_y) = get_card_position(hand_index, card_width);
            let dx = (target_x as f32) - px;
            let dy = (target_y as f32) - py;
            // Spring physics
            let spring = 0.2;
            let friction = 0.7;
            let new_vx = vx * friction + dx * spring;
            let new_vy = vy * friction + dy * spring;
            let new_px = px + new_vx;
            let new_py = py + new_vy;
            if let Some(drag) = &mut state.dragged_card {
                drag.velocity.0 = new_vx;
                drag.velocity.1 = new_vy;
                drag.pos.0 = new_px;
                drag.pos.1 = new_py;
                // Snap to slot if close enough
                if dx.abs() < 1.0 && dy.abs() < 1.0 && new_vx.abs() < 0.5 && new_vy.abs() < 0.5 {
                    drag.pos = (target_x as f32, target_y as f32);
                    clear_dragged = true;
                }
            }
        }
    }
    clear_dragged
}
