use turbo::*;
use serde::{ Serialize, Deserialize };
use crate::game::cards::card::Card;

pub mod send;
pub mod receive;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ClientToServer {
    ResetGame,
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
    FireballShot {
        target_tile: usize,
        direction: crate::game::map::tile::Direction,
    },
    FireballHit {
        shooter_id: String,
        hit_tile_index: usize,
    },
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClient {
    ConnectedUsers {
        users: Vec<String>,
    },
    ResetGame,
    BoardState {
        tiles: Vec<crate::game::map::tile::Tile>,
        players: Vec<crate::game::map::player::Player>,
        monster: Option<crate::game::map::monster::Monster>,
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
    FireballSpawned {
        fireball: crate::game::map::fireball::Fireball,
        player_id: String,
    },
    FireballHitResult {
        shooter_id: String,
        target_player_id: Option<String>,
        damage_dealt: u32,
        monster_damage: Option<u32>, // Optional monster damage if fireball hit monster
    },
    GameOver {
        winner_id: String,
        loser_id: String,
    },
}
