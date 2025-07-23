use turbo::*;
use serde::{ Serialize, Deserialize };
use turbo::borsh::{ BorshSerialize, BorshDeserialize };

#[turbo::program]
pub mod game_server {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
    pub enum GameToClient {
        Turn {
            player_id: String,
        },
        ConnectedUsers {
            users: Vec<String>,
        },
    }

    #[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
    pub enum GameToServer {
        EndTurn,
    }

    #[turbo::channel(name = "game")]
    pub struct GameChannel {
        players: Vec<String>,
        current_turn_index: usize,
    }

    impl turbo::ChannelHandler for GameChannel {
        type Send = GameToClient;
        type Recv = GameToServer;

        fn new() -> Self {
            Self {
                players: Vec::new(),
                current_turn_index: 0,
            }
        }

        fn on_connect(&mut self, user_id: &str) {
            use turbo::os::server;
            server::log!("[GameChannel] on_connect called for user_id: {user_id}");
            if !self.players.contains(&user_id.to_string()) {
                self.players.push(user_id.to_string());
            }
            server::channel::broadcast(GameToClient::ConnectedUsers {
                users: self.players.clone(),
            });
            if self.players.len() == 1 {
                self.current_turn_index = 0;
            }
            self.broadcast_turn();
        }

        fn on_disconnect(&mut self, user_id: &str) {
            use turbo::os::server;
            let was_turn = self.players.get(self.current_turn_index) == Some(&user_id.to_string());
            self.players.retain(|p| p != user_id);
            server::channel::broadcast(GameToClient::ConnectedUsers {
                users: self.players.clone(),
            });
            if was_turn && !self.players.is_empty() {
                self.current_turn_index %= self.players.len();
            }
            self.broadcast_turn();
        }

        fn on_data(&mut self, user_id: &str, data: Self::Recv) {
            use turbo::os::server;
            server::log!("[GameChannel] on_data called for user_id: {user_id}");
            match data {
                GameToServer::EndTurn => {
                    server::log!("[GameChannel] EndTurn received from {user_id}");
                    if self.players.get(self.current_turn_index) == Some(&user_id.to_string()) {
                        self.current_turn_index =
                            (self.current_turn_index + 1) % self.players.len();
                        self.broadcast_turn();
                    }
                }
            }
        }
    }

    impl GameChannel {
        fn broadcast_turn(&self) {
            use turbo::os::server;
            if let Some(user_id) = self.players.get(self.current_turn_index) {
                server::channel::broadcast(GameToClient::Turn {
                    player_id: user_id.clone(),
                });
            }
        }
    }
}
