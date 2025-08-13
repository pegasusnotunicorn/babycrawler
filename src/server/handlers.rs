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
    broadcast_fireball_shot,
    broadcast_fireball_hit_result,
};
use crate::game::cards::card::Card;
use crate::game::constants::{ HAND_SIZE, FIREBALL_DAMAGE };
use crate::game::map::player::Player;
use crate::game::map::player::PlayerId;

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
        let mut new_hand = Player::new_hand(HAND_SIZE, false);

        for (i, card) in new_hand.iter_mut().enumerate() {
            card.hand_index = Some(i);
        }

        player.hand = new_hand;
    }
}

/// Helper function to start a new turn
pub fn handle_new_turn(channel: &mut GameChannel, user_id: &str) {
    log!("[GameChannel] Player {user_id} has ended their turn");

    give_player_new_hand(channel, user_id);

    channel.current_turn_index = (channel.current_turn_index + 1) % channel.players.len();

    if let Some(next_user_id) = channel.players.get(channel.current_turn_index) {
        channel.current_turn = Some(CurrentTurn {
            player_id: next_user_id.clone(),
            selected_card: None,
            selected_card_index: 0,
            confirmed_cards_count: 0,
        });

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
        CardEffect::FireCard => {
            // TODO: Implement fire card cancel logic
            // For now, no special handling needed
            log!("[GameChannel] FireCard cancelled - no special handling needed");
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

    broadcast_card_cancelled(&card, user_id);
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

pub fn handle_fireball_shot(
    channel: &mut GameChannel,
    user_id: &str,
    target_tile: usize,
    direction: crate::game::map::tile::Direction
) {
    log!(
        "[GameChannel] FireballShot received from {user_id}: target_tile={}, direction={:?}",
        target_tile,
        direction
    );

    // Check if it's the user's turn
    if let Some(current_turn) = &channel.current_turn {
        if current_turn.player_id != user_id {
            log!("[GameChannel] Not {}'s turn, ignoring fireball request", user_id);
            return;
        }
    } else {
        log!("[GameChannel] No active turn, ignoring fireball request");
        return;
    }

    // Validate tile index
    if target_tile >= channel.board_tiles.len() {
        log!("[GameChannel] Invalid tile index: {}", target_tile);
        return;
    }

    let player = get_player_mut(channel, user_id).unwrap();
    let player_pos = player.position;

    log!("[GameChannel] Fireball created at {:?} in direction {:?}", player_pos, direction);
    // handle_confirm_card(channel, user_id, Card::fire_card());
    broadcast_fireball_shot(user_id, target_tile, &direction);
}

pub fn handle_fireball_hit(
    channel: &mut GameChannel,
    shooter_id: &str,
    from_tile_index: usize,
    direction: crate::game::map::tile::Direction
) {
    log!(
        "[GameChannel] FireballHit received from {shooter_id}: from_tile_index={}, direction={:?}",
        from_tile_index,
        direction
    );

    // Convert tile index to position coordinates
    let from_position = (from_tile_index % 5, from_tile_index / 5);

    // Calculate where the fireball would hit based on direction and starting position
    let hit_position = calculate_fireball_hit_position(from_position, direction);

    // Check if we hit a player
    let mut target_player_id = None;
    let mut damage_dealt = 0;
    let mut player_index_to_damage = None;

    // First pass: find the target player
    for (player_index, player) in channel.board_players.iter().enumerate() {
        if player.position == hit_position {
            // Convert PlayerId enum to actual user ID
            let actual_user_id = match player.id {
                PlayerId::Player1 => channel.players.get(0).cloned(),
                PlayerId::Player2 => channel.players.get(1).cloned(),
            };

            if let Some(user_id) = actual_user_id {
                if user_id != shooter_id {
                    target_player_id = Some(user_id);
                    damage_dealt = FIREBALL_DAMAGE;
                    player_index_to_damage = Some(player_index);
                    break;
                }
            }
        }
    }

    // Second pass: apply damage if we found a player
    if let Some(player_index) = player_index_to_damage {
        if let Some(player_mut) = channel.board_players.get_mut(player_index) {
            player_mut.take_damage(damage_dealt);
            log!(
                "[GameChannel] Player {} took {} damage from fireball",
                player_mut.id,
                damage_dealt
            );
        }
    }

    if let Some(target_player_id) = target_player_id {
        broadcast_fireball_hit_result(shooter_id, &target_player_id, &damage_dealt);
    }
}

/// Calculate where a fireball would hit based on starting position and direction
fn calculate_fireball_hit_position(
    from_position: (usize, usize),
    direction: crate::game::map::tile::Direction
) -> (usize, usize) {
    match direction {
        crate::game::map::tile::Direction::Up => {
            let new_y = from_position.1.saturating_sub(1);
            (from_position.0, new_y)
        }
        crate::game::map::tile::Direction::Down => {
            let new_y = (from_position.1 + 1).min(4);
            (from_position.0, new_y)
        }
        crate::game::map::tile::Direction::Left => {
            let new_x = from_position.0.saturating_sub(1);
            (new_x, from_position.1)
        }
        crate::game::map::tile::Direction::Right => {
            let new_x = (from_position.0 + 1).min(4);
            (new_x, from_position.1)
        }
    }
}
