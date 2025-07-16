use crate::{ GameState, Card, HAND_SIZE };

/// Advance to the next player's turn (singleplayer only)
pub fn next_turn(state: &mut GameState) {
    state.current_turn = (state.current_turn + 1) % state.players.len();
}

/// Deal a card to the current player (singleplayer only)
pub fn deal_card(state: &mut GameState) {
    let player = &mut state.players[state.current_turn];
    if player.hand.len() < HAND_SIZE {
        player.hand.push(Card::move_card());
    }
}
