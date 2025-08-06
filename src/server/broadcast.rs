use turbo::*;
use borsh::BorshSerialize;
use serde::Serialize;
use crate::server::{ ServerToClient, CurrentTurn };
use crate::game::map::{ Tile, Player };
use crate::game::cards::card::Card;

pub fn broadcast_generic<T: Serialize + BorshSerialize>(msg: T) {
    log!("[GameChannel] Attempting to broadcast message: {:?}", std::any::type_name::<T>());
    if let Err(e) = os::server::channel::broadcast(msg) {
        log!("[GameChannel] Error broadcasting message: {e}");
    } else {
        log!("[GameChannel] Successfully broadcasted message");
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
        });
        // Broadcast the updated board state which includes the current turn
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

pub fn broadcast_card_selected(card_index: usize, card: &Card, player_id: &str) {
    log!(
        "[GameChannel] Broadcasting card selected: index={}, card={:?}, player={}",
        card_index,
        card,
        player_id
    );
    broadcast_generic(ServerToClient::CardSelected {
        card_index,
        card: card.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_card_canceled(card_index: usize, card: &Card, player_id: &str) {
    log!("[GameChannel] Broadcasting card canceled: index={}, player={}", card_index, player_id);
    broadcast_generic(ServerToClient::CardCanceled {
        card_index,
        card: card.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_tile_rotation(tile_index: usize, clockwise: bool, player_id: &str) {
    log!("[GameChannel] Broadcasting tile rotation: index={}, clockwise={}", tile_index, clockwise);
    broadcast_generic(ServerToClient::TileRotated {
        tile_index,
        clockwise,
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_player_moved(player_id: &str, new_position: (usize, usize)) {
    log!(
        "[GameChannel] Broadcasting player moved: player={}, new_position={:?}",
        player_id,
        new_position
    );
    broadcast_generic(ServerToClient::PlayerMoved {
        player_id: player_id.to_string(),
        new_position,
    });
}
