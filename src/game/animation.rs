use crate::GameState;
use crate::game::map::tile::TileRotationAnim;
use crate::game::map::tile_effects::highlight_tiles_for_effect;
use crate::network::send::{ send_card_selection, send_card_cancel };
use crate::game::cards::card::Card;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub enum AnimatedCardOrigin {
    Hand,
    PlayArea,
    Other,
}

#[derive(
    Clone,
    Debug,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub struct AnimatedCard {
    pub card: Card,
    pub pos: (f32, f32), // current position
    pub velocity: (f32, f32), // current velocity
    pub origin_row: AnimatedCardOrigin,
    pub origin_row_index: usize,
    pub origin_pos: (f32, f32), // where the card started animating from
    pub target_row: AnimatedCardOrigin,
    pub target_row_index: usize,
    pub target_pos: (f32, f32), // where the card is animating to
    pub dragging: bool,
    pub animating: bool,
}

pub fn update_animations(state: &mut GameState) {
    if update_animated_card_spring(state) {
        handle_animated_card_complete(state);
    }
    update_tile_rotation_animations(state, 1.0 / 60.0);
}

/// Generic spring-to-target function for 2D positions.
pub fn spring_to_target(
    pos: (f32, f32),
    velocity: (f32, f32),
    target: (f32, f32),
    spring: f32,
    friction: f32,
    snap_distance: f32,
    snap_velocity: f32
) -> ((f32, f32), (f32, f32), bool) {
    let (px, py) = pos;
    let (vx, vy) = velocity;
    let (target_x, target_y) = target;
    let dx = target_x - px;
    let dy = target_y - py;
    let new_vx = vx * friction + dx * spring;
    let new_vy = vy * friction + dy * spring;
    let new_px = px + new_vx;
    let new_py = py + new_vy;
    let snapped =
        dx.abs() < snap_distance &&
        dy.abs() < snap_distance &&
        new_vx.abs() < snap_velocity &&
        new_vy.abs() < snap_velocity;
    let final_pos = if snapped { (target_x, target_y) } else { (new_px, new_py) };
    let final_velocity = (new_vx, new_vy);
    (final_pos, final_velocity, snapped)
}

/// Updates the spring animation for the animated card. Returns true if the animation is complete.
pub fn update_animated_card_spring(state: &mut GameState) -> bool {
    let mut clear_animated = false;
    if let Some(anim) = &mut state.animated_card {
        if anim.animating && !anim.dragging {
            let (new_pos, new_velocity, snapped) = spring_to_target(
                anim.pos,
                anim.velocity,
                anim.target_pos,
                0.2, // spring
                0.6, // friction
                1.0, // snap_distance
                0.5 // snap_velocity
            );
            anim.pos = new_pos;
            anim.velocity = new_velocity;
            if snapped {
                clear_animated = true;
            }
        }
    }
    clear_animated
}

pub fn handle_animated_card_complete(state: &mut crate::GameState) {
    let animated = state.animated_card.as_ref().cloned();
    if let Some(drag) = animated {
        if drag.target_row == AnimatedCardOrigin::PlayArea {
            send_card_selection(drag.target_row_index, &drag.card);
        } else if drag.target_row == AnimatedCardOrigin::Hand {
            send_card_cancel(drag.target_row_index);
        }
    }
    state.animated_card = None;
}

// Highlight tiles for the newly selected card
pub fn highlight_selected_card_tiles(state: &mut GameState) {
    let selected_card = state.selected_card.clone();
    crate::game::map::tile::clear_highlights(&mut state.tiles);
    if let Some(card) = &selected_card {
        if let Some(player) = state.get_local_player() {
            highlight_tiles_for_effect(&card.effect, player.position, &mut state.tiles);
        }
    }
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
