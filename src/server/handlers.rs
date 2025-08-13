use turbo::*;
use crate::game::cards::card_effect::CardEffect;
use crate::server::{ GameChannel, CurrentTurn };
use crate::server::broadcast::{
    broadcast_board_state,
    broadcast_card_cancelled,
    broadcast_card_confirmed,
    broadcast_tile_rotation,
    broadcast_player_moved,
    broadcast_tiles_swapped,
};
use crate::game::cards::card::Card;
use crate::game::constants::HAND_SIZE;

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

/// Helper function to give a player a new hand of random cards
pub fn give_player_new_hand(channel: &mut GameChannel, user_id: &str) {
    if let Some(player) = get_player_mut(channel, user_id) {
        player.hand.clear();
        for i in 0..HAND_SIZE {
            let mut card = Card::random();
            card.hand_index = Some(i);
            log!("[GameChannel] Created card {} with hand_index: {:?}", card.name, card.hand_index);
            player.hand.push(card);
        }
        log!("[GameChannel] Gave new hand to player {:?}", player.id);
    }
}

/// Helper function to start a new turn
pub fn handle_new_turn(channel: &mut GameChannel, user_id: &str) {
    log!("[GameChannel] Player {user_id} has ended their turn");

    // Give the old player a new hand of random cards
    give_player_new_hand(channel, user_id);

    // Move to next player's turn
    channel.current_turn_index = (channel.current_turn_index + 1) % channel.players.len();

    if let Some(next_user_id) = channel.players.get(channel.current_turn_index) {
        channel.current_turn = Some(CurrentTurn {
            player_id: next_user_id.clone(),
            selected_card: None,
            selected_card_index: 0,
            confirmed_cards_count: 0,
        });

        // log the rotation of the first tile
        log!("[GameChannel] Tile 0 rotation: {:?}", channel.board_tiles[0].current_rotation);

        broadcast_board_state(&channel.board_tiles, &channel.board_players, &channel.current_turn);
    }
}

pub fn handle_select_card(channel: &mut GameChannel, user_id: &str, hand_index: usize) {
    log!("[GameChannel] SelectCard received from {user_id}: hand_index={}", hand_index);

    // Get the player and their selected card
    let (_player, selected_card) = if let Some(player) = get_player_mut(channel, user_id) {
        if hand_index < player.hand.len() {
            let card = player.hand[hand_index].clone();
            // Replace the selected card with a dummy card
            player.hand[hand_index] = Card::dummy_card();
            (player, card)
        } else {
            log!("[GameChannel] Invalid hand_index: {} for player {}", hand_index, user_id);
            return;
        }
    } else {
        log!("[GameChannel] Could not find player for user_id: {}", user_id);
        return;
    };

    // Preserve the existing confirmed_cards_count if this is the same player's turn
    let confirmed_count = if let Some(existing_turn) = &channel.current_turn {
        if existing_turn.player_id == user_id {
            existing_turn.confirmed_cards_count
        } else {
            0 // New player's turn, reset counter
        }
    } else {
        0 // No existing turn, start fresh
    };

    channel.current_turn = Some(CurrentTurn {
        player_id: user_id.to_string(),
        selected_card: Some(selected_card.clone()),
        selected_card_index: hand_index,
        confirmed_cards_count: confirmed_count,
    });
}

pub fn handle_cancel_select_card(channel: &mut GameChannel, user_id: &str, hand_index: usize) {
    // Get the card from the current turn to handle cancellation logic
    let card = if let Some(turn) = &channel.current_turn {
        if let Some(selected_card) = &turn.selected_card {
            selected_card.clone()
        } else {
            log!("[GameChannel] No card selected for cancel");
            return;
        }
    } else {
        log!("[GameChannel] No current turn for cancel");
        return;
    };

    // Handle card-specific cancellation logic
    match card.effect {
        CardEffect::RotateCard => {
            // Revert all tiles to their original rotation in the server's board state
            for tile in channel.board_tiles.iter_mut() {
                tile.current_rotation = tile.original_rotation;
                tile.rotate_entrances(tile.original_rotation);
            }
            log!("[GameChannel] Reverted all tiles to original rotation for RotateCard cancel");
        }
        CardEffect::SwapCard => {
            // Revert all tiles to their original positions in the server's board state
            // First, collect tiles that need to be moved back
            let mut tiles_to_revert: Vec<(usize, usize)> = Vec::new();
            for (current_index, tile) in channel.board_tiles.iter().enumerate() {
                let target_index = tile.original_location;
                if current_index != target_index {
                    tiles_to_revert.push((current_index, target_index));
                }
            }

            // Perform the reverts by swapping tiles back to their original positions
            for (current_index, target_index) in tiles_to_revert {
                channel.board_tiles.swap(current_index, target_index);
            }
            log!("[GameChannel] Reverted all tiles to original positions for SwapCard cancel");
        }
        CardEffect::MoveOneTile => {
            // Move the player back to their original position in the server's board state
            if let Some(player) = get_player_mut(channel, user_id) {
                player.position = player.original_position;
                log!(
                    "[GameChannel] Moved player back to original position {:?} for MoveOneTile cancel",
                    player.original_position
                );
            }
        }
        _ => {
            // For other card types, no special handling needed
            // Just update the turn state below
        }
    }

    // Restore the original card back to the player's hand
    if let Some(player) = get_player_mut(channel, user_id) {
        if hand_index < player.hand.len() {
            player.hand[hand_index] = card.clone();
        }
    }

    // Update the current turn state
    channel.current_turn = Some(CurrentTurn {
        player_id: user_id.to_string(),
        selected_card: None,
        selected_card_index: hand_index,
        confirmed_cards_count: 0,
    });

    broadcast_card_cancelled(hand_index, &card, user_id);
    broadcast_board_state(&channel.board_tiles, &channel.board_players, &channel.current_turn);
}

