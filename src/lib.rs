mod board;
mod hand;
mod input;
mod player;
mod tile;
mod card;
mod card_effect;
mod constants;
mod ui;
mod turn;
mod util;

use crate::{
    board::draw_board,
    card::Card,
    constants::*,
    hand::draw_hand,
    input::handle_input,
    player::{ Player, PlayerId },
    tile::{ clear_highlights, Direction, Tile },
    ui::draw_turn_label,
};

use turbo::{ bounds, os::server::fs, * };
use turbo::os;
use turbo::gamepad;

// Add stubs for networking message types and matchmaking module
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ClientMsg {
    FindGame,
    CloseLobby,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ServerMsg {
    ConnectedUsers {
        users: Vec<String>,
    },
    JoinChannel {
        id: String,
    },
    StartGame,
    PlayerLeave {
        player: String,
    },
}

// Stub matchmaking module to allow code to compile
pub mod matchmaking {
    use super::{ ClientMsg, ServerMsg };
    pub struct MatchmakingChannel;
    pub struct GameChannel;
    impl MatchmakingChannel {
        pub fn subscribe(_id: &str) -> Option<Self> {
            None
        }
        pub fn send(&self, _msg: &ClientMsg) -> Result<(), ()> {
            Ok(())
        }
        pub fn recv(&self) -> Result<ServerMsg, ()> {
            Err(())
        }
    }
    impl GameChannel {
        pub fn subscribe(_id: &str) -> Option<Self> {
            None
        }
        pub fn send(&self, _msg: &ClientMsg) -> Result<(), ()> {
            Ok(())
        }
        pub fn recv(&self) -> Result<ServerMsg, ()> {
            Err(())
        }
    }
}

// Add GameMode enum to track play mode
#[derive(
    Debug,
    Clone,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub enum GameMode {
    Singleplayer,
    Multiplayer,
}

// Add Scene enum for menu/game state
#[derive(
    Debug,
    Clone,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub enum Scene {
    Menu,
    Game,
}

// Multiplayer scene state for multiplayer mode
#[derive(
    Debug,
    Clone,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub enum MultiplayerScene {
    MainMenu,
    Lobby {
        id: String,
    },
    Game {
        id: String,
    },
    Disconnected {
        player: String,
    },
}

#[turbo::game]
pub struct GameState {
    pub frame: usize,
    pub tiles: Vec<Tile>,
    pub players: Vec<Player>,
    pub current_turn: usize,
    pub selected_cards: Vec<Card>,
    pub mode: GameMode, // Track current play mode
    pub scene: Scene, // Track current scene (menu or game)
    // Multiplayer state
    pub user: String, // This client's user id
    pub online_now: usize, // Number of users online (matchmaking)
    pub in_lobby: Vec<String>, // Users in the current game lobby
    // Add multiplayer scene state
    pub multiplayer_scene: Option<MultiplayerScene>,
}

impl GameState {
    pub fn new() -> Self {
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
            current_turn: 0,
            selected_cards: vec![],
            mode: GameMode::Singleplayer, // Default to singleplayer for now
            scene: Scene::Menu, // Start in menu scene
            user: "NO_ID".to_string(),
            online_now: 0,
            in_lobby: Vec::new(),
            multiplayer_scene: None,
        }
    }

    pub fn update(&mut self) {
        match self.scene {
            Scene::Menu => self.update_menu(),
            Scene::Game => self.update_game(),
        }
    }

    fn update_menu(&mut self) {
        self.draw_menu();
        let gp = gamepad::get(0);
        if gp.a.just_pressed() {
            self.mode = GameMode::Singleplayer;
            self.scene = Scene::Game;
        } else if gp.start.just_pressed() {
            self.mode = GameMode::Multiplayer;
            self.scene = Scene::Game;
        }
    }

    fn update_game(&mut self) {
        match self.mode {
            GameMode::Singleplayer => {
                self.frame += 1;
                // Refactor draw_hand callback to avoid borrow issues
                let mut clicked_card: Option<Card> = None;
                self.draw_game_with_callback(&mut clicked_card);
                if let Some(card) = clicked_card {
                    Card::toggle_in(&mut self.selected_cards, &card);
                    clear_highlights(&mut self.tiles);
                    if self.selected_cards.len() == 1 {
                        let card = &self.selected_cards[0];
                        let player = &self.players[self.current_turn];
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
                if self.user == "NO_ID" {
                    self.user = os::client::user_id().unwrap_or_else(|| "NO_ID".to_string());
                }
                if self.multiplayer_scene.is_none() {
                    self.multiplayer_scene = Some(MultiplayerScene::MainMenu);
                }
                if let Some(mm_conn) = matchmaking::MatchmakingChannel::subscribe("GLOBAL") {
                    let scene = self.multiplayer_scene.take().unwrap();
                    match &scene {
                        MultiplayerScene::MainMenu => {
                            self.draw_multiplayer_main_menu();
                            let gp = gamepad::get(0);
                            if gp.start.just_pressed() {
                                let _ = mm_conn.send(&ClientMsg::FindGame);
                            }
                            let mut new_scene = MultiplayerScene::MainMenu;
                            while let Ok(server_msg) = mm_conn.recv() {
                                match server_msg {
                                    ServerMsg::ConnectedUsers { ref users } => {
                                        self.online_now = users.len();
                                    }
                                    ServerMsg::JoinChannel { ref id } => {
                                        new_scene = MultiplayerScene::Lobby { id: id.clone() };
                                    }
                                    _ => {}
                                }
                            }
                            self.multiplayer_scene = Some(new_scene);
                        }
                        MultiplayerScene::Lobby { id } | MultiplayerScene::Game { id } => {
                            let lobby_id = id.clone();
                            while let Ok(_) = mm_conn.recv() {}
                            let mut new_scene = match &scene {
                                MultiplayerScene::Lobby { .. } =>
                                    MultiplayerScene::Lobby { id: id.clone() },
                                MultiplayerScene::Game { .. } =>
                                    MultiplayerScene::Game { id: id.clone() },
                                _ => unreachable!(),
                            };
                            if let Some(conn) = matchmaking::GameChannel::subscribe(&id) {
                                let gp = gamepad::get(0);
                                if gp.a.just_pressed() {
                                    if &self.user == id {
                                        let _ = mm_conn.send(&ClientMsg::CloseLobby);
                                        let _ = conn.send(&ClientMsg::CloseLobby);
                                    }
                                    new_scene = MultiplayerScene::Disconnected {
                                        player: self.user.clone(),
                                    };
                                }
                                while let Ok(server_msg) = conn.recv() {
                                    match server_msg {
                                        ServerMsg::ConnectedUsers { ref users } => {
                                            self.in_lobby = users.clone();
                                        }
                                        ServerMsg::StartGame => {
                                            let _ = mm_conn.send(&ClientMsg::CloseLobby);
                                            new_scene = MultiplayerScene::Game {
                                                id: lobby_id.clone(),
                                            };
                                        }
                                        ServerMsg::PlayerLeave { ref player } => {
                                            new_scene = MultiplayerScene::Disconnected {
                                                player: player.clone(),
                                            };
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            self.draw_multiplayer_lobby(&id);
                            self.multiplayer_scene = Some(new_scene);
                        }
                        MultiplayerScene::Disconnected { player } => {
                            self.in_lobby.clear();
                            self.draw_multiplayer_disconnected(&player);
                            let gp = gamepad::get(0);
                            let new_scene = if gp.a.just_pressed() || gp.start.just_pressed() {
                                MultiplayerScene::MainMenu
                            } else {
                                MultiplayerScene::Disconnected { player: player.clone() }
                            };
                            self.multiplayer_scene = Some(new_scene);
                        }
                    }
                }
            }
        }
    }

    // Helper for draw_game to allow card click callback
    fn draw_game_with_callback(&self, clicked_card: &mut Option<Card>) {
        clear(GAME_BG_COLOR);
        let canvas_width = bounds::screen().w();
        let tile_size = canvas_width / (MAP_SIZE as u32);
        let offset_x = canvas_width / 2 - (tile_size * (MAP_SIZE as u32)) / 2;
        let offset_y = 0;
        draw_board(self, self.frame as f64, tile_size, offset_x, offset_y);
        draw_hand(
            &self.players[self.current_turn].hand,
            &self.selected_cards,
            tile_size,
            self.frame as f64,
            |card| {
                *clicked_card = Some(card.clone());
            }
        );
        draw_turn_label(self.current_turn, tile_size);
    }

    fn draw_menu(&self) {
        clear(0x222222ff);
        text!(
            "Press Z for Local Play (Singleplayer)",
            x = 40,
            y = 80,
            font = "large",
            color = 0xffffffff
        );
        text!(
            "Press SPACE for Online Play (Multiplayer)",
            x = 40,
            y = 110,
            font = "large",
            color = 0xffffffff
        );
    }

    fn draw_multiplayer_main_menu(&self) {
        clear(0x222034ff);
        if self.user != "NO_ID" {
            text!(
                "Press SPACE for FIRST AVAILABLE MATCH",
                x = 40,
                y = 80,
                font = "large",
                color = 0xffffffff
            );
            let online_now_text = format!("ONLINE NOW: {}", self.online_now);
            text!(&online_now_text, x = 40, y = 110, font = "large", color = 0xffffffff);
        } else {
            text!(
                "NETWORK ERROR. NOT LOGGED IN.",
                x = 40,
                y = 80,
                font = "large",
                color = 0xffffffff
            );
        }
    }

    fn draw_multiplayer_lobby(&self, id: &str) {
        clear(0x306082ff);
        let mut search = String::from("SEARCHING");
        for _ in 0..(turbo::time::tick() / 30) % 4 {
            search.push('.');
        }
        text!(&search, x = 40, y = 80, font = "large", color = 0xffffffff);
        text!("Press Z to cancel", x = 40, y = 110, font = "large", color = 0xffffffff);
        let in_lobby_text = format!("In lobby: {}", self.in_lobby.len());
        text!(&in_lobby_text, x = 40, y = 140, font = "large", color = 0xffffffff);
        let mut i = 0;
        for user in self.in_lobby.iter() {
            let truncated_user = user.chars().take(6).collect::<String>();
            text!(&truncated_user, x = 40, y = 170 + i * 20, font = "large", color = 0xffffffff);
            i += 1;
        }
    }

    fn draw_multiplayer_disconnected(&self, player: &str) {
        clear(0xac3232ff);
        let truncated_player = player.chars().take(6).collect::<String>();
        let disconnect_text = format!("{} DISCONNECTED", truncated_player);
        text!(&disconnect_text, x = 40, y = 80, font = "large", color = 0xffffffff);
        text!(
            "Press Z or SPACE to return to the main menu",
            x = 40,
            y = 110,
            font = "large",
            color = 0xffffffff
        );
    }
}
