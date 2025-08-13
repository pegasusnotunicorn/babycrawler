use crate::game::map::{ Tile, Player };
use crate::game::cards::card::Card;
use serde::{ Serialize, Deserialize };
use borsh::{ BorshSerialize, BorshDeserialize };

pub mod game_channel;
pub mod broadcast;
pub mod handlers;

pub use game_channel::{ GameChannel, CurrentTurn };

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClient {
    ConnectedUsers {
        users: Vec<String>,
    },
    BoardState {
        tiles: Vec<Tile>,
        players: Vec<Player>,
        current_turn: Option<CurrentTurn>,
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
        tile: Tile,
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
