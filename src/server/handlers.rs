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
    broadcast_game_over,
};
use crate::game::cards::card::Card;
use crate::game::constants::{ DEBUG_MODE, HAND_SIZE, FIREBALL_DAMAGE };
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

pub fn handle_end_turn(channel: &mut GameChannel, user_id: &str) {
    if let Some(turn) = &channel.current_turn {
        if turn.player_id == user_id && turn.selected_card.is_some() {
            if let Some(selected_card) = &turn.selected_card {
                handle_confirm_card(channel, user_id, selected_card.clone());
            }
        }
    }

    give_player_new_hand(channel, user_id);

    channel.current_turn_index = (channel.current_turn_index + 1) % channel.players.len();

    if let Some(next_user_id) = channel.players.get(channel.current_turn_index) {
        channel.current_turn = Some(CurrentTurn {
            player_id: next_user_id.clone(),
            selected_card: None,
            selected_card_index: 0,
        });

        broadcast_board_state(
            &channel.board_tiles,
            &channel.board_players,
            &channel.board_monster,
            &channel.current_turn
        );
    }
}

pub fn handle_select_card(channel: &mut GameChannel, user_id: &str, hand_index: usize) {
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

    channel.current_turn = Some(CurrentTurn {
        player_id: user_id.to_string(),
        selected_card: Some(selected_card.clone()),
        selected_card_index: hand_index,
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
                tile.rotate_entrances(tile.original_rotation);
                tile.current_rotation = tile.original_rotation;
            }
            log!("[GameChannel] Reverted all tiles to original rotation for RotateCard cancel");
        }
        CardEffect::SwapCard => {
            // Revert all tiles to their original positions by rebuilding the array
            let mut tiles_with_positions: Vec<_> = channel.board_tiles
                .iter()
                .enumerate()
                .map(|(_current_index, tile)| (tile.original_location, tile.clone()))
                .collect();

            // Sort by original_location to get tiles back in order
            tiles_with_positions.sort_by_key(|(original_pos, _)| *original_pos);

            // Extract just the tiles in the correct order
            channel.board_tiles = tiles_with_positions
                .into_iter()
                .map(|(_, tile)| tile)
                .collect();

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
    });

    broadcast_card_cancelled(&card, user_id);
    broadcast_board_state(
        &channel.board_tiles,
        &channel.board_players,
        &channel.board_monster,
        &channel.current_turn
    );
}

pub fn handle_confirm_card(channel: &mut GameChannel, user_id: &str, card: Card) {
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

    // Clear the current turn's selected card
    if let Some(turn) = &mut channel.current_turn {
        if turn.player_id == user_id {
            turn.selected_card = None;
        }
    }
}

pub fn handle_rotate_tile(channel: &mut GameChannel, user_id: &str, tile_index: usize) {
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
    if let Some(player) = get_player_mut(channel, user_id) {
        player.position = new_position;
        log!("[GameChannel] Updated player {:?} position to {:?}", player.id, new_position);
    } else {
        log!("[GameChannel] Could not find player for user_id: {}", user_id);
    }

    broadcast_player_moved(user_id, new_position, is_canceled);
}

pub fn handle_swap_tiles(channel: &mut GameChannel, tile_index_1: usize, tile_index_2: usize) {
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
    // if not debug, confirm the card
    if !DEBUG_MODE {
        handle_confirm_card(channel, user_id, Card::fire_card());
    }
    broadcast_fireball_shot(user_id, target_tile, &direction);
}

pub fn handle_fireball_hit(
    channel: &mut GameChannel,
    shooter_id: &str,
    from_tile_index: usize,
    _direction: crate::game::map::tile::Direction
) {
    // Convert tile index to position coordinates
    let from_position = (from_tile_index % 5, from_tile_index / 5);
    let hit_position = from_position;

    // Find the monster at the hit position
    if let Some(monster) = &mut channel.board_monster {
        if monster.position == hit_position {
            log!("[GameChannel] Fireball hit monster at {:?}", hit_position);
            monster.take_damage(FIREBALL_DAMAGE);
            log!(
                "[GameChannel] Monster took {} damage, health now: {}",
                FIREBALL_DAMAGE,
                monster.health
            );

            // Check if monster is defeated
            if monster.health <= 0 {
                log!("[GameChannel] Monster defeated! Both players win!");
                // Get both player user IDs for the victory
                let player1_id = channel.players.get(0).cloned();
                let player2_id = channel.players.get(1).cloned();

                if let (Some(p1), Some(p2)) = (player1_id, player2_id) {
                    broadcast_game_over(&[p1, p2], &[]); // Both players win, no losers
                    return;
                }
            }

            // Broadcast fireball hit result with monster damage
            broadcast_fireball_hit_result(shooter_id, "monster", &0, Some(FIREBALL_DAMAGE));
            return;
        }
    }

    // Find the target player at the hit position
    let (target_player_index, target_player) = match
        channel.board_players
            .iter()
            .enumerate()
            .find(|(_, player)| player.position == hit_position)
    {
        Some((index, player)) => (index, player),
        None => {
            log!("[GameChannel] No player at hit position {:?}, no damage dealt", hit_position);
            return;
        }
    };

    // Convert PlayerId enum to actual user ID
    let target_user_id = match channel.get_user_id(&target_player.id) {
        Some(id) => id.clone(),
        None => {
            log!("[GameChannel] Could not find user ID for player {:?}", target_player.id);
            return;
        }
    };

    // Don't damage the shooter
    if target_user_id == shooter_id {
        log!("[GameChannel] Fireball hit shooter, no damage dealt");
        return;
    }

    // Apply damage
    let damage_dealt = FIREBALL_DAMAGE;
    let player_id = target_player.id.clone(); // Clone the ID before mutable borrow

    if let Some(player_mut) = channel.board_players.get_mut(target_player_index) {
        player_mut.take_damage(damage_dealt);
        log!("[GameChannel] Player {} took {} damage from fireball", player_mut.id, damage_dealt);

        // Check for game over
        if player_mut.health <= 0 {
            handle_player_death(channel, player_id);
        }
    }

    // Broadcast the hit result
    broadcast_fireball_hit_result(shooter_id, &target_user_id, &damage_dealt, None);
}

/// Handle player death and determine winner
fn handle_player_death(channel: &mut GameChannel, dead_player_id: PlayerId) {
    // Get the winner and loser user IDs
    let winner_user_id = match dead_player_id {
        PlayerId::Player1 => channel.get_user_id(&PlayerId::Player2),
        PlayerId::Player2 => channel.get_user_id(&PlayerId::Player1),
    };

    let loser_user_id = channel.get_user_id(&dead_player_id);

    if let (Some(winner), Some(loser)) = (winner_user_id, loser_user_id) {
        log!("[GameChannel] Player {} wins the game!", winner);
        broadcast_game_over(&[winner.clone()], &[loser.clone()]);
    }
}
