use crate::game::cards::card::{ Card, CardVisualState };
use crate::game::cards::card_effect::CardEffect;
use crate::game::cards::card_slot::CardSlot;
use crate::game::constants::{ GAME_PADDING, HAND_SIZE, CARD_DUMMY_COLOR };
use crate::game::util::point_in_bounds;

#[derive(Clone, Debug)]
pub struct CardRow {
    pub slots: Vec<CardSlot>,
    pub y: u32,
    pub card_width: u32,
    pub card_height: u32,
}

impl CardRow {
    pub fn new(cards: &[Card], y: u32, card_width: u32, card_height: u32) -> Self {
        let mut slots = Vec::with_capacity(HAND_SIZE);
        for i in 0..HAND_SIZE {
            let card = cards.get(i).cloned();
            slots.push(CardSlot {
                card,
                visual_state: CardVisualState::NONE,
            });
        }
        Self { slots, y, card_width, card_height }
    }

    pub fn get_slot_position(&self, index: usize) -> (u32, u32) {
        let x = GAME_PADDING + (index as u32) * (self.card_width + GAME_PADDING);
        (x, self.y)
    }

    pub fn draw(&self, frame: f64) {
        for (i, slot) in self.slots.iter().enumerate() {
            let (x, y) = self.get_slot_position(i);
            let outline_color = slot.card
                .as_ref()
                .map(|c| c.color)
                .unwrap_or(CARD_DUMMY_COLOR);
            if let Some(card) = &slot.card {
                card.draw(
                    x,
                    y,
                    self.card_width,
                    self.card_height,
                    outline_color,
                    true,
                    slot.visual_state,
                    Some(frame)
                );
            } else {
                // Draw dummy/empty slot
                let dummy = Card {
                    id: 0,
                    name: String::new(),
                    effect: CardEffect::Dummy,
                    color: outline_color,
                    hand_index: None,
                    hide_confirm_button: false,
                };
                dummy.draw(
                    x,
                    y,
                    self.card_width,
                    self.card_height,
                    outline_color,
                    true,
                    slot.visual_state | CardVisualState::DUMMY,
                    Some(frame)
                );
            }
        }
    }

    /// Returns the index of the slot at the given (x, y) point, or None if not in any slot
    pub fn slot_at_point(&self, px: i32, py: i32) -> Option<usize> {
        for (i, _slot) in self.slots.iter().enumerate() {
            let (x, y) = self.get_slot_position(i);
            let bounds = turbo::Bounds::new(x, y, self.card_width, self.card_height);
            if point_in_bounds(px, py, &bounds) {
                return Some(i);
            }
        }
        None
    }

    /// Returns the index of the leftmost dummy or real card, depending on the dummy parameter
    pub fn leftmost_card_index(&self, dummy: bool) -> Option<usize> {
        self.slots.iter().position(|slot|
            slot.card
                .as_ref()
                .map(|c| c.is_dummy() == dummy)
                .unwrap_or(false)
        )
    }
}
