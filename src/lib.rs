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
use crate::game::tile::{ clear_highlights, Direction, Tile };
use crate::game::ui::draw_turn_label;

use turbo::{ bounds, os::server::fs, * };
use turbo::os;
use turbo::gamepad;
use scene::{ GameMode, Scene, MultiplayerScene };

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
    pub mode: GameMode, // Track current play mode
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
            mode: GameMode::Singleplayer, // Default to singleplayer for now
            scene: Scene::Menu, // Start in menu scene
            user: PlayerId::Player1.to_string(), // Set to Player1 for singleplayer
            online_now: 0,
            in_lobby: Vec::new(),
            multiplayer_scene: None,
            current_turn_player_id: Some(PlayerId::Player1.to_string()),
            debug,
            user_id_to_player_id: HashMap::new(),
        }
    }

    /// Helper to get the current player by ID (returns Option<&Player>)
    fn current_player(&self) -> Option<&Player> {
        let id = self.current_turn_player_id.as_deref()?;
        self.players.iter().find(|p| p.id.to_string() == id)
    }

    /// Helper to get the current player as mutable
    fn current_player_mut(&mut self) -> Option<&mut Player> {
        let id = self.current_turn_player_id.as_deref()?;
        self.players.iter_mut().find(|p| p.id.to_string() == id)
    }

    fn draw_menu(&self) {
        clear(0x222222ff);
        let canvas_bounds = bounds::screen();
        let canvas_width = canvas_bounds.w();
        let canvas_height = canvas_bounds.h();
        let menu_items = [
            "Press Z for Local Play (Singleplayer)",
            "Press SPACE for Online Play (Multiplayer)",
        ];
        let font_height = 30; // Approximate height for "large" font
        let total_height =
            (menu_items.len() as u32) * font_height + ((menu_items.len() as u32) - 1) * 10;
        let start_y = canvas_height / 2 - total_height / 2;
        for (i, item) in menu_items.iter().enumerate() {
            let text_width = (item.len() as u32) * 12; // Approximate width per char for "large" font
            let x = canvas_width / 2 - text_width / 2;
            let y = start_y + (i as u32) * (font_height + 10);
            text!(item, x = x, y = y, font = "large", color = 0xffffffff);
        }
    }

    pub fn update(&mut self) {
        match self.scene {
            Scene::Menu => self.update_menu(),
            Scene::Game => self.update_game(),
        }
        if self.debug {
            self.draw_debug();
        }
    }

    fn update_menu(&mut self) {
        self.draw_menu();
        let gp = gamepad::get(0);
        if gp.a.just_pressed() {
            self.mode = GameMode::Singleplayer;
            self.scene = Scene::Game;
            self.user = PlayerId::Player1.to_string(); // Ensure user is Player1 in singleplayer
            self.user_id_to_player_id.clear();
            self.user_id_to_player_id.insert(self.user.clone(), PlayerId::Player1);
        } else if gp.start.just_pressed() {
            self.mode = GameMode::Multiplayer;
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
        match self.mode {
            GameMode::Singleplayer => {
                self.frame += 1;
                let mut clicked_card: Option<Card> = None;
                self.draw_game_with_callback(&mut clicked_card);
                if let Some(card) = clicked_card {
                    Card::toggle_in(&mut self.selected_cards, &card);
                    clear_highlights(&mut self.tiles);
                    if self.selected_cards.len() == 1 {
                        let card = &self.selected_cards[0];
                        let player = self.current_player().unwrap();
                        card.effect.highlight_tiles(player.position, &mut self.tiles);
                    }
                }
                let pointer = mouse::screen();
                let pointer_xy = (pointer.x, pointer.y);
                let canvas_width = bounds::screen().w();
                let tile_size = canvas_width / (MAP_SIZE as u32);
                let offset_x = canvas_width / 2 - (tile_size * (MAP_SIZE as u32)) / 2;
                let offset_y = 0;
                handle_input(self, &pointer, pointer_xy, tile_size, offset_x, offset_y);
                fs::write("state", self).ok();
            }
            GameMode::Multiplayer => {
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
                    let is_my_turn = self.current_turn_player_id
                        .as_deref()
                        .map_or(false, |id| self.user == id);
                    // Draw the full game for both players
                    let mut clicked_card: Option<Card> = None;
                    self.draw_game_with_callback(&mut clicked_card);
                    if is_my_turn {
                        // Only allow input for the active player
                        let pointer = mouse::screen();
                        let pointer_xy = (pointer.x, pointer.y);
                        let canvas_width = bounds::screen().w();
                        let tile_size = canvas_width / (MAP_SIZE as u32);
                        let offset_x = canvas_width / 2 - (tile_size * (MAP_SIZE as u32)) / 2;
                        let offset_y = 0;
                        handle_input(self, &pointer, pointer_xy, tile_size, offset_x, offset_y);
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
        }
    }

    fn draw_debug(&self) {
        let id = &self.user;
        let debug_str = format!("Player ID: {}", id);
        text!(&debug_str, x = 8, y = 8, font = "medium", color = 0xffffffff);

        let current_turn_player_id = self.current_turn_player_id.as_deref().unwrap_or("None");
        let debug_str = format!("Current Turn Player ID: {}", current_turn_player_id);
        text!(&debug_str, x = 8, y = 8 + 15, font = "medium", color = 0xffffffff);
    }

    fn draw_game_with_callback(&self, clicked_card: &mut Option<Card>) {
        clear(GAME_BG_COLOR);
        let canvas_width = bounds::screen().w();
        let tile_size = canvas_width / (MAP_SIZE as u32);
        let offset_x = canvas_width / 2 - (tile_size * (MAP_SIZE as u32)) / 2;
        let offset_y = 0;
        draw_board(self, self.frame as f64, tile_size, offset_x, offset_y);
        let player_id = self.user_id_to_player_id.get(&self.user);
        let player = player_id.and_then(|pid| self.players.iter().find(|p| &p.id == pid));
        if let Some(player) = player {
            draw_hand(&player.hand, &self.selected_cards, tile_size, self.frame as f64, |card| {
                *clicked_card = Some(card.clone());
            });
            let is_my_turn = self.current_turn_player_id
                .as_deref()
                .map_or(false, |id| self.user == id);
            draw_turn_label(is_my_turn, tile_size);
        }
        if self.debug {
            self.draw_debug();
        }
    }
}
