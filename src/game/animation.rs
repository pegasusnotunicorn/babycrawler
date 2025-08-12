use turbo::*;
use crate::GameState;
use crate::game::map::tile::{ TileRotationAnim, Tile, clear_highlights };
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

#[derive(
    Clone,
    Debug,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub struct AnimatedPlayer {
    pub player_id: String,
    pub pos: (f32, f32), // current screen position
    pub velocity: (f32, f32), // current velocity
    pub origin_pos: (usize, usize), // starting tile position
    pub target_pos: (usize, usize), // target tile position
    pub path: Vec<usize>, // path of tile indices to follow
    pub current_path_index: usize, // current position in the path
    pub animating: bool,
}

pub fn update_animations(state: &mut GameState) {
    if update_animated_card_spring(state) {
        handle_animated_card_complete(state);
    }
    update_tile_rotation_animations(state, 1.0 / 60.0);
    update_player_movement_animations(state);
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
            send_card_cancel(drag.target_row_index, &drag.card);
        }
    }
    state.animated_card = None;
}

// Highlight tiles for the newly selected card
pub fn highlight_selected_card_tiles(state: &mut GameState) {
    let selected_card = state.selected_card.clone();
    clear_highlights(&mut state.tiles);
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

/// Start a player movement animation
pub fn start_player_movement_animation(
    state: &mut GameState,
    player_id: &str,
    from_pos: (usize, usize),
    to_pos: (usize, usize),
    tile_size: u32,
    offset_x: u32,
    offset_y: u32
) {
    // Calculate screen positions
    let from_screen_x = offset_x + (from_pos.0 as u32) * tile_size + tile_size / 2;
    let from_screen_y = offset_y + (from_pos.1 as u32) * tile_size + tile_size / 2;

    // Find the path from start to target
    let start_index = Tile::index(from_pos.0, from_pos.1);
    let target_index = Tile::index(to_pos.0, to_pos.1);
    let path = Tile::find_path(start_index, target_index, &state.tiles).unwrap_or_else(||
        vec![start_index, target_index]
    );

    state.animated_player = Some(AnimatedPlayer {
        player_id: player_id.to_string(),
        pos: (from_screen_x as f32, from_screen_y as f32),
        velocity: (0.0, 0.0),
        origin_pos: from_pos,
        target_pos: to_pos,
        path,
        current_path_index: 0,
        animating: true,
    });
}

/// Start a direct A-to-B player movement animation (for canceled movements)
pub fn start_direct_player_movement_animation(
    state: &mut GameState,
    player_id: &str,
    from_pos: (usize, usize),
    to_pos: (usize, usize),
    tile_size: u32,
    offset_x: u32,
    offset_y: u32
) {
    // Calculate screen positions
    let from_screen_x = offset_x + (from_pos.0 as u32) * tile_size + tile_size / 2;
    let from_screen_y = offset_y + (from_pos.1 as u32) * tile_size + tile_size / 2;

    // Use direct path (no pathfinding)
    let start_index = Tile::index(from_pos.0, from_pos.1);
    let target_index = Tile::index(to_pos.0, to_pos.1);
    let path = vec![start_index, target_index];

    state.animated_player = Some(AnimatedPlayer {
        player_id: player_id.to_string(),
        pos: (from_screen_x as f32, from_screen_y as f32),
        velocity: (0.0, 0.0),
        origin_pos: from_pos,
        target_pos: to_pos,
        path,
        current_path_index: 0,
        animating: true,
    });
}

/// Update player movement animations
pub fn update_player_movement_animations(state: &mut GameState) {
    // Get board layout before mutable borrow
    let (_, _, tile_size, offset_x, offset_y) = state.get_board_layout(false);

    if let Some(anim) = &mut state.animated_player {
        if anim.animating {
            // Get current target from path
            if anim.current_path_index < anim.path.len() {
                let current_target_index = anim.path[anim.current_path_index];
                let (target_x, target_y) = Tile::position(current_target_index);
                let target_screen_x = offset_x + (target_x as u32) * tile_size + tile_size / 2;
                let target_screen_y = offset_y + (target_y as u32) * tile_size + tile_size / 2;
                let target_pos = (target_screen_x as f32, target_screen_y as f32);

                let (new_pos, new_velocity, snapped) = spring_to_target(
                    anim.pos,
                    anim.velocity,
                    target_pos,
                    0.5, // spring - very fast
                    0.0, // friction - almost no bounce at all
                    1.0, // snap_distance - snap very soon
                    0.1 // snap_velocity - snap very easily
                );

                anim.pos = new_pos;
                anim.velocity = new_velocity;

                if snapped {
                    // Move to next waypoint in path
                    anim.current_path_index += 1;

                    // If we've reached the end of the path, complete the animation
                    if anim.current_path_index >= anim.path.len() {
                        // Animation complete - update the actual player position
                        let player_id = anim.player_id.clone();
                        let target_pos = anim.target_pos;
                        state.animated_player = None;

                        // Update player position after clearing the animation
                        if let Some(player) = state.get_player_by_user_id(&player_id) {
                            player.position = target_pos;
                        }
                    }
                }
            }
        }
    }
}
