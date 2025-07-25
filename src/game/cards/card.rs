use crate::game::cards::card_effect::CardEffect;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };
use crate::game::constants::{
    GAME_PADDING,
    CARD_HOVER_FILL_COLOR,
    GAME_BG_COLOR,
    FLASH_SPEED,
    CARD_DUMMY_COLOR,
    CARD_BUTTON_A_COLOR,
    CARD_BUTTON_B_COLOR,
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
}

const CARD_CONSTRUCTORS: &[fn() -> Card] = &[Card::move_card, Card::rotate_card, Card::swap_card];

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

    pub fn dummy_card() -> Self {
        Self {
            id: 0,
            name: String::new(),
            effect: CardEffect::Dummy,
            color: CARD_DUMMY_COLOR,
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
                color = CARD_HOVER_FILL_COLOR,
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
        // Selected state: flashing outline
        if visual_state.contains(CardVisualState::SELECTED) {
            let t: f64 = frame.unwrap_or(0.0) * FLASH_SPEED;
            let t = t.sin() * 0.5 + 0.5;
            let alpha = (t * 255.0) as u32;
            let flash_color: u32 = (0xff << 24) | (0xff << 16) | (0xff << 8) | alpha;
            rect!(x = x, y = y, w = w, h = h, color = flash_color, border_radius = border_radius);
        } else if outline {
            rect!(x = x, y = y, w = w, h = h, color = outline_color, border_radius = border_radius);
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
        // Always draw hover overlay if HOVERED is set (even for dummy)
        if visual_state.contains(CardVisualState::HOVERED) {
            let highlight_color = CARD_HOVER_FILL_COLOR;
            rect!(
                x = inner_x,
                y = inner_y,
                w = inner_w,
                h = inner_h,
                color = highlight_color,
                border_radius = border_radius
            );
        }
        // Draw label if not dummy
        if !visual_state.contains(CardVisualState::DUMMY) {
            let border_width = GAME_PADDING;
            let inset = border_width / 2;
            let label_x = x + inset + 4;
            let label_y = y + inset + 4;
            text!(&self.name, x = label_x, y = label_y, font = "large", color = 0x000000ff);
        }
    }
}

pub fn draw_card_buttons(x: u32, y: u32, w: u32, h: u32, pointer_xy: (i32, i32)) {
    let border_width = GAME_PADDING;
    let inset_x = border_width / 2;
    let inset_y = border_width;
    let button_w = (w - GAME_PADDING * 3) / 2;
    let button_h = button_w;
    let button_y = y + h - inset_y - button_h;
    let pointer_bounds = turbo::Bounds::new(pointer_xy.0 as u32, pointer_xy.1 as u32, 1, 1);
    let button_specs = [
        ("B", CARD_BUTTON_B_COLOR, x + inset_x + GAME_PADDING / 2),
        ("A", CARD_BUTTON_A_COLOR, x + w + inset_x - inset_y - GAME_PADDING / 2 - button_w),
    ];
    for (label, color, bx) in button_specs {
        let bounds = turbo::Bounds::new(bx, button_y, button_w, button_h);
        let hovered = bounds.contains(&pointer_bounds);
        Card::draw_button(label, bx, button_y, button_w, button_h, color, hovered);
    }
}
