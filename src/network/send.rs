use turbo::*;
use crate::game::cards::card::Card;
use crate::server::GameChannel;
use crate::game::constants::GAME_CHANNEL;
use super::ClientToServer;

pub fn send_card_selection(card_index: usize, card: &Card) {
    log!("🚀 [SEND] Card selection: {}", card.name);
    let msg = ClientToServer::SelectCard { card_index, card: card.clone() };
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}

pub fn send_card_cancel(card_index: usize) {
    log!("🚀 [SEND] Card cancel: {}", card_index);
    let msg = ClientToServer::CancelSelectCard { card_index };
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}

pub fn send_end_turn() {
    log!("🚀 [SEND] End turn");
    let msg = ClientToServer::EndTurn;
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}
