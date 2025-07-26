use turbo::*;
use crate::server::{ GameChannel, CurrentTurn };
use crate::server::broadcast::*;
use crate::game::cards::card::Card;

pub fn handle_end_turn(channel: &mut GameChannel, user_id: &str) {
    log!("[GameChannel] EndTurn received from {user_id}");
    if channel.players.get(channel.current_turn_index) == Some(&user_id.to_string()) {
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
