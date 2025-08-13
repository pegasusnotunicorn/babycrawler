use crate::game::cards::card_effect::CardEffect;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };
use crate::game::constants::{
    GAME_PADDING,
    CARD_HOVER_OUTLINE_COLOR,
    GAME_BG_COLOR,
    FLASH_SPEED,
    CARD_DUMMY_COLOR,
};
use bitflags::bitflags;

bitflags! {
    pub struct CardVisualState: u8 {
        const NONE     = 0b0000;
        const HOVERED  = 0b0001;
        const SELECTED = 0b0010;
        const DUMMY    = 0b0100;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Card {
    pub id: u32,
    pub name: String,
    pub effect: CardEffect,
    pub color: u32,
    pub hand_index: Option<usize>, // Track which hand slot this card came from
    pub hide_confirm_button: bool,
}

const CARD_CONSTRUCTORS: &[fn() -> Card] = &[
    Card::move_card,
    Card::rotate_card,
    Card::swap_card,
    Card::fire_card,
];

impl Card {
    pub fn toggle_selection(selected: &mut Option<Card>, card: &Card) {
        if selected.as_ref() == Some(card) {
            *selected = None;
        } else {
            *selected = Some(card.clone());
        }
    }

    pub fn random() -> Self {
        let index = (random::u32() as usize) % CARD_CONSTRUCTORS.len();
        (CARD_CONSTRUCTORS[index])()
    }

    pub fn get_unique_cards() -> Vec<Self> {
        vec![Self::move_card(), Self::rotate_card(), Self::swap_card(), Self::fire_card()]
    }

    pub fn rotate_card() -> Self {
        Self {
            id: random::u32(),
            name: "TURN".into(),
            effect: CardEffect::RotateCard,
            color: 0xb804b8ff, // Light purple
            hand_index: None,
            hide_confirm_button: false,
        }
    }

    pub fn move_card() -> Self {
        Self {
            id: random::u32(),
            name: "MOVE".into(),
            effect: CardEffect::MoveOneTile,
            color: 0x00aa00ff, // Dark green
            hand_index: None,
            hide_confirm_button: false,
        }
    }

    pub fn swap_card() -> Self {
        Self {
            id: random::u32(),
            name: "SWAP".into(),
            effect: CardEffect::SwapCard,
            color: 0xff8800ff, // Dark orange
            hand_index: None,
            hide_confirm_button: false,
        }
    }

    pub fn fire_card() -> Self {
        Self {
            id: random::u32(),
            name: "FIRE".into(),
            effect: CardEffect::FireCard,
            color: 0x8b0000ff, // Dark red
            hand_index: None,
            hide_confirm_button: true,
        }
    }

    pub fn dummy_card() -> Self {
        Self {
            id: 0,
            name: String::new(),
            effect: CardEffect::Dummy,
            color: CARD_DUMMY_COLOR,
            hand_index: None,
            hide_confirm_button: false,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.id == 0
    }

    pub fn draw_button(
        label: &str,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        fill_color: u32,
        hovered: bool
    ) {
        // Draw button outline (black)
        rect!(x = x, y = y, w = w, h = h, color = 0x000000ff, border_radius = h / 3);
        // Draw button fill (slightly inset)
        let inset = 2;
        rect!(
            x = x + inset,
            y = y + inset,
            w = w.saturating_sub(inset * 2),
            h = h.saturating_sub(inset * 2),
            color = fill_color,
            border_radius = h / 3 - 1
        );
        // Draw hover overlay if hovered
        if hovered {
            rect!(
                x = x + inset,
                y = y + inset,
                w = w.saturating_sub(inset * 2),
                h = h.saturating_sub(inset * 2),
                color = CARD_HOVER_OUTLINE_COLOR,
                border_radius = h / 3 - 1
            );
        }
        // Draw label centered
        // For horizontal centering, estimate font width as w/2 (since it's a single char)
        let label_x = x + w / 2 - w / 8;
        let label_y = y + h / 2 - h / 5;
        text!(label, x = label_x, y = label_y, font = "large", color = 0xffffffff);
    }

    pub fn draw(
        &self,
        x: u32,
        y: u32,
        w: u32,
        h: u32,
        outline_color: u32,
        outline: bool,
        visual_state: CardVisualState,
        frame: Option<f64>
    ) {
        let border_radius = GAME_PADDING / 2;
        let border_width = GAME_PADDING;
        let inset = border_width / 2;
        let inner_x = x + inset;
        let inner_y = y + inset;
        let inner_w = w.saturating_sub(border_width);
        let inner_h = h.saturating_sub(border_width);

        // Outline
        if visual_state.contains(CardVisualState::SELECTED) {
            let t: f64 = frame.unwrap_or(0.0) * FLASH_SPEED;
            let t = t.sin() * 0.5 + 0.5;
            let alpha = (t * 255.0) as u32;
            let flash_color: u32 = (0xff << 24) | (0xff << 16) | (0xff << 8) | alpha;
            rect!(x = x, y = y, w = w, h = h, color = flash_color, border_radius = border_radius);
        } else if outline {
            let hovered = visual_state.contains(CardVisualState::HOVERED);
            let color = if hovered { CARD_HOVER_OUTLINE_COLOR } else { outline_color };
            rect!(x = x, y = y, w = w, h = h, color = color, border_radius = border_radius);
        }

        // Card fill
        let fill_color = if visual_state.contains(CardVisualState::DUMMY) {
            GAME_BG_COLOR
        } else {
            self.color
        };
        rect!(
            x = inner_x,
            y = inner_y,
            w = inner_w,
            h = inner_h,
            color = fill_color,
            border_radius = border_radius
        );

        // Draw label if not dummy
        if !visual_state.contains(CardVisualState::DUMMY) {
            let border_width = GAME_PADDING;
            let inset = border_width / 2;
            let label_x = x + inset + GAME_PADDING * 2 - 8;
            let label_y = y + inset + GAME_PADDING;
            text!(&self.name, x = label_x, y = label_y, font = "large", color = 0x000000ff);
        }
    }
}
