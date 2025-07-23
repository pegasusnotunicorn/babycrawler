use crate::game::card_effect::CardEffect;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };
use crate::game::constants::{ GAME_PADDING, CARD_HOVER_COLOR };

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
    pub fn toggle_selection(selected: &mut Vec<Card>, card: &Card) {
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
            color: 0xffa500ff, // Orange
        }
    }

    pub fn draw_rect(
        &self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        outline_color: u32,
        outline: bool,
        is_hovered: bool,
        is_selected: bool,
        frame: Option<f64>
    ) {
        let border_radius = GAME_PADDING / 2;
        let border_width = GAME_PADDING;
        let inset = border_width / 2;
        let inner_x = x + inset;
        let inner_y = y + inset;
        let inner_w = w.saturating_sub(border_width);
        let inner_h = h.saturating_sub(border_width);
        // Selected state: flashing outline
        if is_selected {
            let t: f64 = frame.unwrap_or(0.0) * crate::game::constants::FLASH_SPEED;
            let t = t.sin() * 0.5 + 0.5;
            let alpha = (t * 255.0) as u32;
            let flash_color: u32 = (0xff << 24) | (0xff << 16) | (0xff << 8) | alpha;
            rect!(x = x, y = y, w = w, h = h, color = flash_color, border_radius = border_radius);
        } else if outline {
            rect!(x = x, y = y, w = w, h = h, color = outline_color, border_radius = border_radius);
        }
        // Card fill
        rect!(
            x = inner_x,
            y = inner_y,
            w = inner_w,
            h = inner_h,
            color = self.color,
            border_radius = border_radius
        );
        // Hover overlay
        if is_hovered {
            let highlight_color = CARD_HOVER_COLOR;
            rect!(
                x = inner_x,
                y = inner_y,
                w = inner_w,
                h = inner_h,
                color = highlight_color,
                border_radius = border_radius
            );
        }
    }
}
