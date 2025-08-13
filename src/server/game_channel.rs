use turbo::*;
use serde::{ Serialize, Deserialize };
use crate::game::map::{ Tile, Player, PlayerId };
use crate::game::constants::{ MAP_SIZE, HAND_SIZE };
use crate::game::map::board::random_tiles;
use crate::network::ClientToServer;
use crate::game::cards::card::Card;
use crate::server::broadcast::{ broadcast_generic, broadcast_turn, broadcast_board_state };
use crate::server::handlers::*;

#[turbo::os::channel(program = "game_server", name = "game")]
pub struct GameChannel {
    pub players: Vec<String>,
    pub current_turn_index: usize,
    pub current_turn: Option<CurrentTurn>,
    pub board_tiles: Vec<Tile>,
    pub board_players: Vec<Player>,
}

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct CurrentTurn {
    pub player_id: String,
    pub selected_card: Option<Card>,
    pub selected_card_index: usize,
    pub confirmed_cards_count: usize,
}

impl os::server::channel::ChannelHandler for GameChannel {
    type Send = crate::server::ServerToClient;
    type Recv = ClientToServer;

    fn new() -> Self {
        let board_players = vec![
            Player::new(PlayerId::Player1, 0, 0, HAND_SIZE, false),
            Player::new(PlayerId::Player2, MAP_SIZE - 1, MAP_SIZE - 1, HAND_SIZE, false)
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
        broadcast_generic(crate::server::ServerToClient::ConnectedUsers {
            users: self.players.clone(),
        });

        // Only start the game when we have 2 players
        if self.players.len() == 2 {
            self.current_turn_index = 0;

            // Give initial hands to both players
            let player_ids: Vec<String> = self.players.clone();
            for player_id in player_ids {
                crate::server::handlers::give_player_new_hand(self, &player_id);
            }

            broadcast_turn(
                &self.players,
                self.current_turn_index,
                &mut self.current_turn,
                &self.board_tiles,
                &self.board_players
            );
        }
        Ok(())
    }

    fn on_disconnect(&mut self, user_id: &str) -> Result<(), std::io::Error> {
        let was_turn = self.players.get(self.current_turn_index) == Some(&user_id.to_string());
        self.players.retain(|p| p != user_id);
        broadcast_generic(crate::server::ServerToClient::ConnectedUsers {
            users: self.players.clone(),
        });

        if self.players.len() < 2 {
            // Game stops if we have less than 2 players
            self.current_turn = None;
            broadcast_board_state(&self.board_tiles, &self.board_players, &None);
        } else {
            // Continue game with remaining players
            if was_turn {
                self.current_turn_index %= self.players.len();
            }
            broadcast_turn(
                &self.players,
                self.current_turn_index,
                &mut self.current_turn,
                &self.board_tiles,
                &self.board_players
            );
        }
        Ok(())
    }

    fn on_data(&mut self, user_id: &str, data: Self::Recv) -> Result<(), std::io::Error> {
        log!("[GameChannel] on_data called for user_id: {user_id}");
        match data {
            ClientToServer::EndTurn => {
                handle_new_turn(self, user_id);
            }
            ClientToServer::SelectCard { hand_index } => {
                handle_select_card(self, user_id, hand_index);
            }
            ClientToServer::CancelSelectCard { hand_index } => {
                handle_cancel_select_card(self, user_id, hand_index);
            }
            ClientToServer::ConfirmCard { card } => {
                handle_confirm_card(self, user_id, card);
            }
            ClientToServer::RotateTile { tile_index } => {
                handle_rotate_tile(self, user_id, tile_index);
            }
            ClientToServer::MovePlayer { new_position, is_canceled } => {
                handle_move_player(self, user_id, new_position, is_canceled);
            }
            ClientToServer::SwapTiles { tile_index_1, tile_index_2 } => {
                handle_swap_tiles(self, user_id, tile_index_1, tile_index_2);
            }
            ClientToServer::FireballShot { target_tile, direction } => {
                handle_fireball_shot(self, user_id, target_tile, direction);
            }
            ClientToServer::FireballHit { shooter_id, from_tile_index, direction } => {
                handle_fireball_hit(self, &shooter_id, from_tile_index, direction);
            }
        }
        Ok(())
    }
}
