use turbo::*;
use crate::game::cards::card::Card;
use crate::game::cards::card_input::update_state_with_card;
use crate::game::animation::start_tile_rotation_animation;
use crate::game::cards::card_effect::CardEffect;
use crate::GameState;

pub fn receive_connected_users(game_state: &mut GameState, users: Vec<String>) {
    log!("📨 [RECEIVE] Connected users: {:?}", users);

    game_state.in_lobby = users.clone();
    game_state.user_id_to_player_id.clear();
    if game_state.in_lobby.len() >= 2 {
        game_state.user_id_to_player_id.insert(
            game_state.in_lobby[0].clone(),
            crate::game::map::PlayerId::Player1
        );
        game_state.user_id_to_player_id.insert(
            game_state.in_lobby[1].clone(),
            crate::game::map::PlayerId::Player2
        );
    }
}

pub fn receive_board_state(
    game_state: &mut GameState,
    tiles: Vec<crate::game::map::Tile>,
    players: Vec<crate::game::map::Player>,
    current_turn: Option<crate::server::CurrentTurn>
) {
    log!("📨 [RECEIVE] Board state, current_turn: {:?}", current_turn);

    // Update game state
    game_state.tiles = tiles;
    game_state.players = players;
    game_state.current_turn = current_turn.clone();

    // Handle current turn card selection if present
    if let Some(turn_info) = &current_turn {
        receive_card_selection(
            game_state,
            &turn_info.selected_card_index,
            &turn_info.selected_card.as_ref().unwrap(),
            &turn_info.player_id
        );
    }
}

pub fn receive_card_selection(
    game_state: &mut GameState,
    card_index: &usize,
    card: &Card,
    player_id: &str
) {
    log!("📨 [RECEIVE] Card selected by {}: {:?}", player_id, card);

    if let Some(current_turn) = &mut game_state.current_turn {
        if current_turn.player_id == player_id {
            current_turn.selected_card = Some(card.clone());
            current_turn.selected_card_index = *card_index;

            // If it's the local player's turn, update the play area
            if game_state.user == player_id {
                update_state_with_card(game_state, *card_index, card.as_ref());
            }
        }
    }

    match card.effect {
        CardEffect::MoveOneTile => {
            // Do nothing
        }
        CardEffect::RotateCard => {
            // Store current rotation for all tiles
            for tile in game_state.tiles.iter_mut() {
                tile.original_rotation = tile.current_rotation;
            }
        }
        CardEffect::SwapCard => {
            // Do nothing
        }
        CardEffect::Dummy => {
            // Do nothing
        }
    }
}

pub fn receive_card_cancel(
    game_state: &mut GameState,
    card_index: &usize,
    card: &Card,
    player_id: &str
) {
    log!("📨 [RECEIVE] Card canceled by {}: {}", player_id, card_index);

    let is_local_player = game_state.user == player_id;

    // If it's the local player's turn, update the hand
    if is_local_player {
        if let Some(player) = game_state.get_local_player_mut() {
            player.hand[*card_index] = card.clone();
        }
    }

    // Depending on the canceled card, we need to do different things
    match card.effect {
        CardEffect::MoveOneTile => {
            // Do nothing
        }
        CardEffect::RotateCard => {
            CardEffect::revert_tile_rotations(&mut game_state.tiles);
        }
        CardEffect::SwapCard => {
            // Do nothing
        }
        CardEffect::Dummy => {
            // Do nothing
        }
    }
}

pub fn receive_tile_rotation(
    game_state: &mut GameState,
    tile_index: &usize,
    clockwise: &bool,
    player_id: &str
) {
    log!(
        "📨 [RECEIVE] Tile rotation: index={}, clockwise={}, player={}",
        tile_index,
        clockwise,
        player_id
    );
    let is_local_player = game_state.user == player_id;
    let tile = &mut game_state.tiles[*tile_index];

    if !is_local_player {
        if tile.rotation_anim.is_none() {
            start_tile_rotation_animation(game_state, *tile_index, *clockwise, 0.25);
        }
    }
}
