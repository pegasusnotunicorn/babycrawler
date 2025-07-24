mod game_channel;
use game_channel::GameChannel;
use game_channel::{ GameToClient as GCToClient, GameToServer as GCToServer };
mod game;
mod scene;

use crate::game::map::draw_board;
use crate::game::constants::{ GAME_PADDING, HAND_SIZE, MAP_SIZE, GAME_BG_COLOR };
use crate::game::input::handle_input;
use crate::game::map::{ Player, PlayerId };
use crate::game::map::Tile;
use crate::game::ui::{ draw_turn_label, draw_waiting_for_players, draw_menu_screen };
use crate::game::cards::draw_play_area;
use crate::game::animation::update_spring_back_dragged_card;
use crate::game::cards::draw_hand;
use crate::game::cards::card::Card;
use crate::game::debug::draw_debug;
use crate::game::animation::update_tile_rotation_animations;

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

#[derive(
    Clone,
    Debug,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub struct DraggedCard {
    pub card: Card,
    pub hand_index: usize,
    pub offset: (i32, i32), // pointer - card origin at drag start
    pub pos: (f32, f32), // current position (for spring-back)
    pub velocity: (f32, f32), // for spring-back
    pub dragging: bool,
    pub released: bool,
}

fn fill_with_dummies(vec: &mut Vec<Card>, size: usize) {
    while vec.len() < size {
        vec.push(Card::dummy_card());
    }
    vec.truncate(size);
}

#[turbo::game]
pub struct GameState {
    pub frame: usize,
    pub tiles: Vec<Tile>,
    pub players: Vec<Player>,
    pub selected_card: Option<Card>,
    pub scene: Scene, // Track current scene (menu or game)
    pub user: String, // This client's user id
    pub online_now: usize, // Number of users online (matchmaking)
    pub in_lobby: Vec<String>, // Users in the current game lobby
    pub multiplayer_scene: Option<MultiplayerScene>,
    pub current_turn_player_id: Option<String>, // Track whose turn it is (None if no turn)
    pub debug: bool,
    pub user_id_to_player_id: HashMap<String, PlayerId>,
    pub dragged_card: Option<DraggedCard>,
    pub play_area: Vec<Card>,
}

impl GameState {
    pub fn new() -> Self {
        let debug = true; // Hardcoded for development

        Self {
            frame: 0,
            tiles: Vec::new(),
            players: Vec::new(),
            selected_card: None,
            scene: Scene::Menu, // Start in menu scene
            user: String::new(), // Will be set on connect
            online_now: 0,
            in_lobby: Vec::new(),
            multiplayer_scene: None,
            current_turn_player_id: None,
            debug,
            user_id_to_player_id: HashMap::new(),
            dragged_card: None,
            play_area: {
                let mut play_area = Vec::new();
                fill_with_dummies(&mut play_area, HAND_SIZE);
                play_area
            },
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
        // Spring-back for dragged card (animation logic)
        if update_spring_back_dragged_card(self) {
            self.dragged_card = None;
        }
        // Update tile rotation animations
        update_tile_rotation_animations(self, 1.0 / 60.0);
        match self.scene {
            Scene::Menu => self.update_menu(),
            Scene::Game => self.update_game(),
        }
        draw_debug(self);
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
                    GCToClient::BoardState { tiles, players } => {
                        log!("Board state received: {:?}", tiles.len());
                        self.tiles = tiles;
                        self.players = players;
                    }
                    // handle other variants if needed
                }
            }

            self.draw_game();

            let is_my_turn = self.is_my_turn();
            if is_my_turn {
                let pointer = mouse::screen();
                let pointer_xy = (pointer.x, pointer.y);
                handle_input(self, &pointer, pointer_xy);

                if self.debug {
                    let gp = gamepad::get(0);
                    if gp.a.just_pressed() {
                        let _ = conn.send(&GCToServer::EndTurn);
                        log!("End turn sent");
                    }
                }
            }
        }
    }

    fn draw_menu(&self) {
        draw_menu_screen();
    }

    fn draw_game(&self) {
        clear(GAME_BG_COLOR);
        let (_canvas_width, _canvas_height, tile_size, offset_x, offset_y) =
            self.get_board_layout(false);
        draw_board(self, self.frame as f64, tile_size, offset_x, offset_y);
        draw_play_area(self, self.frame as f64);

        if let Some(player) = self.get_local_player() {
            draw_hand(self, &player.hand, &self.selected_card, self.frame as f64);
            draw_turn_label(self.is_my_turn(), self);
        } else {
            draw_waiting_for_players(self);
        }

        draw_debug(self);
    }
}
