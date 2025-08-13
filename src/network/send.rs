use turbo::*;
use crate::game::cards::card::Card;
use crate::server::GameChannel;
use crate::game::constants::GAME_CHANNEL;
use super::ClientToServer;

pub fn send_card_selection(hand_index: usize) {
    log!("🚀 [SEND] Card selection at hand index: {}", hand_index);
    let msg = ClientToServer::SelectCard { hand_index };
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}

pub fn send_card_cancel(hand_index: usize) {
    log!("🚀 [SEND] Card cancel at hand index: {}", hand_index);
    let msg = ClientToServer::CancelSelectCard { hand_index };
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

pub fn send_tile_rotation(tile_index: usize) {
    log!("🚀 [SEND] Rotate tile index: {}", tile_index);
    let msg = ClientToServer::RotateTile { tile_index };
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}

pub fn send_move(new_position: (usize, usize), is_canceled: bool) {
    log!("🚀 [SEND] Player move to: {:?} (canceled: {})", new_position, is_canceled);
    let msg = ClientToServer::MovePlayer { new_position, is_canceled };
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}

pub fn send_confirm_card(card: &Card) {
    log!("🚀 [SEND] Card confirmed: {}", card.name);
    let msg = ClientToServer::ConfirmCard { card: card.clone() };
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}

pub fn send_swap_tiles(tile_index_1: usize, tile_index_2: usize) {
    log!("🚀 [SEND] Swap tiles: {} <-> {}", tile_index_1, tile_index_2);
    let msg = ClientToServer::SwapTiles { tile_index_1, tile_index_2 };
    if let Some(conn) = GameChannel::subscribe(GAME_CHANNEL) {
        let _ = conn.send(&msg);
    }
}
