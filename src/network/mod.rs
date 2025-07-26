use turbo::*;
use serde::{ Serialize, Deserialize };
use crate::game::cards::card::Card;

pub mod send;
pub mod receive;

#[derive(borsh::BorshSerialize, borsh::BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub enum ClientToServer {
    EndTurn,
    SelectCard {
        card_index: usize,
        card: Card,
    },
    CancelSelectCard {
        card_index: usize,
    },
}
