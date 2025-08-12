use turbo::*;
use crate::server::{ GameChannel, CurrentTurn };
use crate::server::broadcast::*;
use crate::game::cards::card::Card;

/// Helper function to get the player index for a given user_id
fn get_player_index(channel: &GameChannel, user_id: &str) -> Option<usize> {
    channel.players.iter().position(|p| p == user_id)
}

/// Helper function to get a mutable reference to the player for a given user_id
fn get_player_mut<'a>(
    channel: &'a mut GameChannel,
    user_id: &str
) -> Option<&'a mut crate::game::map::Player> {
    let player_index = get_player_index(channel, user_id)?;
    channel.board_players.get_mut(player_index)
}

pub fn handle_end_turn(channel: &mut GameChannel, user_id: &str) {
    log!("[GameChannel] EndTurn received from {user_id}");
    if channel.players.get(channel.current_turn_index) == Some(&user_id.to_string()) {
        // Update the original position for the current player to their final position
        if let Some(player) = get_player_mut(channel, user_id) {
            player.update_original_position();
            log!(
                "[GameChannel] Updated original position for {:?} to {:?}",
                player.id,
                player.original_position
            );
        }

        channel.current_turn_index = (channel.current_turn_index + 1) % channel.players.len();
        broadcast_turn(
            &channel.players,
            channel.current_turn_index,
            &mut channel.current_turn,
            &channel.board_tiles,
            &channel.board_players
        );
    }
}

pub fn handle_select_card(channel: &mut GameChannel, user_id: &str, card_index: usize, card: Card) {
    log!("[GameChannel] SelectCard received from {user_id}: index={}, card={:?}", card_index, card);

    channel.current_turn = Some(CurrentTurn {
        player_id: user_id.to_string(),
        selected_card: Some(card.clone()),
        selected_card_index: card_index,
    });
    broadcast_card_selected(card_index, &card, user_id);
}

pub fn handle_cancel_select_card(
    channel: &mut GameChannel,
    user_id: &str,
    card_index: usize,
    card: Card
) {
    log!(
        "[GameChannel] CancelSelectCard received from {user_id}: index={}, card={:?}",
        card_index,
        card
    );
    channel.current_turn = Some(CurrentTurn {
        player_id: user_id.to_string(),
        selected_card: None,
        selected_card_index: card_index,
    });
    broadcast_card_canceled(card_index, &card, user_id);
}

pub fn handle_rotate_tile(
    _channel: &mut GameChannel,
    user_id: &str,
    tile_index: usize,
    clockwise: bool
) {
    log!(
        "[GameChannel] RotateTile received from {user_id}: index={}, clockwise={}",
        tile_index,
        clockwise
    );
    // You can add tile rotation logic here if needed
    broadcast_tile_rotation(tile_index, clockwise, user_id);
}

pub fn handle_move_player(
    channel: &mut GameChannel,
    user_id: &str,
    new_position: (usize, usize),
    is_canceled: bool
) {
    log!(
        "[GameChannel] MovePlayer received from {user_id}: new_position={:?}, is_canceled={}",
        new_position,
        is_canceled
    );

    // Use helper function to get and update the player
    if let Some(player) = get_player_mut(channel, user_id) {
        player.position = new_position;
        log!("[GameChannel] Updated player {:?} position to {:?}", player.id, new_position);
    } else {
        log!("[GameChannel] Could not find player for user_id: {}", user_id);
    }

    // Broadcast the move to all clients
    broadcast_player_moved(user_id, new_position, is_canceled);
}

pub fn handle_confirm_card(channel: &mut GameChannel, user_id: &str, card: Card) {
    log!(
        "[GameChannel] ConfirmCard received from {user_id}: card={:?}",
        card
    );
    
    // Clear the current turn's selected card
    if let Some(turn) = &mut channel.current_turn {
        if turn.player_id == user_id {
            turn.selected_card = None;
        }
    }
    
    // Broadcast the confirmation to all clients
    broadcast_card_confirmed(&card, user_id);
}

pub fn handle_swap_tiles(
    channel: &mut GameChannel,
    user_id: &str,
    tile_index_1: usize,
    tile_index_2: usize
) {
    log!(
        "[GameChannel] SwapTiles received from {user_id}: {} <-> {}",
        tile_index_1,
        tile_index_2
    );
    
    // Validate that the tiles are within bounds
    if tile_index_1 >= channel.board_tiles.len() || tile_index_2 >= channel.board_tiles.len() {
        log!("[GameChannel] Invalid tile indices for swap: {} or {} out of bounds", tile_index_1, tile_index_2);
        return;
    }
    
    // Swap the tiles in the board
    channel.board_tiles.swap(tile_index_1, tile_index_2);
    
    // Broadcast the swap to all clients
    broadcast_tiles_swapped(tile_index_1, tile_index_2);
}
