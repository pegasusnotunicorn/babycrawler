mod game_channel;
use game_channel::game_server::GameChannel;
use game_channel::game_server::{ GameToClient as GCToClient, GameToServer as GCToServer };
mod game;
mod scene;

use crate::game::board::draw_board;
use crate::game::card::Card;
use crate::game::constants::*;
use crate::game::hand::draw_hand;
use crate::game::input::handle_input;
use crate::game::player::{ Player, PlayerId };
use crate::game::tile::{ Direction, Tile };
use crate::game::ui::{ draw_turn_label, draw_waiting_for_players, draw_menu_screen };

use turbo::{ bounds, * };
use turbo::os;
use turbo::gamepad;
use scene::{ Scene, MultiplayerScene };

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
    pub selected_cards: Vec<Card>,
    pub scene: Scene, // Track current scene (menu or game)
    // Multiplayer state
    pub user: String, // This client's user id
    pub online_now: usize, // Number of users online (matchmaking)
    pub in_lobby: Vec<String>, // Users in the current game lobby
    pub multiplayer_scene: Option<MultiplayerScene>,
    pub current_turn_player_id: Option<String>, // Track whose turn it is (None if no turn)
    pub debug: bool,
    pub user_id_to_player_id: HashMap<String, PlayerId>,
}

impl GameState {
    pub fn new() -> Self {
        let debug = true; // Hardcoded for development

        let mut tiles = vec![];
        for _ in 0..MAP_SIZE * MAP_SIZE {
            let mut entrances = vec![];
            for dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                if random::bool() == true {
                    entrances.push(*dir);
                }
            }
            tiles.push(Tile::new(entrances));
        }

        let mut players = vec![
            Player::new(PlayerId::Player1, 0, 0),
            Player::new(PlayerId::Player2, MAP_SIZE - 1, MAP_SIZE - 1)
        ];
        for player in &mut players {
            for _ in 0..HAND_SIZE {
                player.hand.push(Card::random());
            }
        }

        Self {
            frame: 0,
            tiles,
            players,
            selected_cards: vec![],
            scene: Scene::Menu, // Start in menu scene
            user: String::new(), // Will be set on connect
            online_now: 0,
            in_lobby: Vec::new(),
            multiplayer_scene: None,
            current_turn_player_id: None,
            debug,
            user_id_to_player_id: HashMap::new(),
        }
    }

    /// Helper to get the current turn player by ID (returns Option<&Player>)
    fn get_turn_player(&self) -> Option<&Player> {
        let id = self.current_turn_player_id.as_deref()?;
        self.players.iter().find(|p| p.id.to_string() == id)
    }

    /// Helper to get the current turn player as mutable
    fn get_turn_player_mut(&mut self) -> Option<&mut Player> {
        let id = self.current_turn_player_id.as_deref()?;
        self.players.iter_mut().find(|p| p.id.to_string() == id)
    }

    /// Returns true if it's this user's turn
    fn is_my_turn(&self) -> bool {
        self.current_turn_player_id.as_deref().map_or(false, |id| self.user == id)
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

    /// Returns (canvas_width, canvas_height, tile_size, offset_x, offset_y) for the board layout
    pub fn get_board_layout(&self, padded: bool) -> (u32, u32, u32, u32, u32) {
        let canvas_width = bounds::screen().w() - (if padded { GAME_PADDING * 2 } else { 0 });
        let canvas_height = bounds::screen().h();
        let tile_size = canvas_width / (MAP_SIZE as u32);
        let offset_x = canvas_width / 2 - (tile_size * (MAP_SIZE as u32)) / 2;
        let offset_y = 0;
        (canvas_width, canvas_height, tile_size, offset_x, offset_y)
    }

    pub fn update(&mut self) {
        clear(GAME_BG_COLOR);
        self.frame += 1;
        match self.scene {
            Scene::Menu => self.update_menu(),
            Scene::Game => self.update_game(),
        }
        self.draw_debug();
    }

    fn update_menu(&mut self) {
        self.draw_menu();
        let gp = gamepad::get(0);
        if gp.start.just_pressed() {
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
        if self.multiplayer_scene.is_none() {
            self.multiplayer_scene = Some(MultiplayerScene::MainMenu);
        }
        if let Some(conn) = GameChannel::subscribe("GLOBAL") {
            while let Ok(msg) = conn.recv() {
                match msg {
                    GCToClient::Turn { player_id } => {
                        log!("Turn received: {}", player_id);
                        self.current_turn_player_id = Some(player_id);
                    }
                    GCToClient::ConnectedUsers { users } => {
                        self.in_lobby = users.clone();
                        self.user_id_to_player_id.clear();
                        if self.in_lobby.len() >= 2 {
                            self.user_id_to_player_id.insert(
                                self.in_lobby[0].clone(),
                                PlayerId::Player1
                            );
                            self.user_id_to_player_id.insert(
                                self.in_lobby[1].clone(),
                                PlayerId::Player2
                            );
                        }
                    }
                    // handle other variants if needed
                }
            }

            // Draw the full game for both players
            self.draw_game();

            let is_my_turn = self.is_my_turn();
            if is_my_turn {
                // Only allow input for the active player
                let pointer = mouse::screen();
                let pointer_xy = (pointer.x, pointer.y);
                handle_input(self, &pointer, pointer_xy);
                let gp = gamepad::get(0);
                if gp.a.just_pressed() {
                    let _ = conn.send(&GCToServer::EndTurn);
                    log!("EndTurn");
                }
            } else {
                // Input is disabled for the non-active player
                // (No handle_input or end turn allowed)
            }
        }
    }

    fn draw_menu(&self) {
        draw_menu_screen();
    }

    fn draw_debug(&self) {
        if !self.debug {
            return;
        }

        // draw user id
        let id = &self.user;
        let debug_str = format!("Player ID: {}", id);
        text!(&debug_str, x = 8, y = 8, font = "medium", color = 0xffffffff);

        // draw current turn player id
        let current_turn_player_id = self.current_turn_player_id.as_deref().unwrap_or("None");
        let debug_str = format!("Current Turn Player ID: {}", current_turn_player_id);
        text!(&debug_str, x = 8, y = 8 + 15, font = "medium", color = 0xffffffff);

        // draw selected cards
        let selected_cards = &self.selected_cards;
        let debug_str = format!("Selected Cards: {}", selected_cards.len());
        text!(&debug_str, x = 8, y = 8 + 30, font = "medium", color = 0xffffffff);

        // draw get_local_player
        let local_player = self.get_local_player();
        let debug_str = format!("Local Player: {:?}", local_player);
        text!(&debug_str, x = 8, y = 8 + 45, font = "medium", color = 0xffffffff);
    }

    fn draw_game(&self) {
        let (_canvas_width, _canvas_height, tile_size, offset_x, offset_y) =
            self.get_board_layout(false);
        draw_board(self, self.frame as f64, tile_size, offset_x, offset_y);

        if let Some(player) = self.get_local_player() {
            draw_hand(self, &player.hand, &self.selected_cards, tile_size, self.frame as f64);
            draw_turn_label(self.is_my_turn(), self);
        } else {
            draw_waiting_for_players(self);
        }

        self.draw_debug();
    }
}
