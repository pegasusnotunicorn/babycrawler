use crate::game::constants::MAP_SIZE;
use crate::game::map::Tile;
use crate::game::animation::{ start_tile_rotation_animation, start_player_movement_animation };
use crate::GameState;
use crate::network::send::{ send_tile_rotation, send_move, send_swap_tiles };

use turbo::*;
use serde::{ Serialize, Deserialize };
use turbo::borsh::{ BorshDeserialize, BorshSerialize };

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    BorshDeserialize,
    BorshSerialize,
    Serialize,
    Deserialize
)]
pub enum CardEffect {
    Dummy,
    MoveOneTile,
    RotateCard,
    SwapCard,
    FireCard,
}

impl CardEffect {
    pub fn apply_effect(&self, state: &mut GameState, tile_index: usize) {
        match self {
            CardEffect::Dummy => {}
            CardEffect::MoveOneTile => self.apply_move_one_tile(state, tile_index),
            CardEffect::RotateCard => self.apply_rotate_card(state, tile_index),
            CardEffect::SwapCard => self.apply_swap_card(state, tile_index),
            CardEffect::FireCard => self.apply_fire_card(state, tile_index),
        }
    }

    fn apply_move_one_tile(&self, state: &mut GameState, tile_index: usize) {
        // Get board layout first to avoid borrowing conflicts
        let (_, _, tile_size, offset_x, offset_y) = state.get_board_layout(false);

        let player = state.get_turn_player().unwrap();
        let (px, py) = player.position;
        let current_position = player.position;
        let user_id = state.user.clone();

        let current_index = py * MAP_SIZE + px;

        if tile_index == current_index {
            return;
        }

        if !state.tiles[tile_index].is_highlighted {
            return;
        }

        // Check if player is already moving
        if let Some(animated_player) = &state.animated_player {
            if animated_player.animating {
                return; // Don't send move if already moving
            }
        }

        // Calculate new position from tile index
        let new_x = tile_index % MAP_SIZE;
        let new_y = tile_index / MAP_SIZE;
        let new_position = (new_x, new_y);

        // Start local player movement animation immediately
        start_player_movement_animation(
            state,
            &user_id,
            current_position,
            new_position,
            tile_size,
            offset_x,
            offset_y
        );

        send_move(new_position, false);
    }

    fn apply_rotate_card(&self, state: &mut GameState, tile_index: usize) {
        let tile = &mut state.tiles[tile_index];
        if tile.is_highlighted {
            if tile.rotation_anim.is_none() {
                send_tile_rotation(tile_index);
            }
            start_tile_rotation_animation(state, tile_index, None, 0.25);
        }
    }

    fn apply_swap_card(&self, state: &mut GameState, tile_index: usize) {
        // Check if this tile is selectable (within 1 tile distance from player)
        if !state.tiles[tile_index].is_highlighted {
            return;
        }

        // If this tile is already selected, deselect it
        if let Some(pos) = state.swap_tiles_selected.iter().position(|&i| i == tile_index) {
            state.swap_tiles_selected.remove(pos);
            return;
        }

        state.swap_tiles_selected.push(tile_index);
        state.tiles[tile_index].is_highlighted = true;

        // If we have 2 tiles selected, send swap request
        if state.swap_tiles_selected.len() == 2 {
            let tile1 = state.swap_tiles_selected[0];
            let tile2 = state.swap_tiles_selected[1];
            send_swap_tiles(tile1, tile2);
            state.swap_tiles_selected.clear();
        }
    }

    fn apply_fire_card(&self, _state: &mut GameState, _tile_index: usize) {
        // TODO: Implement fire card effect
        // For now, it does nothing
    }

    // Add a function to revert all tiles to their original_rotation
    pub fn revert_tile_rotations(tiles: &mut [Tile]) {
        for tile in tiles.iter_mut() {
            let current = tile.current_rotation as i32;
            let orig = tile.original_rotation as i32;
            let diff = (4 + orig - current) % 4;
            if diff > 0 {
                tile.rotate_entrances(orig as u8);
            }
            tile.rotation_anim = None;
        }
    }

    // Add a function to revert all tiles to their original positions
    // This function now uses the deferred approach - it collects tiles that need to move
    // and adds them to pending_swaps, then starts animations for visual movement
    pub fn revert_tile_positions(state: &mut GameState) {
        // Clear swap selection and highlights
        state.swap_tiles_selected.clear();
        crate::game::map::clear_highlights(&mut state.tiles);

        // For tile reverts, use the same deferred approach as swaps
        // First collect all tiles that need to move
        let mut tiles_to_animate: Vec<(usize, usize)> = Vec::new();
        for (current_index, tile) in state.tiles.iter().enumerate() {
            let target_index = tile.original_location;
            if current_index != target_index {
                tiles_to_animate.push((current_index, target_index));
            }
        }

        // Then add them to pending swaps and start animations
        for (current_index, target_index) in tiles_to_animate {
            state.pending_swaps.push((current_index, target_index));
            crate::game::animation::animate_tile_to_index(state, current_index, target_index);
        }
    }
}
