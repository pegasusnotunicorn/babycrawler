use crate::game::constants::MAP_SIZE;
use crate::game::map::{ Tile, clear_highlights, Player };
use crate::GameState;

use turbo::*;
use serde::{ Serialize, Deserialize };
use turbo::borsh::{ BorshDeserialize, BorshSerialize };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum CardEffect {
    Dummy,
    MoveOneTile,
    RotateCard,
    SwapCard,
}

impl CardEffect {
    pub fn apply_effect(&self, state: &mut GameState, tile_index: usize) {
        match self {
            CardEffect::Dummy => {}
            CardEffect::MoveOneTile => self.apply_move_one_tile(state, tile_index),
            CardEffect::RotateCard => self.apply_rotate_card(state, tile_index),
            CardEffect::SwapCard => self.apply_swap_card(state, tile_index),
        }
    }

    fn apply_move_one_tile(&self, state: &mut GameState, tile_index: usize) {
        let player = state.get_turn_player().unwrap();
        let (px, py) = player.position;
        let current_index = py * MAP_SIZE + px;

        if tile_index == current_index {
            return;
        }

        if !state.tiles[tile_index].is_highlighted {
            return;
        }

        let player = state.get_turn_player_mut().unwrap();
        player.move_to(tile_index);
        Self::end_turn(state);
    }

    fn apply_rotate_card(&self, state: &mut GameState, tile_index: usize) {
        if state.tiles[tile_index].is_highlighted {
            crate::game::animation::start_tile_rotation_animation(state, tile_index, true, 0.25);
        }
    }

    fn apply_swap_card(&self, state: &mut GameState, tile_index: usize) {
        let (_, current_index) = Self::current_player_and_index(state);

        if current_index == tile_index {
            return;
        }

        let (low, high) = if current_index < tile_index {
            (current_index, tile_index)
        } else {
            (tile_index, current_index)
        };

        let (left, right) = state.tiles.split_at_mut(high);
        let (_a, b) = if current_index < tile_index {
            (&mut left[low], &mut right[0])
        } else {
            (&mut right[0], &mut left[low])
        };

        if b.is_highlighted {
            state.tiles.swap(current_index, tile_index);
            Self::end_turn(state);
        }
    }

    fn current_player_and_index(state: &mut GameState) -> (&mut Player, usize) {
        let player = state.get_turn_player_mut().unwrap();
        let (px, py) = player.position;
        (player, py * MAP_SIZE + px)
    }

    fn end_turn(state: &mut GameState) {
        state.selected_card = None;
        clear_highlights(&mut state.tiles);
    }

    // Add a function to revert all tiles to their original_rotation
    pub fn revert_tile_rotations(tiles: &mut [Tile]) {
        for tile in tiles.iter_mut() {
            let current = tile.current_rotation as i32;
            let orig = tile.original_rotation as i32;
            let diff = (4 + orig - current) % 4;
            if diff > 0 {
                tile.rotate_clockwise(diff as usize);
            }
            tile.rotation_anim = None;
        }
    }
}
