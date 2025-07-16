use crate::game::card_effect::CardEffect;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Card {
    pub id: u32,
    pub name: String,
    pub effect: CardEffect,
    pub color: u32,
}

const CARD_CONSTRUCTORS: &[fn() -> Card] = &[Card::move_card, Card::rotate_card, Card::swap_card];

impl Card {
    // toggle a card as selected
    pub fn toggle_in(selected: &mut Vec<Card>, card: &Card) {
        if let Some(pos) = selected.iter().position(|c| c == card) {
            selected.remove(pos);
        } else {
            selected.clear();
            selected.push(card.clone());
        }
    }

    // get a random card
    pub fn random() -> Self {
        let index = (random::u32() as usize) % CARD_CONSTRUCTORS.len();
        (CARD_CONSTRUCTORS[index])()
    }

    pub fn rotate_card() -> Self {
        Self {
            id: random::u32(),
            name: "Rotate".into(),
            effect: CardEffect::RotateCard,
            color: 0x3366ccff, // Blue
        }
    }

    pub fn move_card() -> Self {
        Self {
            id: random::u32(),
            name: "Move".into(),
            effect: CardEffect::MoveOneTile,
            color: 0x33cc33ff, // Green
        }
    }

    pub fn swap_card() -> Self {
        Self {
            id: random::u32(),
            name: "Swap".into(),
            effect: CardEffect::SwapCard,
            color: 0xffff00ff, // Yellow
        }
    }
}
