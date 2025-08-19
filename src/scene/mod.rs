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
    GameOver {
        winner_ids: Vec<String>,
        loser_ids: Vec<String>,
    },
}
