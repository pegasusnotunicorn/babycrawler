mod server;
use crate::server::{ GameChannel, CurrentTurn, ServerToClient };
mod game;
mod scene;
mod network;

use crate::game::map::draw_board;
use crate::game::constants::{ GAME_PADDING, HAND_SIZE, MAP_SIZE, GAME_BG_COLOR, GAME_CHANNEL };
use crate::game::inputs::handle_input;
use crate::game::map::{ Player, PlayerId };
use crate::game::map::Tile;
use crate::game::ui::{ draw_turn_label, draw_waiting_for_players, draw_menu };
use crate::game::animation::{ update_animations, AnimatedCard, AnimatedPlayer };
use crate::game::debug::draw_debug;
use crate::game::cards::{ draw_play_area, draw_hand };
use crate::game::cards::card::Card;
use crate::game::cards::play_area::fill_with_dummies;
use crate::network::receive::*;

use turbo::{ os, gamepad, bounds, * };
use scene::Scene;

use std::fmt;
use std::collections::HashMap;

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerId::Player1 => write!(f, "Player1"),
            PlayerId::Player2 => write!(f, "Player2"),
        }
    }
}

#[turbo::game]
pub struct GameState {
    pub frame: usize,
    pub tiles: Vec<Tile>,
    pub players: Vec<Player>,
    pub selected_card: Option<Card>,
    pub scene: Scene, // Track current scene (menu or game)
    pub user: String, // This client's user id
    pub in_lobby: Vec<String>, // Users in the current game lobby
    pub debug: bool,
    pub user_id_to_player_id: HashMap<String, PlayerId>,
    pub animated_card: Option<AnimatedCard>,
    pub animated_player: Option<AnimatedPlayer>,
    pub play_area: Vec<Card>,
    pub current_turn: Option<CurrentTurn>,
}

impl GameState {
    pub fn new() -> Self {
        let debug = true; // Hardcoded for development

        Self {
            debug,
            frame: 0,
            tiles: Vec::new(),
            players: Vec::new(),
            selected_card: None,
            scene: Scene::Menu, // Start in menu scene
            user: String::new(), // Will be set on connect
            in_lobby: Vec::new(),
            user_id_to_player_id: HashMap::new(),
            animated_card: None,
            animated_player: None,
            play_area: {
                let mut play_area = Vec::new();
                fill_with_dummies(&mut play_area, HAND_SIZE);
                play_area
            },
            current_turn: None,
        }
    }

    // #region getters

    /// Returns true if it's this user's turn
    fn is_my_turn(&self) -> bool {
        self.current_turn.as_ref().map_or(false, |turn| self.user == turn.player_id)
    }

    /// Helper to get the current turn player by ID (returns Option<&Player>)
    fn get_turn_player(&self) -> Option<&Player> {
        let user_id = self.current_turn.as_ref()?.player_id.as_str();
        let player_id = self.user_id_to_player_id.get(user_id)?;
        self.players.iter().find(|p| &p.id == player_id)
    }

    /// Helper to get the current turn player as mutable
    fn get_turn_player_mut(&mut self) -> Option<&mut Player> {
        let user_id = self.current_turn.as_ref()?.player_id.as_str();
        let player_id = self.user_id_to_player_id.get(user_id)?;
        self.players.iter_mut().find(|p| &p.id == player_id)
    }

    /// Returns a reference to the Player struct for the current user, if any.
    pub fn get_local_player(&self) -> Option<&Player> {
        let player_id = self.user_id_to_player_id.get(&self.user)?;
        self.players.iter().find(|p| &p.id == player_id)
    }

    /// Returns a reference to the Player struct for the current user, if any (public helper for use in other modules).
    pub fn get_local_player_ref(&self) -> Option<&Player> {
        self.get_local_player()
    }

