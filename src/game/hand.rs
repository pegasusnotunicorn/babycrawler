use crate::game::card::Card;
use crate::game::constants::{ FLASH_SPEED, GAME_PADDING };
use crate::game::util::point_in_bounds;
use turbo::*;

pub fn draw_hand(
    hand: &[Card],
    selected_cards: &[Card],
    tile_size: u32,
    frame: f64,
    mut on_card_click: impl FnMut(&Card)
) {
    if hand.is_empty() {
        return;
    }

    let pointer = mouse::screen();
    let pointer_xy = (pointer.x, pointer.y);
    let canvas_width = bounds::screen().w() - GAME_PADDING * 2;
    let canvas_height = bounds::screen().h();

    let card_width = canvas_width / (hand.len() as u32);
    let card_height = card_width.min(canvas_height / 5);
    let y = canvas_height - card_height - GAME_PADDING;
    let offset_x = GAME_PADDING;

    let mx = pointer_xy.0;
    let my = pointer_xy.1;

    let border_width = tile_size / 4;
    let border_radius = tile_size / 16;
    let inset = border_width / 2;

    for (i, card) in hand.iter().enumerate() {
        let x = (i as u32) * card_width + offset_x;
        let bounds = Bounds::new(x, y, card_width, card_height);
        let is_hovered = point_in_bounds(mx, my, &bounds);
        let is_selected = selected_cards.contains(card);

        // Inset dimensions to make room for overlay-style border
        let inner_x = x + inset;
        let inner_y = y + inset;
        let inner_w = card_width.saturating_sub(border_width);
        let inner_h = card_height.saturating_sub(border_width);

        if is_selected {
            let t = (frame * FLASH_SPEED).sin() * 0.5 + 0.5;
            let alpha = (t * 255.0) as u32;
            let flash_color: u32 = (0xff << 24) | (0xff << 16) | (0xff << 8) | alpha;

            rect!(
                x = x,
                y = y,
                w = card_width,
                h = card_height,
                color = flash_color,
                border_radius = border_radius
            );
        }

        // Background fill
        rect!(
            x = inner_x,
            y = inner_y,
            w = inner_w,
            h = inner_h,
            color = card.color,
            border_radius = border_radius
        );

        // Hover overlay
        if is_hovered {
            let highlight_color = 0xffffff80;
            rect!(
                x = inner_x,
                y = inner_y,
                w = inner_w,
                h = inner_h,
                color = highlight_color,
                border_radius = border_radius
            );
        }

        // Card label
        text!(&card.name, x = inner_x + 4, y = inner_y + 4, font = "large", color = 0x000000ff);

        if is_hovered && pointer.just_pressed() {
            on_card_click(card);
        }
    }
}