pub fn handle_confirm_card(channel: &mut GameChannel, user_id: &str, card: Card) {
    log!("[GameChannel] ConfirmCard received from {user_id}: card={:?}", card);

    match card.effect {
        CardEffect::MoveOneTile => {
            if let Some(player) = get_player_mut(channel, user_id) {
                player.update_original_position();
            }
        }
        CardEffect::RotateCard => {
            for tile in channel.board_tiles.iter_mut() {
                tile.original_rotation = tile.current_rotation;
            }
        }
        CardEffect::SwapCard => {
            for (current_index, tile) in channel.board_tiles.iter_mut().enumerate() {
                tile.original_location = current_index;
            }
        }
        _ => {
            // Do nothing for other card types
        }
    }

    // Replace the confirmed card in the player's hand with a dummy card
    // Use the card's hand_index instead of the turn's selected_card_index
    if let Some(hand_index) = card.hand_index {
        if let Some(player) = get_player_mut(channel, user_id) {
            if hand_index < player.hand.len() {
                player.hand[hand_index] = Card::dummy_card();
                log!(
                    "[GameChannel] Replaced card at hand index {} with dummy card for player {}",
                    hand_index,
                    user_id
                );
            }
        }
    }

    broadcast_card_confirmed(&card, user_id);

    // Clear the current turn's selected card and increment confirmed count
    if let Some(turn) = &mut channel.current_turn {
        if turn.player_id == user_id {
            turn.selected_card = None;
            turn.confirmed_cards_count += 1;

            // Check if this was last card in their hand
            if turn.confirmed_cards_count >= HAND_SIZE {
                handle_new_turn(channel, user_id);
            }
        }
    }
}

pub fn handle_rotate_tile(channel: &mut GameChannel, user_id: &str, tile_index: usize) {
    log!("[GameChannel] RotateTile received from {user_id}: index={}", tile_index);

    if let Some(tile) = channel.board_tiles.get_mut(tile_index) {
        let new_rotation = (tile.current_rotation + 1) % 4;
        tile.rotate_entrances(new_rotation);
    }

    if let Some(tile) = channel.board_tiles.get(tile_index) {
        broadcast_tile_rotation(tile_index, tile, user_id);
    }
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

    if let Some(player) = get_player_mut(channel, user_id) {
        player.position = new_position;
        log!("[GameChannel] Updated player {:?} position to {:?}", player.id, new_position);
    } else {
        log!("[GameChannel] Could not find player for user_id: {}", user_id);
    }

    broadcast_player_moved(user_id, new_position, is_canceled);
}

pub fn handle_swap_tiles(
    channel: &mut GameChannel,
    user_id: &str,
    tile_index_1: usize,
    tile_index_2: usize
) {
    log!("[GameChannel] SwapTiles received from {user_id}: {} <-> {}", tile_index_1, tile_index_2);

    // Validate that the tiles are within bounds
    if tile_index_1 >= channel.board_tiles.len() || tile_index_2 >= channel.board_tiles.len() {
        log!(
            "[GameChannel] Invalid tile indices for swap: {} or {} out of bounds",
            tile_index_1,
            tile_index_2
        );
        return;
    }

    channel.board_tiles.swap(tile_index_1, tile_index_2);
    broadcast_tiles_swapped(tile_index_1, tile_index_2);
}
