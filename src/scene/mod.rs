// Scene and mode enums for the game
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
