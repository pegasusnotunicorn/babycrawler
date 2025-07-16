use crate::game::constants::MAP_SIZE;
use crate::game::tile::{ Tile, clear_highlights };
use crate::game::turn::commands::{ NextTurn, DealCard };
use crate::game::player::Player;
use crate::GameState;

use turbo::*;
use serde::{ Serialize, Deserialize };
use turbo::borsh::{ BorshDeserialize, BorshSerialize };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum CardEffect {
    MoveOneTile,
    RotateCard,
    SwapCard,
    // more to come
}

impl CardEffect {
    pub fn highlight_tiles(&self, player_pos: (usize, usize), tiles: &mut [Tile]) {
        clear_highlights(tiles);

        let (px, py) = player_pos;
        let current_index = Tile::index(px, py);
        let current_tile = tiles[current_index].clone();

        match self {
            CardEffect::MoveOneTile => {
                for (i, tile) in tiles.iter_mut().enumerate() {
                    if current_tile.can_move_to(current_index, tile, i) {
                        tile.is_highlighted = true;
                    }
                }
            }

            CardEffect::RotateCard => {
                for i in Tile::get_adjacent_indices(current_index, true, true) {
                    tiles[i].is_highlighted = true;
                }
            }

            CardEffect::SwapCard => {
                for i in Tile::get_adjacent_indices(current_index, true, true) {
                    tiles[i].is_highlighted = true;
                }
            }
        }
    }

    pub fn apply_effect(&self, state: &mut GameState, tile_index: usize) {
        match self {
            CardEffect::MoveOneTile => self.apply_move_one_tile(state, tile_index),
            CardEffect::RotateCard => self.apply_rotate_card(state, tile_index),
            CardEffect::SwapCard => self.apply_swap_card(state, tile_index),
        }
    }

    fn apply_move_one_tile(&self, state: &mut GameState, tile_index: usize) {
        let current_turn = state.current_turn;
        let (px, py) = state.players[current_turn].position;
        let current_index = py * MAP_SIZE + px;

        if tile_index == current_index {
            return;
        }

        if !state.tiles[tile_index].is_highlighted {
            return;
        }

        state.players[current_turn].move_to(tile_index);
        Self::end_turn(state);
    }

    fn apply_rotate_card(&self, state: &mut GameState, tile_index: usize) {
        if state.tiles[tile_index].is_highlighted {
            state.tiles[tile_index].rotate_clockwise(1);
            Self::end_turn(state);
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
        let player = &mut state.players[state.current_turn];
        let (px, py) = player.position;
        (player, py * MAP_SIZE + px)
    }

    fn end_turn(state: &mut GameState) {
        state.selected_cards.clear();
        clear_highlights(&mut state.tiles);
        let _ = DealCard.exec();
        let _ = NextTurn.exec();
    }
}
