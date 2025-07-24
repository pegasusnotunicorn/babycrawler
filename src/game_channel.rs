use turbo::*;
use serde::{ Serialize, Deserialize };
use crate::game::map::{ Tile, Player, PlayerId };
use crate::game::constants::{ MAP_SIZE, HAND_SIZE };
use crate::game::map::board::random_tiles;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum GameToClient {
    Turn {
        player_id: String,
    },
    ConnectedUsers {
        users: Vec<String>,
    },
    BoardState {
        tiles: Vec<Tile>,
        players: Vec<Player>,
    },
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum GameToServer {
    EndTurn,
}

#[turbo::os::channel(program = "server", name = "game")]
pub struct GameChannel {
    players: Vec<String>,
    current_turn_index: usize,
    board_tiles: Vec<Tile>,
    board_players: Vec<Player>,
}

impl os::server::channel::ChannelHandler for GameChannel {
    type Send = GameToClient;
    type Recv = GameToServer;

    fn new() -> Self {
        let board_players = vec![
            Player::new(PlayerId::Player1, 0, 0, HAND_SIZE),
            Player::new(PlayerId::Player2, MAP_SIZE - 1, MAP_SIZE - 1, HAND_SIZE)
        ];
        let board_tiles = random_tiles(MAP_SIZE * MAP_SIZE);
        Self {
            players: Vec::new(),
            current_turn_index: 0,
            board_tiles,
            board_players,
        }
    }

    fn on_connect(&mut self, user_id: &str) -> Result<(), std::io::Error> {
        log!("[GameChannel] on_connect called for user_id: {user_id}");
        if !self.players.contains(&user_id.to_string()) {
            self.players.push(user_id.to_string());
        }
        // TODO: handle broadcast error
        let _ = os::server::channel::broadcast(GameToClient::ConnectedUsers {
            users: self.players.clone(),
        });
        if self.players.len() == 1 {
            self.current_turn_index = 0;
        }
        self.broadcast_turn();
        self.broadcast_board_state();
        Ok(())
    }

    fn on_disconnect(&mut self, user_id: &str) -> Result<(), std::io::Error> {
        let was_turn = self.players.get(self.current_turn_index) == Some(&user_id.to_string());
        self.players.retain(|p| p != user_id);
        // TODO: handle broadcast error
        let _ = os::server::channel::broadcast(GameToClient::ConnectedUsers {
            users: self.players.clone(),
        });
        if was_turn && !self.players.is_empty() {
            self.current_turn_index %= self.players.len();
        }
        self.broadcast_turn();
        self.broadcast_board_state();
        Ok(())
    }

    fn on_data(&mut self, user_id: &str, data: Self::Recv) -> Result<(), std::io::Error> {
        log!("[GameChannel] on_data called for user_id: {user_id}");
        match data {
            GameToServer::EndTurn => {
                log!("[GameChannel] EndTurn received from {user_id}");
                if self.players.get(self.current_turn_index) == Some(&user_id.to_string()) {
                    self.current_turn_index = (self.current_turn_index + 1) % self.players.len();
                    self.broadcast_turn();
                    self.broadcast_board_state();
                }
            }
        }
        Ok(())
    }
}

impl GameChannel {
    fn broadcast_turn(&self) {
        if let Some(user_id) = self.players.get(self.current_turn_index) {
            // TODO: handle broadcast error
            let _ = os::server::channel::broadcast(GameToClient::Turn {
                player_id: user_id.clone(),
            });
        }
    }
    fn broadcast_board_state(&self) {
        // TODO: handle broadcast error
        let _ = os::server::channel::broadcast(GameToClient::BoardState {
            tiles: self.board_tiles.clone(),
            players: self.board_players.clone(),
        });
    }
}