    /// Returns a mutable reference to the Player struct for the current user, if any.
    pub fn get_local_player_mut(&mut self) -> Option<&mut Player> {
        let player_id = self.user_id_to_player_id.get(&self.user)?;
        self.players.iter_mut().find(|p| &p.id == player_id)
    }

    /// Returns a mutable reference to a player by user_id
    pub fn get_player_by_user_id(&mut self, user_id: &str) -> Option<&mut Player> {
        let player_id = self.user_id_to_player_id.get(user_id)?;
        self.players.iter_mut().find(|p| &p.id == player_id)
    }

    /// Returns (canvas_width, canvas_height, tile_size, offset_x, offset_y) for the board layout
    pub fn get_board_layout(&self, padded: bool) -> (u32, u32, u32, u32, u32) {
        let canvas_width = bounds::screen().w() - (if padded { GAME_PADDING * 2 } else { 0 });
        let canvas_height = bounds::screen().h();
        let tile_size = canvas_width / (MAP_SIZE as u32);
        let offset_x = canvas_width / 2 - (tile_size * (MAP_SIZE as u32)) / 2;
        let offset_y = 0;
        (canvas_width, canvas_height, tile_size, offset_x, offset_y)
    }

    // #endregion

    pub fn update(&mut self) {
        clear(GAME_BG_COLOR);
        self.frame += 1;
        update_animations(self);
        match self.scene {
            Scene::Menu => self.update_menu(),
            Scene::Game => self.update_game(),
        }
        draw_debug(self);
    }

    fn update_menu(&mut self) {
        draw_menu();
        if gamepad::get(0).start.just_pressed() {
            self.scene = Scene::Game;
            self.user = os::client::user_id().unwrap_or_else(|| "NO_ID".to_string());
            // In a real game, you would get the mapping from the server or lobby
            // For now, assign the first two users to Player1 and Player2 as a placeholder
            self.user_id_to_player_id.clear();
            if self.in_lobby.len() >= 2 {
                self.user_id_to_player_id.insert(self.in_lobby[0].clone(), PlayerId::Player1);
                self.user_id_to_player_id.insert(self.in_lobby[1].clone(), PlayerId::Player2);
            }
        }
    }

    fn update_game(&mut self) {
        if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
            while let Ok(msg) = conn.recv() {
                match msg {
                    ServerToClient::ConnectedUsers { users } => {
                        receive_connected_users(self, users);
                    }

                    ServerToClient::BoardState { tiles, players, current_turn } => {
                        receive_board_state(self, tiles, players, current_turn);
                    }

                    ServerToClient::CardSelected { card_index, card, player_id } => {
                        receive_card_selection(self, &card_index, &card, &player_id);
                    }

                    ServerToClient::CardCanceled { card_index, card, player_id } => {
                        receive_card_cancel(self, &card_index, &card, &player_id);
                    }
                    ServerToClient::CardConfirmed { card, player_id } => {
                        receive_card_confirmed(self, &card, &player_id);
                    }

                    ServerToClient::TileRotated { tile_index, clockwise, player_id } => {
                        receive_tile_rotation(self, &tile_index, &clockwise, &player_id);
                    }

                    ServerToClient::PlayerMoved { player_id, new_position, is_canceled } => {
                        receive_player_moved(self, &player_id, &new_position, is_canceled);
                    }
                }
            }

            self.draw_game();

            if self.is_my_turn() {
                handle_input(self);
            }
        }
    }

    fn draw_game(&self) {
        clear(GAME_BG_COLOR);
        let (_canvas_width, _canvas_height, tile_size, offset_x, offset_y) =
            self.get_board_layout(false);
        draw_board(self, self.frame as f64, tile_size, offset_x, offset_y);
        draw_play_area(self, self.frame as f64);

        if self.get_local_player().is_some() {
            draw_hand(self, self.frame as f64);
            if self.current_turn.is_some() {
                draw_turn_label(self.is_my_turn(), self);
            } else {
                draw_waiting_for_players(self);
            }
        } else {
            draw_waiting_for_players(self);
        }
    }
}
