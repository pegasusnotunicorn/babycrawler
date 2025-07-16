use turbo::*;

// Scene enum controls the primary state machine of the game
#[turbo::serialize]
enum Scene {
    MainMenu,
    Lobby { id: String }, // Lobby and Game scenes store the channel id the user subscribes to when in the scenes
    Game { id: String },
    Disconnected { player: String }, // Disconnected scene stores the player id of the user that disconnected
}
impl std::fmt::Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scene::MainMenu => write!(f, "MainMenu"),
            Scene::Lobby { id } => write!(f, "Lobby"),
            Scene::Game { id } => write!(f, "Game"),
            Scene::Disconnected { player } => write!(f, "Disconnected"),
        }
    }
}

// TURBO GAME LOOP
#[turbo::game]
struct GameState {
    scene: Scene, // State machine
    user: String, // CLient user id
    online_now: usize, // Local store of matchmaking channel's online user count
    in_lobby: Vec<String>, // Local store of game channel's connected users
}
impl GameState {
    pub fn new() -> Self {
        Self { 
            scene: Scene::MainMenu,
            user: "NO_ID".to_string(), // Assign on program initialization to require user authentication on Turbo OS
            online_now: 0,
            in_lobby: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        let gp = gamepad::get(0); // Local store of gamepad input
        // Prevent the user from continuing if not logged in / network error
        if self.user == "NO_ID" {
            self.user = os::client::user_id().unwrap_or_else(|| "NO_ID".to_string());
        } else {
            // Always subscribe to the global matchmaking channel
            if let Some(mm_conn) = matchmaking::MatchmakingChannel::subscribe("GLOBAL") {
                // Match the current scene for update logic
                match &self.scene {
                // Main menu
                    Scene::MainMenu => {
                        // Send message to channel to search for a game 
                        if gp.start.just_pressed() {
                            let _ = mm_conn.send(&ClientMsg::FindGame);
                            log!("find game sent");
                        }
                        // Recieve messages from the matchmaking channel
                        while let Ok(server_msg) = mm_conn.recv() {
                            log!("message recieved");
                            match server_msg {
                                // Channel broadcasts connected users
                                ServerMsg::ConnectedUsers { users } => {
                                    self.online_now = users.len(); // Store the count of online users to display in the main menu
                                    log!("online now count changed: {:#?}", users.len());
                                }
                                // Matchmaking channel directs user to join a specific game channel
                                ServerMsg::JoinChannel { id } => {
                                    self.scene = Scene::Lobby { id: id.clone() }; // Pass the id from the matchmaking channel's ServerMsg to the lobby scene
                                    log!("joining channel {:#?}", id);
                                }
                                _ => {}
                            }
                        }
                    }
                // Lobby and Game scenes
                    // Both scenes use the same logic to handle the game channel, for now
                    Scene::Lobby { id } | Scene::Game { id } => {
                        let lobby_id = id.clone(); // Cloned instance for dereferencing later
                        // Satisfy recieving messages from the matchmaking channel, even if we don't do anything with them
                        while let Ok(_) = mm_conn.recv() {}
                        // Subscribe to the actual game channel with id passed from the matchmaking channel stored in the current Scene enum
                        if let Some(conn) = matchmaking::GameChannel::subscribe( id ) {
                            // Leave game button
                            if gp.a.just_pressed() {
                                // If this user is the host of the game channel, send messages to close the lobby
                                if &self.user == id {
                                    let _ = mm_conn.send(&ClientMsg::CloseLobby);
                                    let _ = conn.send(&ClientMsg::CloseLobby);
                                }
                                self.scene = Scene::Disconnected { player: os::client::user_id().unwrap_or("NO_ID".to_string()) };
                                log!("leave game channel");
                            }
                            // Recieve messages from the game channel
                            while let Ok(server_msg) = conn.recv() {
                                log!("message recieved");
                                match server_msg {
                                    // Channel broadcasts list of connected users
                                    ServerMsg::ConnectedUsers { users } => {
                                        self.in_lobby = users;
                                        log!("in lobby count changed");
                                    }
                                    // Channel broadcasts start of game
                                    ServerMsg::StartGame => {
                                        let _ = mm_conn.send(&ClientMsg::CloseLobby);
                                        self.scene = Scene::Game { id: lobby_id.clone() };
                                        log!("game starting");
                                    }
                                    // Channel broadcasts a player has left the lobby after the game has started
                                    ServerMsg::PlayerLeave { player } => {
                                        self.scene = Scene::Disconnected { player: player.clone() };
                                        log!("disconnected player: {:#?}", player);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                // Disconnected
                    Scene::Disconnected { player: _player } => {
                        self.in_lobby = Vec::new();
                        if gp.a.just_pressed() || gp.start.just_pressed() {
                            self.scene = Scene::MainMenu;
                        }
                    }
                }   
            }
        }

        self.draw();
    }
pub fn draw(&mut self) {
    let (cam_x, cam_y, _) = camera::xyz();

    match &self.scene {
        Scene::MainMenu => {
            clear(0x222034ff);
            if self.user != "NO_ID" {
                text!(
                    "Press SPACE for FIRST AVAILABLE MATCH",
                    xy = (cam_x - 90.0, cam_y - 10.0),
                );
                text!(
                    "ONLINE NOW: {:#?}", self.online_now;
                    xy = (cam_x - 30.0, cam_y + 50.0),
                );
            } else {
                text!(
                    "NETWORK ERROR. NOT LOGGED IN.",
                    xy = (cam_x - 72.0, cam_y - 10.0),
                );
            }
        }

        Scene::Lobby { id } | Scene::Game { id } => {
            if let Scene::Lobby { .. } = self.scene {
                clear(0x306082ff);
                let mut search = String::from("SEARCHING");
                for _ in 0..(turbo::time::tick() / 30 % 4) {
                    search.push('.');
                }
                text!(
                    "{}", search;
                    xy = (cam_x - search.len() as f32 * 2.0, cam_y - 10.0),
                );
                text!(
                    "Press Z to cancel",
                    xy = (cam_x - 40.0, cam_y + 50.0),
                );
            } else {
                clear(0x4b692fff);
                text!(
                    "Press Z to abandon game",
                    xy = (cam_x - 55.0, cam_y + 50.0),
                );
            }

            let mut n = id.clone();
            let mut offset = 0.0;
            if n == self.user {
                n = "YOUR".to_string();
                offset = 10.0;
            } else {
                n.truncate(6);
                n.push_str("'s");
            }
            text!(
                "{} GAME", n;
                xy = (cam_x - 32.0 + offset, cam_y - 30.0),
            );
            text!(
                "In game:",
                xy = (cam_x - 100.0, cam_y + 70.0),
            );
            let mut i = 0;
            for user in self.in_lobby.iter() {
                let mut truncated_user = user.clone();
                truncated_user.truncate(6);
                text!(
                    "{}",
                    truncated_user;
                    xy = (cam_x - 100.0, cam_y + 80.0 + i as f32 * 10.0),
                );
                i += 1;
            }
        }

        Scene::Disconnected { player } => {
            clear(0xac3232ff);
            let mut truncated_player = player.clone();
            truncated_player.truncate(6);
            text!(
                "{} DISCONNECTED",
                truncated_player;
                xy = (cam_x - 48.0, cam_y - 30.0),
            );
            text!(
                "Press START to return to the main menu",
                xy = (cam_x - 94.0, cam_y - 10.0),
            );
        }
    }

    let mut id = self.user.clone();
    id.truncate(6);
    text!("user: {}", id; xy = (1.0, 1.0));
    text!("scene: {}", self.scene; xy = (1.0, 9.0));
}

}

// Client and Server message enums are how we pass data between the client and server
#[turbo::serialize]
pub enum ClientMsg {
    FindGame,
    CloseLobby,
}
#[turbo::serialize]
pub enum ServerMsg {
    ConnectedUsers { users: Vec<String> }, // Used to broadcast connected users in a channel
    JoinChannel { id: String }, // Used to signal to a specific client to connect to a specific game channel
    StartGame, // Used to signal to connected clients that the game is starting
    PlayerLeave { player: String }, // Used 
}

#[turbo::program]
pub mod matchmaking {
    use super::*;
    use turbo::os::server::channel::ChannelSettings;

    // This channel facilitates matchmaking
    #[turbo::channel(name = "matchmaking")]
    pub struct MatchmakingChannel {
        online_now: Vec<String>, // Channel's store of connected users
        open_game: Option<String>, // Channel's store of the open game, if any
    }
    // Implementation of ChannelHandler trait required for Turbo channels, defines how the channel behaves server-side
    impl ChannelHandler for MatchmakingChannel {
        // Req. Define the channel's send and receive message types
        type Send = ServerMsg;
        type Recv = ClientMsg;
        // Req. Creates a new instance of the channel
        fn new() -> Self {
            MatchmakingChannel {
                online_now: Vec::new(),
                open_game: None,
            }
        }
        // Req. Handles opening the channel
        fn on_open(&mut self, settings: &mut ChannelSettings) {
            settings.set_interval(16 * 60 * 10);
        }
        // Req. Handles new connections to the channel
        fn on_connect(&mut self, user_id: &str) {
            // If the user is not already registered in the channel, store them in the players list
            if !self.online_now.contains(&user_id.to_string()) {
                self.online_now.push(user_id.to_string());
            }
            // Broadcast the connected users to all clients in the channel
            os::server::channel::broadcast(ServerMsg::ConnectedUsers { users: self.online_now.clone() });
        }
        // Req. Handles disconnections from the channel
        fn on_disconnect(&mut self, user_id: &str) {
            // Remove the user from the stored players list
            self.online_now.retain_mut(|p| p != user_id);
            // Broadcast the connected users to all clients in the channel
            os::server::channel::broadcast(ServerMsg::ConnectedUsers { users: self.online_now.clone() });
        }
        // Req. Handles recieving messages from connected clients
        fn on_data(&mut self, user_id: &str, data: Self::Recv) {
            match data {
                // Client sends a message to request connecting to a game
                ClientMsg::FindGame => {
                    // If there is an open game, send the user to that game's channel id
                    if let Some(open_game) = &self.open_game {
                        os::server::channel::send(user_id, ServerMsg::JoinChannel { id: open_game.to_string() });
                    } 
                    // Otherwise, create a new game channel with the user's id
                    else {
                        self.open_game = Some(user_id.to_string());
                        os::server::channel::send(user_id, ServerMsg::JoinChannel { id: user_id.to_string() });
                    }
                }
                ClientMsg::CloseLobby => {
                    self.open_game = None;
                }
            }
        }
    }

    // This channel facilitates lobbied players, and would be expanded to include game messaging
    #[turbo::channel(name = "game")]
    pub struct GameChannel {
        players: Vec<String>,
        max_players: usize,
        game_started: bool,
    }
    // Implementation of ChannelHandler trait required for Turbo channels, defines how the channel behaves server-side
    impl ChannelHandler for GameChannel {
        // Req. Define the channel's send and receive message types
        type Send = ServerMsg;
        type Recv = ClientMsg;
        // Req. Creates a new instance of the channel
        fn new() -> Self {
            GameChannel {
                players: Vec::new(),
                max_players: 2,
                game_started: false,
            }
        }
        // Req. Handles opening the channel
        fn on_open(&mut self, settings: &mut ChannelSettings) {
            settings.set_interval(16 * 60 * 10);
        }
        // Req. Handles new connections to the channel
        fn on_connect(&mut self, user_id: &str) {
            // If the user is not already registered in the channel, store them in the players list
            if !self.players.contains(&user_id.to_string()) {
                self.players.push(user_id.to_string());
            }
            // Broadcast the connected users to all clients in the channel
            os::server::channel::broadcast(ServerMsg::ConnectedUsers { users: self.players.clone() });
            // Start the game if the max players is reached
            if self.players.len() >= self.max_players {
                self.game_started = true;
                os::server::channel::broadcast(ServerMsg::StartGame);
            }
        }
        // Req. Handles disconnections from the channel
        fn on_disconnect(&mut self, user_id: &str) {
            // Remove the user from the stored players list
            self.players.retain_mut(|p| p != user_id);
            // Broadcast the connected users to all clients in the channel
            os::server::channel::broadcast(ServerMsg::ConnectedUsers { users: self.players.clone() });
            // If the game has started, broadcast that the player has left
            if self.game_started {
                os::server::channel::broadcast(ServerMsg::PlayerLeave { player: user_id.to_string() });
            }
        }
        // Req. Handles recieving messages from connected clients -- Build your specific (game -> server -> game) logic here
        fn on_data(&mut self, user_id: &str, data: Self::Recv) {
            match data {
                // Client disconnects in the lobby, broadcast to all clients in the channel to leave
                ClientMsg::CloseLobby => {
                    os::server::channel::broadcast(ServerMsg::PlayerLeave { player: user_id.to_string() });
                }
                _ => {}
            }
        }
    }
}