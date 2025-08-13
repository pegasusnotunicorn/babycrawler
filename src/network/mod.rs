use turbo::*;
use serde::{ Serialize, Deserialize };
use crate::game::cards::card::Card;

pub mod send;
pub mod receive;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ClientToServer {
    EndTurn,
    SelectCard {
        hand_index: usize,
    },
    CancelSelectCard {
        hand_index: usize,
    },
    ConfirmCard {
        card: Card,
    },
    RotateTile {
        tile_index: usize,
    },
    MovePlayer {
        new_position: (usize, usize),
        is_canceled: bool,
    },
    SwapTiles {
        tile_index_1: usize,
        tile_index_2: usize,
    },
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClient {
    ConnectedUsers {
        users: Vec<String>,
    },
    BoardState {
        tiles: Vec<crate::game::map::tile::Tile>,
        players: Vec<crate::game::map::player::Player>,
        current_turn: Option<crate::server::CurrentTurn>,
    },

    CardCancelled {
        card_index: usize,
        card: Card,
        player_id: String,
    },
    CardConfirmed {
        card: Card,
        player_id: String,
    },
    TileRotated {
        tile_index: usize,
        tile: crate::game::map::tile::Tile,
        player_id: String,
    },
    PlayerMoved {
        player_id: String,
        new_position: (usize, usize),
        is_canceled: bool,
    },
    TilesSwapped {
        tile_index_1: usize,
        tile_index_2: usize,
    },
}
