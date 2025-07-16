// Networking message types and matchmaking module
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
