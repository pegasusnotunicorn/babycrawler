use turbo::*;
use crate::GameState;
use crate::game::map::tile::{ TileRotationAnim, Tile, clear_highlights };
use crate::game::map::tile_effects::highlight_tiles_for_effect;
use crate::network::send::send_card_selection;
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

#[derive(
    Clone,
    Debug,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub struct AnimatedTile {
    pub tile_index: usize, // which tile is being animated
    pub pos: (f32, f32), // current screen position
    pub velocity: (f32, f32), // current velocity
    pub target_index: usize, // target tile index
    pub animating: bool,
}

pub fn update_animations(state: &mut GameState) {
    if update_animated_card_spring(state) {
        handle_animated_card_complete(state);
    }
    update_tile_rotation_animations(state, 1.0 / 60.0);
    update_player_movement_animations(state);
    update_tile_animations(state);
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
    log!("🎮 [ANIMATION] handle_animated_card_complete");
    let animated = state.animated_card.as_ref().cloned();
    if let Some(drag) = animated {
        if drag.target_row == AnimatedCardOrigin::PlayArea {
            // Immediately update the play area for instant feedback
            state.play_area[drag.target_row_index] = drag.card.clone();
            state.selected_card = Some(drag.card.clone());
            highlight_selected_card_tiles(state);
            if let Some(hand_index) = drag.card.hand_index {
                send_card_selection(hand_index);
            }
        } else if drag.target_row == AnimatedCardOrigin::Hand {
            // Card is returning to hand from play area - restore it to the hand
            if let Some(player) = state.get_local_player_mut() {
                if drag.target_row_index < player.hand.len() {
                    player.hand[drag.target_row_index] = drag.card.clone();
                }
            }
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
/// If target_rotation is provided, rotates to that specific rotation.
/// If clockwise is provided, rotates by 90 degrees in that direction.
pub fn start_tile_rotation_animation(
    state: &mut GameState,
    tile_index: usize,
    target_rotation: Option<u8>,
    duration: f64
) {
    let tile = &mut state.tiles[tile_index];
    if tile.rotation_anim.is_some() {
        return; // Already animating
    }

    let (from_angle, to_angle, clockwise) = if let Some(target) = target_rotation {
        // Rotate to specific target rotation
        let current = tile.current_rotation as i32;
        let target = target as i32;

        // Calculate the shortest rotation path
        let clockwise_dist = (4 + target - current) % 4;
        let counter_clockwise_dist = (4 + current - target) % 4;

        let clockwise = clockwise_dist <= counter_clockwise_dist;
        let rotation_count = if clockwise { clockwise_dist } else { counter_clockwise_dist };

        // Calculate total rotation angle
        let total_angle = if clockwise { rotation_count * 90 } else { rotation_count * -90 };

        (0.0, total_angle as f32, clockwise)
    } else {
        // Default behavior: rotate by 90 degrees clockwise
        (0.0, 90.0, true)
    };

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
    // Collect pending rotations that need to be started
    let mut pending_rotations: Vec<(usize, u8)> = Vec::new();

    for (i, tile) in state.tiles.iter_mut().enumerate() {
        // Handle pending rotations (debounced fast rotations)
        if let Some(pending) = &mut tile.pending_rotation {
            pending.timer -= dt;
            if pending.timer <= 0.0 {
                // Timer expired, collect this rotation to start
                if tile.rotation_anim.is_none() {
                    pending_rotations.push((i, pending.target));
                }
                tile.pending_rotation = None;
            }
        }

        // Update existing rotation animations
        if let Some(anim) = &mut tile.rotation_anim {
            anim.elapsed += dt;
            let t = (anim.elapsed / anim.duration).min(1.0) as f32;
            anim.current_angle = anim.from_angle + (anim.to_angle - anim.from_angle) * t;
            if t >= 1.0 {
                tile.rotation_anim = None; // Animation complete
            }
        }
    }

    // Start pending rotations after the loop to avoid borrowing conflicts
    for (tile_index, target_rotation) in pending_rotations {
        start_tile_rotation_animation(state, tile_index, Some(target_rotation), 0.25);
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
    let path = Tile::find_walkable_path(start_index, target_index, &state.tiles).unwrap_or_else(||
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

/// Simple function to animate one tile to a specific index
pub fn animate_tile_to_index(state: &mut GameState, tile_index: usize, target_index: usize) {
    // Get board layout for animation
    let (_, _, tile_size, offset_x, offset_y) = state.get_board_layout(false);

    // Calculate current screen position
    let (from_x, from_y) = Tile::screen_position(tile_index, tile_size, offset_x, offset_y);

    // Create new animated tile
    let animated_tile = AnimatedTile {
        tile_index,
        pos: (from_x as f32, from_y as f32),
        velocity: (0.0, 0.0),
        target_index,
        animating: true,
    };

    // Add to list of animated tiles
    state.animated_tiles.push(animated_tile);
}

/// Update all tile animations
pub fn update_tile_animations(state: &mut GameState) {
    // Get board layout before mutable borrow
    let (_, _, tile_size, offset_x, offset_y) = state.get_board_layout(false);

    // Update each animated tile
    let mut completed_indices = Vec::new();

    for (i, anim) in state.animated_tiles.iter_mut().enumerate() {
        if anim.animating {
            // Calculate target screen position
            let (target_x, target_y) = Tile::screen_position(
                anim.target_index,
                tile_size,
                offset_x,
                offset_y
            );
            let target_pos = (target_x as f32, target_y as f32);

            let (new_pos, new_velocity, snapped) = spring_to_target(
                anim.pos,
                anim.velocity,
                target_pos,
                0.3, // spring - moderate speed
                0.4, // friction - some bounce
                2.0, // snap_distance
                1.0 // snap_velocity
            );

            anim.pos = new_pos;
            anim.velocity = new_velocity;

            if snapped {
                // Animation complete - mark for removal
                completed_indices.push(i);
            }
        }
    }

    // Remove completed animations (in reverse order to maintain indices)
    for &index in completed_indices.iter().rev() {
        state.animated_tiles.remove(index);
    }

    // Check if we have any pending swaps that can now be completed
    // (when both tiles in a swap have finished animating)
    let mut completed_swaps = Vec::new();
    for (i, (tile1, tile2)) in state.pending_swaps.iter().enumerate() {
        let tile1_animating = state.animated_tiles.iter().any(|anim| anim.tile_index == *tile1);
        let tile2_animating = state.animated_tiles.iter().any(|anim| anim.tile_index == *tile2);

        if !tile1_animating && !tile2_animating {
            // Both tiles have finished animating, perform the swap
            state.tiles.swap(*tile1, *tile2);
            completed_swaps.push(i);
        }
    }

    // Remove completed swaps (in reverse order to maintain indices)
    for &index in completed_swaps.iter().rev() {
        state.pending_swaps.remove(index);
    }
}
