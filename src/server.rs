use turbo::*;
use borsh::BorshSerialize;
use serde::{ Serialize, Deserialize };
use crate::game::map::{ Tile, Player, PlayerId };
use crate::game::constants::{ MAP_SIZE, HAND_SIZE };
use crate::game::map::board::random_tiles;
use crate::network::ClientToServer;
use crate::game::cards::card::Card;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClient {
    ConnectedUsers {
        users: Vec<String>,
    },
    BoardState {
        tiles: Vec<Tile>,
        players: Vec<Player>,
        current_turn: Option<CurrentTurn>,
    },
    CardSelected {
        card_index: usize,
        card: Card,
        player_id: String,
    },
    CardCanceled {
        card_index: usize,
        player_id: String,
    },
}

#[turbo::os::channel(program = "server", name = "game")]
pub struct GameChannel {
    players: Vec<String>,
    current_turn_index: usize,
    current_turn: Option<CurrentTurn>,
    board_tiles: Vec<Tile>,
    board_players: Vec<Player>,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct CurrentTurn {
    pub player_id: String,
    pub selected_card: Option<Card>,
    pub selected_card_index: usize,
}

impl os::server::channel::ChannelHandler for GameChannel {
    type Send = ServerToClient;
    type Recv = ClientToServer;

    fn new() -> Self {
        let board_players = vec![
            Player::new(PlayerId::Player1, 0, 0, HAND_SIZE),
            Player::new(PlayerId::Player2, MAP_SIZE - 1, MAP_SIZE - 1, HAND_SIZE)
        ];
        let board_tiles = random_tiles(MAP_SIZE * MAP_SIZE);
        Self {
            players: Vec::new(),
            current_turn_index: 0,
            current_turn: None,
            board_tiles,
            board_players,
        }
    }

    fn on_connect(&mut self, user_id: &str) -> Result<(), std::io::Error> {
        log!("[GameChannel] on_connect called for user_id: {user_id}");
        if !self.players.contains(&user_id.to_string()) {
            self.players.push(user_id.to_string());
        }
        self.broadcast_generic(ServerToClient::ConnectedUsers {
            users: self.players.clone(),
        });
        if self.players.len() == 1 {
            self.current_turn_index = 0;
        }
        self.broadcast_turn();
        Ok(())
    }

    fn on_disconnect(&mut self, user_id: &str) -> Result<(), std::io::Error> {
        let was_turn = self.players.get(self.current_turn_index) == Some(&user_id.to_string());
        self.players.retain(|p| p != user_id);
        self.broadcast_generic(ServerToClient::ConnectedUsers {
            users: self.players.clone(),
        });
        if was_turn && !self.players.is_empty() {
            self.current_turn_index %= self.players.len();
        }
        self.broadcast_turn();
        Ok(())
    }

    fn on_data(&mut self, user_id: &str, data: Self::Recv) -> Result<(), std::io::Error> {
        log!("[GameChannel] on_data called for user_id: {user_id}");
        match data {
            ClientToServer::EndTurn => {
                log!("[GameChannel] EndTurn received from {user_id}");
                if self.players.get(self.current_turn_index) == Some(&user_id.to_string()) {
                    self.current_turn_index = (self.current_turn_index + 1) % self.players.len();
                    self.broadcast_turn();
                }
            }
            ClientToServer::SelectCard { card_index, card } => {
                log!(
                    "[GameChannel] SelectCard received from {user_id}: index={}, card={:?}",
                    card_index,
                    card
                );
                self.current_turn = Some(CurrentTurn {
                    player_id: user_id.to_string(),
                    selected_card: Some(card.clone()),
                    selected_card_index: card_index,
                });
                self.broadcast_card_selected(card_index, card, user_id);
            }
            ClientToServer::CancelSelectCard { card_index } => {
                log!("[GameChannel] CancelSelectCard received from {user_id}: index={}", card_index);
                self.current_turn = Some(CurrentTurn {
                    player_id: user_id.to_string(),
                    selected_card: None,
                    selected_card_index: card_index,
                });
                self.broadcast_card_canceled(card_index, user_id);
            }
        }
        Ok(())
    }
}

impl GameChannel {
    fn broadcast_generic<T: Serialize + BorshSerialize>(&self, msg: T) {
        log!("[GameChannel] Attempting to broadcast message: {:?}", std::any::type_name::<T>());
        if let Err(e) = os::server::channel::broadcast(msg) {
            log!("[GameChannel] Error broadcasting message: {e}");
        } else {
            log!("[GameChannel] Successfully broadcasted message");
        }
    }

    fn broadcast_turn(&mut self) {
        if let Some(user_id) = self.players.get(self.current_turn_index) {
            // Update current_turn with the new player
            self.current_turn = Some(CurrentTurn {
                player_id: user_id.clone(),
                selected_card: None,
                selected_card_index: 0,
            });
            // Broadcast the updated board state which includes the current turn
            self.broadcast_generic(ServerToClient::BoardState {
                tiles: self.board_tiles.clone(),
                players: self.board_players.clone(),
                current_turn: self.current_turn.clone(),
            });
        }
    }

    fn broadcast_card_selected(&self, card_index: usize, card: Card, player_id: &str) {
        log!(
            "[GameChannel] Broadcasting card selected: index={}, card={:?}, player={}",
            card_index,
            card,
            player_id
        );
        self.broadcast_generic(ServerToClient::CardSelected {
            card_index,
            card,
            player_id: player_id.to_string(),
        });
    }

    fn broadcast_card_canceled(&self, card_index: usize, player_id: &str) {
        log!(
            "[GameChannel] Broadcasting card canceled: index={}, player={}",
            card_index,
            player_id
        );
        self.broadcast_generic(ServerToClient::CardCanceled {
            card_index,
            player_id: player_id.to_string(),
        });
    }
}
