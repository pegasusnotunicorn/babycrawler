use turbo::*;
use crate::game::cards::card::Card;

use crate::game::animation::{
    start_tile_rotation_animation,
    start_player_movement_animation,
    start_direct_player_movement_animation,
    animate_tile_to_index,
};

use crate::GameState;
use crate::game::map::clear_highlights;

pub fn receive_connected_users(game_state: &mut GameState, users: Vec<String>) {
    log!("📨 [RECEIVE] Connected users: {:?}", users);

    game_state.in_lobby = users.clone();
    game_state.user_id_to_player_id.clear();

    // Handle player mapping for any number of players
    if !game_state.in_lobby.is_empty() {
        log!("📨 [RECEIVE] Inserting player 1");
        game_state.user_id_to_player_id.insert(
            game_state.in_lobby[0].clone(),
            crate::game::map::PlayerId::Player1
        );

        if game_state.in_lobby.len() >= 2 {
            log!("📨 [RECEIVE] Inserting player 2");
            game_state.user_id_to_player_id.insert(
                game_state.in_lobby[1].clone(),
                crate::game::map::PlayerId::Player2
            );
        }
    }
}

pub fn receive_board_state(
    game_state: &mut GameState,
    tiles: Vec<crate::game::map::Tile>,
    players: Vec<crate::game::map::Player>,
    current_turn: Option<crate::server::CurrentTurn>
) {
    log!("📨 [RECEIVE] Board state, current_turn: {:?}", current_turn);

    // Check if the turn has changed to a different player
    let turn_changed = if
        let (Some(old_turn), Some(new_turn)) = (&game_state.current_turn, &current_turn)
    {
        old_turn.player_id != new_turn.player_id
    } else {
        // If either is None, consider it a change (new game or first turn)
        game_state.current_turn.is_some() != current_turn.is_some()
    };

    // If turn changed to a different player, start a new turn
    if turn_changed {
        log!("📨 [RECEIVE] Turn changed, starting new turn");
        game_state.start_new_turn();
    }

    // Update game state
    game_state.tiles = tiles;
    game_state.players = players;
    game_state.current_turn = current_turn.clone();
}

pub fn receive_card_cancelled(
    game_state: &mut GameState,
    card_index: &usize,
    card: &Card,
    player_id: &str
) {
    log!(
        "📨 [RECEIVE] Card cancelled by {}: {:?}, hand_index: {:?}",
        player_id,
        card.name,
        card.hand_index
    );

    if game_state.user == player_id {
        game_state.selected_card = None;
        game_state.play_area[*card_index] = Card::dummy_card();
        crate::game::map::tile::clear_highlights(&mut game_state.tiles);
    }
}

pub fn receive_tile_rotation(
    game_state: &mut GameState,
    tile_index: &usize,
    tile: &crate::game::map::tile::Tile,
    player_id: &str
) {
    log!(
        "📨 [RECEIVE] Tile rotation: index={}, rotation={}, player={}",
        tile_index,
        tile.current_rotation,
        player_id
    );
    let is_local_player = game_state.user == player_id;
    let client_tile = &mut game_state.tiles[*tile_index];

    // Update the client tile with all the server data
    client_tile.entrances = tile.entrances.clone();
    client_tile.current_rotation = tile.current_rotation;
    client_tile.original_rotation = tile.original_rotation;
    client_tile.original_location = tile.original_location;

    if !is_local_player {
        start_tile_rotation_animation(game_state, *tile_index, None, 0.25);
    }
}

pub fn receive_player_moved(
    game_state: &mut GameState,
    player_id: &str,
    new_position: &(usize, usize),
    is_canceled: bool
) {
    log!(
        "📨 [RECEIVE] Player moved: player={}, new_position={:?}, is_canceled={}",
        player_id,
        new_position,
        is_canceled
    );

    // Only start animation for non-local players (local player animation is handled in card_effect)
    let is_local_player = game_state.user == player_id;
    if is_local_player {
        log!("📨 [RECEIVE] Local player moved, skipping animation (already handled)");
        return;
    }

    // Get board layout before mutable borrow
    let (_, _, tile_size, offset_x, offset_y) = game_state.get_board_layout(false);

    // Get the current position of the player
    if let Some(player) = game_state.get_player_by_user_id(player_id) {
        let current_position = player.position;
        log!(
            "📨 [RECEIVE] Player {:?} moving from {:?} to {:?}",
            player.id,
            current_position,
            new_position
        );

        if is_canceled {
            // Use direct animation for canceled movements
            start_direct_player_movement_animation(
                game_state,
                player_id,
                current_position,
                *new_position,
                tile_size,
                offset_x,
                offset_y
            );
        } else {
            // Use path-based animation for normal movements
            start_player_movement_animation(
                game_state,
                player_id,
                current_position,
                *new_position,
                tile_size,
                offset_x,
                offset_y
            );
        }
    } else {
        log!("📨 [RECEIVE] Could not find PlayerId for user_id: {}", player_id);
    }
}

pub fn receive_card_confirmed(game_state: &mut GameState, card: &Card, player_id: &str) {
    log!("📨 [RECEIVE] Card confirmed by {}: {:?}", player_id, card);

    if game_state.user == player_id {
        game_state.selected_card = None;
        clear_highlights(&mut game_state.tiles);
    }
}

pub fn receive_tiles_swapped(
    game_state: &mut GameState,
    tile_index_1: &usize,
    tile_index_2: &usize
) {
    log!("📨 [RECEIVE] Tiles swapped: {} <-> {}", tile_index_1, tile_index_2);
    // For tile swaps, we animate first, then swap when animation completes
    // This keeps the indices consistent during animation

    // Track this swap to be performed when animation completes
    game_state.pending_swaps.push((*tile_index_1, *tile_index_2));

    // Start animations for both tiles
    animate_tile_to_index(game_state, *tile_index_1, *tile_index_2);
    animate_tile_to_index(game_state, *tile_index_2, *tile_index_1);
}
