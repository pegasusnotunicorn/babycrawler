use turbo::*;
use serde::{ Serialize, Deserialize };
use crate::game::cards::card::Card;

pub mod send;
pub mod receive;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ClientToServer {
    EndTurn,
    SelectCard {
        card_index: usize,
        card: Card,
    },
    CancelSelectCard {
        card_index: usize,
        card: Card,
    },
    ConfirmCard {
        card: Card,
    },
    RotateTile {
        tile_index: usize,
        clockwise: bool,
    },
    MovePlayer {
        new_position: (usize, usize),
        is_canceled: bool,
    },
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClient {
    ConnectedUsers { users: Vec<String> },
    BoardState { 
        tiles: Vec<crate::game::map::tile::Tile>,
        players: Vec<crate::game::map::player::Player>,
        current_turn: Option<crate::server::CurrentTurn>,
    },
    CardSelected { 
        user_id: String, 
        card_index: usize, 
        card: Card,
        original_position: Option<(usize, usize)>,
    },
    CardCanceled { 
        user_id: String, 
        card_index: usize, 
        card: Card,
    },
    CardConfirmed { 
        user_id: String, 
        card: Card,
    },
    TileRotated { 
        tile_index: usize, 
        clockwise: bool,
    },
    PlayerMoved { 
        player_id: String, 
        new_position: (usize, usize),
    },
}
