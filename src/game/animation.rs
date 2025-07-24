use crate::GameState;
use crate::game::cards::{ get_card_sizes, get_card_position };
use crate::game::map::tile::TileRotationAnim;

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

/// Starts a tile rotation animation for a given tile index.
pub fn start_tile_rotation_animation(
    state: &mut GameState,
    tile_index: usize,
    clockwise: bool,
    duration: f64
) {
    let tile = &mut state.tiles[tile_index];
    if tile.rotation_anim.is_some() {
        return; // Already animating
    }
    let from_angle = 0.0;
    let to_angle = if clockwise { 90.0 } else { -90.0 };
    tile.rotation_anim = Some(TileRotationAnim {
        from_angle,
        to_angle,
        current_angle: from_angle,
        duration,
        elapsed: 0.0,
        clockwise,
    });
}

/// Call this every frame to update all tile rotation animations.
pub fn update_tile_rotation_animations(state: &mut GameState, dt: f64) {
    for tile in &mut state.tiles {
        if let Some(anim) = &mut tile.rotation_anim {
            anim.elapsed += dt;
            let t = (anim.elapsed / anim.duration).min(1.0) as f32;
            anim.current_angle = anim.from_angle + (anim.to_angle - anim.from_angle) * t;
            if t >= 1.0 {
                // Complete the rotation
                if anim.clockwise {
                    tile.rotate_clockwise(1);
                } else {
                    tile.rotate_clockwise(3);
                }
                tile.rotation_anim = None;
            }
        }
    }
}
