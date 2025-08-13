use turbo::*;
use borsh::BorshSerialize;
use serde::Serialize;
use crate::server::{ ServerToClient, CurrentTurn };
use crate::game::map::{ Tile, Player };
use crate::game::cards::card::Card;

pub fn broadcast_generic<T: Serialize + BorshSerialize>(msg: T) {
    if let Err(e) = os::server::channel::broadcast(msg) {
        log!("[GameChannel] Error broadcasting message: {e}");
    }
}

pub fn broadcast_turn(
    players: &[String],
    current_turn_index: usize,
    current_turn: &mut Option<CurrentTurn>,
    board_tiles: &[Tile],
    board_players: &[Player]
) {
    log!("[GameChannel] Broadcasting turn");
    if let Some(user_id) = players.get(current_turn_index) {
        // Update current_turn with the new player
        *current_turn = Some(CurrentTurn {
            player_id: user_id.clone(),
            selected_card: None,
            selected_card_index: 0,
            confirmed_cards_count: 0,
        });
        broadcast_board_state(board_tiles, board_players, current_turn);
    }
}

pub fn broadcast_board_state(
    board_tiles: &[Tile],
    board_players: &[Player],
    current_turn: &Option<CurrentTurn>
) {
    log!("[GameChannel] Broadcasting board state");
    broadcast_generic(ServerToClient::BoardState {
        tiles: board_tiles.to_vec(),
        players: board_players.to_vec(),
        current_turn: current_turn.clone(),
    });
}

pub fn broadcast_card_cancelled(card: &Card, player_id: &str) {
    log!("[GameChannel] Broadcasting card cancelled: card={:?}, player={}", card.name, player_id);
    broadcast_generic(ServerToClient::CardCancelled {
        card: card.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_card_confirmed(card: &Card, player_id: &str) {
    log!("[GameChannel] Broadcasting card confirmed: card={:?}, player={}", card, player_id);
    broadcast_generic(ServerToClient::CardConfirmed {
        card: card.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_tile_rotation(tile_index: usize, tile: &Tile, player_id: &str) {
    log!(
        "[GameChannel] Broadcasting tile rotation: index={}, rotation={}",
        tile_index,
        tile.current_rotation
    );
    broadcast_generic(ServerToClient::TileRotated {
        tile_index,
        tile: tile.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_player_moved(player_id: &str, new_position: (usize, usize), is_canceled: bool) {
    log!(
        "[GameChannel] Broadcasting player moved: player={}, new_position={:?}, is_canceled={}",
        player_id,
        new_position,
        is_canceled
    );
    broadcast_generic(ServerToClient::PlayerMoved {
        player_id: player_id.to_string(),
        new_position,
        is_canceled,
    });
}

pub fn broadcast_tiles_swapped(tile_index_1: usize, tile_index_2: usize) {
    log!("[GameChannel] Broadcasting tiles swapped: {} <-> {}", tile_index_1, tile_index_2);
    broadcast_generic(ServerToClient::TilesSwapped {
        tile_index_1,
        tile_index_2,
    });
}

pub fn broadcast_fireball_shot(
    player_id: &str,
    target_tile: usize,
    direction: &crate::game::map::tile::Direction
) {
    log!(
        "[GameChannel] Broadcasting fireball shot: player={}, target_tile={}, direction={:?}",
        player_id,
        target_tile,
        direction
    );
    broadcast_generic(ServerToClient::FireballShot {
        player_id: player_id.to_string(),
        tile_index: target_tile,
        direction: direction.clone(),
    });
}

pub fn broadcast_fireball_hit_result(shooter_id: &str, target_player_id: &str, damage_dealt: &u32) {
    log!(
        "[GameChannel] Broadcasting fireball hit result: player={}, target={:?}, damage={}",
        shooter_id,
        target_player_id,
        damage_dealt
    );
    broadcast_generic(ServerToClient::FireballHit {
        player_id: shooter_id.to_string(),
        target_id: target_player_id.to_string(),
        damage_dealt: *damage_dealt,
    });
}

pub fn broadcast_game_over(winner_id: &str, loser_id: &str) {
    broadcast_generic(ServerToClient::GameOver {
        winner_id: winner_id.to_string(),
        loser_id: loser_id.to_string(),
    });
}
