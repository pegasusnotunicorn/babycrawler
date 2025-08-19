use turbo::*;
use crate::GameState;
use crate::game::map::tile::{ TileRotationAnim, Tile, clear_highlights };
use crate::game::map::tile_effects::highlight_tiles_for_effect;
use crate::network::send::send_card_selection;
use crate::game::cards::card::Card;
use crate::game::map::tile::Direction;
use crate::network::send::send_fireball_hit;

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

#[derive(
    Clone,
    Debug,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub struct AnimatedFireball {
    pub fireball_id: u32,
    pub current_pos: (f32, f32), // current screen position in pixels
    pub direction: Direction,
    pub animating: bool,
    pub current_tile_index: usize, // which tile we're currently in
}

pub fn update_animations(state: &mut GameState) {
    if update_animated_card_spring(state) {
        handle_animated_card_complete(state);
    }
    update_tile_rotation_animations(state, 1.0 / 60.0);
    update_player_movement_animations(state);
    update_tile_animations(state);
    update_fireball_animations(state);

    // Update player sprite animations
    update_player_sprite_animations(state);
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
/// Simply animates from the current rotation to the target rotation.
pub fn start_tile_rotation_animation(state: &mut GameState, tile_index: usize, duration: f64) {
    let tile = &mut state.tiles[tile_index];
    if tile.rotation_anim.is_some() {
        return; // Already animating
    }

    let (_, current_rotation_offset) = tile.get_wall_sprite_and_rotation();

    let from_angle = current_rotation_offset;
    let to_angle = current_rotation_offset + 90.0; // Always rotate 90° clockwise

    tile.rotation_anim = Some(TileRotationAnim {
        from_angle,
        to_angle,
        current_angle: 0.0,
        duration,
        elapsed: 0.0,
    });
}

/// Call this every frame to update all tile rotation animations.
pub fn update_tile_rotation_animations(state: &mut GameState, dt: f64) {
    for tile in state.tiles.iter_mut() {
        if let Some(anim) = &mut tile.rotation_anim {
            anim.elapsed += dt;
            let t = (anim.elapsed / anim.duration).min(1.0) as f32;
            anim.current_angle = (anim.to_angle - anim.from_angle) * t;
            if t >= 1.0 {
                let new_rotation = (tile.current_rotation + 1) % 4;
                tile.rotate_entrances(new_rotation);
                tile.current_rotation = new_rotation;
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

    // Set the player's moving state to true
    if let Some(player) = state.get_player_by_user_id(player_id) {
        player.set_moving(true);
    }
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

    // Set the player's moving state to true
    if let Some(player) = state.get_player_by_user_id(player_id) {
        player.set_moving(true);
    }
}

/// Update player movement animations
pub fn update_player_movement_animations(state: &mut GameState) {
    // Get board layout before mutable borrow
    let (_, _, tile_size, offset_x, offset_y) = state.get_board_layout(false);

    let mut direction_update: Option<(String, crate::game::map::player::Direction)> = None;

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

                // Calculate movement direction for sprite facing
                let (from_x, from_y) = anim.pos;
                let (to_x, to_y) = new_pos;
                let dx = to_x - from_x;
                let dy = to_y - from_y;

                // Only update direction if there's significant movement (prevents oscillation)
                let movement_threshold = 1.0; // minimum pixels of movement
                if dx.abs() > movement_threshold || dy.abs() > movement_threshold {
                    // Determine which direction the player should face
                    let new_direction = if dy.abs() > dx.abs() {
                        if dy > 0.0 {
                            crate::game::map::player::Direction::Down
                        } else {
                            crate::game::map::player::Direction::Up
                        }
                    } else {
                        if dx > 0.0 {
                            crate::game::map::player::Direction::Right
                        } else {
                            crate::game::map::player::Direction::Left
                        }
                    };

                    direction_update = Some((anim.player_id.clone(), new_direction));
                }

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
                            player.set_moving(false); // Stop moving when animation completes
                        }
                    }
                }
            }
        }
    }

    // Apply direction update after animation update to avoid borrowing conflicts
    if let Some((player_id, direction)) = direction_update {
        if let Some(player) = state.get_player_by_user_id(&player_id) {
            player.set_direction(direction);
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

/// Start fireball animation from player position
pub fn start_fireball_animation(
    state: &mut GameState,
    fireball_id: u32,
    start_pos: (usize, usize),
    direction: Direction,
    _target_tile: usize
) {
    log!(
        "🔥 [ANIMATION] Starting fireball animation: id={}, start={:?}, direction={:?}",
        fireball_id,
        start_pos,
        direction
    );

    // Get board layout for initial position calculation
    let (_, _, tile_size, offset_x, offset_y) = state.get_board_layout(false);

    // Calculate starting screen position (center of starting tile)
    let start_tile_index = Tile::index(start_pos.0, start_pos.1);
    let (start_x, start_y) = Tile::screen_position(start_tile_index, tile_size, offset_x, offset_y);
    let start_screen_pos = (
        (start_x as f32) + (tile_size as f32) / 2.0,
        (start_y as f32) + (tile_size as f32) / 2.0,
    );

    // Create animated fireball
    let animated_fireball = AnimatedFireball {
        fireball_id,
        current_pos: start_screen_pos,
        direction,
        animating: true,
        current_tile_index: start_tile_index,
    };

    state.animated_fireballs.push(animated_fireball);
}

/// Update all fireball animations
pub fn update_fireball_animations(state: &mut GameState) {
    let mut completed_indices = Vec::new();
    let mut fireballs_to_deactivate = Vec::new();

    // Get board layout for position calculations
    let (_, _, tile_size, offset_x, offset_y) = state.get_board_layout(false);

    // Get local player ID before the loop to avoid borrowing issues
    let local_player_id = state.get_local_player().map(|p| p.id.clone());

    for (i, anim) in state.animated_fireballs.iter_mut().enumerate() {
        if anim.animating {
            // Move fireball by pixels per frame
            let speed = 10.0;
            let (x, y) = anim.current_pos;

            let new_pos = match anim.direction {
                Direction::Up => (x, y - speed),
                Direction::Down => (x, y + speed),
                Direction::Left => (x - speed, y),
                Direction::Right => (x + speed, y),
            };

            // Check if we hit a wall or player using the cleaner helper approach
            let fireball_radius = (tile_size as f32) / 8.0; // tile_size / 4 / 2

            // First check if we hit a player (excluding the shooter)
            let player_hit = {
                let shooter_id = if
                    let Some(fireball) = state.fireballs.iter().find(|f| f.id == anim.fireball_id)
                {
                    fireball.shooter_id.clone()
                } else {
                    continue; // Skip if fireball not found
                };

                // Get positions of all players except the shooter
                let other_player_positions: Vec<_> = state.players
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.id != shooter_id)
                    .map(|(_, p)| p.position)
                    .collect();

                Tile::would_fireball_hit_player(
                    new_pos,
                    fireball_radius,
                    &other_player_positions,
                    tile_size,
                    offset_x,
                    offset_y
                )
            };

            let hit_wall = if player_hit.is_some() {
                false // Player hit takes priority over wall hit
            } else {
                let current_tile = &state.tiles[anim.current_tile_index];

                // Check if we've reached the far edge of the current tile (minus fireball radius)
                let reached_far_edge = Tile::has_fireball_reached_far_edge(
                    anim.current_tile_index,
                    anim.direction,
                    new_pos,
                    fireball_radius,
                    tile_size,
                    offset_x,
                    offset_y
                );

                if reached_far_edge {
                    // Use the helper function to check if we'd hit a wall
                    current_tile.would_fireball_hit_wall(
                        anim.current_tile_index,
                        anim.direction,
                        &state.tiles
                    )
                } else {
                    false // Still within current tile, no wall hit
                }
            };

            if hit_wall || player_hit.is_some() {
                if let Some(player_index) = player_hit {
                    log!(
                        "🔥 [ANIMATION] Fireball {} hit player {} at {:?}",
                        anim.fireball_id,
                        player_index,
                        anim.current_pos
                    );

                    // Only send the hit to the server if I'm the one who shot this fireball
                    if
                        let Some(fireball) = state.fireballs
                            .iter()
                            .find(|f| f.id == anim.fireball_id)
                    {
                        // Check if the local player is the shooter
                        if let Some(ref local_id) = local_player_id {
                            if fireball.shooter_id == *local_id {
                                // Send FireballHit message to server
                                send_fireball_hit(
                                    state.user.clone(),
                                    anim.current_tile_index,
                                    fireball.direction
                                );
                            }
                        }
                    }
                }
                completed_indices.push(i);
                fireballs_to_deactivate.push(anim.fireball_id);
            } else {
                // Move to new position
                anim.current_pos = new_pos;

                // Update the main fireball position to stay in sync
                if
                    let Some(fireball) = state.fireballs
                        .iter_mut()
                        .find(|f| f.id == anim.fireball_id)
                {
                    // Convert screen position back to tile position for the fireball struct
                    let tile_x = ((new_pos.0 - (offset_x as f32)) / (tile_size as f32)) as usize;
                    let tile_y = ((new_pos.1 - (offset_y as f32)) / (tile_size as f32)) as usize;
                    fireball.position = (tile_x.min(4), tile_y.min(4)); // Clamp to map bounds
                }

                // Check if we've moved to a new tile and update current_tile_index
                let new_tile_x = ((new_pos.0 - (offset_x as f32)) / (tile_size as f32)) as usize;
                let new_tile_y = ((new_pos.1 - (offset_y as f32)) / (tile_size as f32)) as usize;
                let new_tile_index = new_tile_x + new_tile_y * 5;

                if new_tile_index != anim.current_tile_index && new_tile_index < 25 {
                    anim.current_tile_index = new_tile_index;
                }
            }
        }
    }

    // Deactivate completed fireballs
    for fireball_id in fireballs_to_deactivate {
        if let Some(fireball) = state.fireballs.iter_mut().find(|f| f.id == fireball_id) {
            fireball.deactivate();
        }
    }

    // Remove completed animations (in reverse order to maintain indices)
    for &index in completed_indices.iter().rev() {
        state.animated_fireballs.remove(index);
    }
}

/// Updates player sprite animations (walking frames, etc.)
pub fn update_player_sprite_animations(state: &mut GameState) {
    let delta_time = 1.0 / 60.0; // Assuming 60 FPS

    for player in &mut state.players {
        player.update_animation(delta_time);
    }
}
