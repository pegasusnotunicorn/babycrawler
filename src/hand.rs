use crate::card::Card;
use crate::util::point_in_bounds;
use turbo::*;

pub fn draw_hand(
    hand: &[Card],
    selected_card: &Option<Card>,
    pointer_xy: (i32, i32),
    tile_size: u32,
    canvas_width: u32,
    mut on_card_click: impl FnMut(&Card) // <- changed to FnMut
) {
    let spacing = 6;
    let x = canvas_width - tile_size - 8;
    let mx = pointer_xy.0;
    let my = pointer_xy.1;

    for (i, card) in hand.iter().enumerate() {
        let y = 8 + (i as u32) * (tile_size + spacing);
        let bounds = Bounds::new(x, y, tile_size, tile_size);
        let is_hovered = point_in_bounds(mx, my, &bounds);
        let is_selected = selected_card.as_ref() == Some(card);

        let bg_color = if is_selected { 0xffffffff } else { card.color };
        let border_color = if is_hovered { 0xffff00ff } else { bg_color };

        // Draw card background
        rect!(x = x, y = y, w = tile_size, h = tile_size, color = bg_color);

        let left = x;
        let right = x + tile_size - 1;
        let top = y;
        let bottom = y + tile_size - 1;

        // Top edge
        path!(start = (left, top), end = (right, top), size = 1, color = border_color);

        // Right edge
        path!(start = (right, top), end = (right, bottom), size = 1, color = border_color);

        // Bottom edge
        path!(start = (right + 1, bottom), end = (left, bottom), size = 1, color = border_color);

        // Left edge
        path!(start = (left, bottom), end = (left, top), size = 1, color = border_color);

        // Draw card label
        text!(&card.name, x = x + 4, y = y + 4, font = "small", color = 0x000000ff);

        if is_hovered && turbo::mouse::screen().pressed() {
            on_card_click(card);
        }
    }
}
