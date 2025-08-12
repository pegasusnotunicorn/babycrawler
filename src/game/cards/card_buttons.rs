use turbo::*;
use crate::game::constants::{ GAME_PADDING, CARD_BUTTON_A_COLOR, CARD_BUTTON_B_COLOR };

#[derive(Debug, Clone)]
pub struct CardButton {
    pub button_w: u32,
    pub button_h: u32,
    pub button_y: u32,
    pub inset: u32,
}

impl CardButton {
    pub fn new(card_y: u32, card_w: u32, card_h: u32) -> Self {
        let inset = GAME_PADDING / 2;
        let button_w = (card_w - GAME_PADDING * 3) / 2;
        let button_h = button_w;
        let button_y = card_y + card_h - GAME_PADDING - button_h;

        Self {
            button_w,
            button_h,
            button_y,
            inset,
        }
    }

    pub fn get_button_positions(&self, card_x: u32, card_w: u32) -> [(u32, u32); 2] {
        let button_a_x = card_x + self.inset + GAME_PADDING / 2;
        let button_b_x = card_x + card_w - self.inset - GAME_PADDING / 2 - self.button_w;

        [
            (button_a_x, self.button_y),
            (button_b_x, self.button_y),
        ]
    }

    pub fn draw_buttons(&self, card_x: u32, card_w: u32, pointer_xy: (i32, i32)) {
        let button_positions = self.get_button_positions(card_x, card_w);
        let pointer_bounds = turbo::Bounds::new(pointer_xy.0 as u32, pointer_xy.1 as u32, 1, 1);

        let button_specs = [
            ("B", CARD_BUTTON_B_COLOR, button_positions[0]),
            ("A", CARD_BUTTON_A_COLOR, button_positions[1]),
        ];

        for (label, color, (bx, by)) in button_specs {
            let bounds = turbo::Bounds::new(bx, by, self.button_w, self.button_h);
            let hovered = bounds.contains(&pointer_bounds);
            self.draw_button(label, bx, by, color, hovered);
        }
    }

    fn draw_button(&self, label: &str, x: u32, y: u32, color: u32, hovered: bool) {
        rect!(x = x, y = y, w = self.button_w, h = self.button_h, color = color, border_radius = 4);
        text!(
            label,
            x = x + self.button_w / 2,
            y = y + self.button_h / 2,
            font = "large",
            color = 0x000000ff
        );

        if hovered {
            // Draw a semi-transparent white rectangle over the button when hovered
            rect!(
                x = x,
                y = y,
                w = self.button_w,
                h = self.button_h,
                color = 0xffffff70,
                border_radius = 4
            );
        }
    }
}

// Convenience function for backward compatibility
pub fn draw_card_buttons(x: u32, y: u32, w: u32, h: u32, pointer_xy: (i32, i32)) {
    let geometry = CardButton::new(y, w, h);
    geometry.draw_buttons(x, w, pointer_xy);
}

/// Determines if a card should show buttons based on whether it's the selected card
pub fn should_show_buttons(
    card: &crate::game::cards::card::Card,
    selected_card: Option<&crate::game::cards::card::Card>
) -> bool {
    // Only show buttons if the card is not a dummy and it's the selected card
    !card.is_dummy() && selected_card.map_or(false, |selected| selected == card)
}
