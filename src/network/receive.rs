use turbo::*;
use crate::game::cards::card::Card;
use crate::game::cards::card_input::update_state_with_card;
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
            &turn_info.selected_card,
            &turn_info.player_id
        );
    }
}

pub fn receive_card_selection(
    game_state: &mut GameState,
    card_index: &usize,
    card: &Option<Card>,
    player_id: &str
) {
    log!("📨 [RECEIVE] Card selected by {}: {:?}", player_id, card);

    if let Some(current_turn) = &mut game_state.current_turn {
        if current_turn.player_id == player_id {
            current_turn.selected_card = card.clone();
            current_turn.selected_card_index = *card_index;

            // If it's the local player's turn, update the play area
            if game_state.user == player_id {
                update_state_with_card(game_state, *card_index, card.as_ref());
            }
        }
    }
}

pub fn receive_card_cancel(game_state: &mut GameState, card_index: &usize, player_id: &str) {
    log!("📨 [RECEIVE] Card canceled by {}: {}", player_id, card_index);

    let selected_card = game_state.current_turn
        .as_ref()
        .and_then(|turn| turn.selected_card.as_ref())
        .cloned()
        .unwrap_or_else(|| Card::dummy_card());

    let is_local_player = game_state.user == player_id;

    if let Some(player) = game_state.get_local_player_mut() {
        // If it's the local player's turn, update the hand
        if is_local_player {
            player.hand[*card_index] = selected_card;
        }
    }
}
