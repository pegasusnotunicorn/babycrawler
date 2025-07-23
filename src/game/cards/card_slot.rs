use crate::game::card::{ Card, CardVisualState };

#[derive(Clone, Debug)]
pub struct CardSlot {
    pub card: Option<Card>,
    pub visual_state: CardVisualState,
}
