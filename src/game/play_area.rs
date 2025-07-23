use crate::game::constants::{ HAND_SIZE, PLAY_AREA_COLOR, GAME_PADDING };
use crate::game::hand::{ get_card_sizes, get_hand_y };
use crate::game::card::Card;
use crate::game::card_effect::CardEffect;
use crate::GameState;

/// Draws 3 card-shaped rectangles in the play area (above the text bar, below the board)
pub fn draw_play_area(state: &GameState) {
    let (canvas_width, canvas_height, _tile_size, _offset_x, _offset_y) =
        state.get_board_layout(false);
    let (card_width, card_height) = get_card_sizes(canvas_width, canvas_height);
    // Place the play area cards at the same X as the hand cards, but just above the text bar
    let y = get_hand_y() + card_height + GAME_PADDING;
    for i in 0..HAND_SIZE {
        let x = GAME_PADDING + (i as u32) * (card_width + GAME_PADDING);
        // Create a dummy Card just for drawing the rectangle with the correct color
        let dummy_card = Card {
            id: 0,
            name: String::new(),
            effect: CardEffect::MoveOneTile,
            color: PLAY_AREA_COLOR,
        };
        dummy_card.draw_rect(
            x,
            y,
            card_width,
            card_height,
            PLAY_AREA_COLOR,
            true,
            false,
            false,
            None
        );
    }
}
