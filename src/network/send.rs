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

pub fn send_card_cancel(card_index: usize, card: &Card) {
    log!("🚀 [SEND] Card cancel: {}", card.name);
    let msg = ClientToServer::CancelSelectCard { card_index, card: card.clone() };
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

pub fn send_tile_rotation(tile_index: usize, clockwise: bool) {
    log!("🚀 [SEND] Tile rotation: {}", tile_index);
    let msg = ClientToServer::RotateTile { tile_index, clockwise };
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
