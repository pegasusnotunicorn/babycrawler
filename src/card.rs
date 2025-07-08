use crate::{ constants::MAP_SIZE, tile::Tile, GameState };
use crate::turn::commands::{ NextTurn, DealCard };
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, random::rand, * };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum CardEffect {
    MoveOneTile,
    RotateCard,
    // more to come
}

impl CardEffect {
    pub fn highlight_tiles(&self, player_pos: (usize, usize), tiles: &[Tile]) -> Vec<usize> {
        let (px, py) = player_pos;
        let current_index = py * MAP_SIZE + px;
        let current_tile = &tiles[current_index];

        match self {
            CardEffect::MoveOneTile =>
                tiles
                    .iter()
                    .enumerate()
                    .filter(|(_, tile)| current_tile.can_move_to(tile))
                    .map(|(i, _)| i)
                    .collect(),

            CardEffect::RotateCard =>
                tiles
                    .iter()
                    .enumerate()
                    .filter(|(_, tile)| {
                        let dx = ((tile.grid_x as isize) - (current_tile.grid_x as isize)).abs();
                        let dy = ((tile.grid_y as isize) - (current_tile.grid_y as isize)).abs();
                        dx + dy == 1 || (dx == 0 && dy == 0)
                    })
                    .map(|(i, _)| i)
                    .collect(),
        }
    }

    pub fn apply_effect(&self, state: &mut GameState, tile_index: usize) {
        let player = &mut state.players[state.current_turn];
        let (px, py) = player.position;
        let current_index = py * MAP_SIZE + px;

        // Make sure indices are not the same to prevent split overlap issues
        if current_index == tile_index {
            let tile = &mut state.tiles[current_index];
            match self {
                CardEffect::MoveOneTile => {
                    // Can't move to same tile, so do nothing
                }
                CardEffect::RotateCard => {
                    tile.rotate_clockwise(1);
                    state.selected_card = None;
                    let _ = DealCard.exec();
                    let _ = NextTurn.exec();
                }
            }
        } else {
            let (first, second) = if current_index < tile_index {
                state.tiles.split_at_mut(tile_index)
            } else {
                state.tiles.split_at_mut(current_index)
            };

            let (current_tile, target_tile) = if current_index < tile_index {
                (&first[current_index], &mut second[0])
            } else {
                (&second[0], &mut first[tile_index])
            };

            match self {
                CardEffect::MoveOneTile => {
                    if current_tile.can_move_to(target_tile) {
                        player.move_to(target_tile);
                        state.selected_card = None;
                        let _ = DealCard.exec();
                        let _ = NextTurn.exec();
                    }
                }
                CardEffect::RotateCard => {
                    let dx = ((target_tile.grid_x as isize) - (current_tile.grid_x as isize)).abs();
                    let dy = ((target_tile.grid_y as isize) - (current_tile.grid_y as isize)).abs();

                    if (dx == 1 && dy == 0) || (dx == 0 && dy == 1) {
                        target_tile.rotate_clockwise(1);
                        state.selected_card = None;
                        let _ = DealCard.exec();
                        let _ = NextTurn.exec();
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Card {
    pub id: u32,
    pub name: String,
    pub effect: CardEffect,
    pub color: u32,
}

const CARD_CONSTRUCTORS: &[fn() -> Card] = &[Card::move_card, Card::rotate_card];

impl Card {
    pub fn random() -> Self {
        let index = (rand() as usize) % CARD_CONSTRUCTORS.len();
        (CARD_CONSTRUCTORS[index])()
    }

    pub fn rotate_card() -> Self {
        Self {
            id: rand(),
            name: "Rotate".into(),
            effect: CardEffect::RotateCard,
            color: 0x3366ccff, // Blue
        }
    }

    pub fn move_card() -> Self {
        Self {
            id: rand(),
            name: "Move".into(),
            effect: CardEffect::MoveOneTile,
            color: 0x33cc33ff, // Green
        }
    }
}
