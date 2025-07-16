use turbo::*;
use serde::{ Serialize, Deserialize };
use turbo::borsh::{ BorshSerialize, BorshDeserialize };

#[turbo::program]
pub mod server {
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
    }

    impl turbo::ChannelHandler for GameChannel {
        type Send = GameToClient;
        type Recv = GameToServer;

        fn new() -> Self {
            Self {
                players: Vec::new(),
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
            // Optionally, broadcast whose turn it is (first player by default)
            if self.players.len() == 1 {
                server::channel::broadcast(GameToClient::Turn {
                    player_id: self.players[0].clone(),
                });
            }
        }

        fn on_disconnect(&mut self, user_id: &str) {
            use turbo::os::server;
            self.players.retain(|p| p != user_id);
            server::channel::broadcast(GameToClient::ConnectedUsers {
                users: self.players.clone(),
            });
            // Optionally, broadcast new turn if needed
            if !self.players.is_empty() {
                server::channel::broadcast(GameToClient::Turn {
                    player_id: self.players[0].clone(),
                });
            }
        }

        fn on_data(&mut self, user_id: &str, data: Self::Recv) {
            use turbo::os::server;
            server::log!("[GameChannel] on_data called for user_id: {user_id}");
            match data {
                GameToServer::EndTurn => {
                    server::log!("[GameChannel] EndTurn received from {user_id}");
                    // Advance to next player
                    if let Some(idx) = self.players.iter().position(|id| id == user_id) {
                        let next_idx = (idx + 1) % self.players.len();
                        server::log!("[GameChannel] Next player index: {next_idx}");
                        let next_player_id = self.players[next_idx].clone();
                        server::channel::broadcast(GameToClient::Turn {
                            player_id: next_player_id,
                        });
                    }
                }
            }
        }
    }
}
