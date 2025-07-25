use turbo::Bounds;

pub fn point_in_bounds(x: i32, y: i32, bounds: &Bounds) -> bool {
    let bx = bounds.x();
    let by = bounds.y();
    let bw = bounds.w() as i32;
    let bh = bounds.h() as i32;

    x >= bx && x < bx + bw && y >= by && y < by + bh
}

pub fn lerp_color(from: u32, to: u32, t: f64) -> u32 {
    let from_r = ((from >> 24) & 0xff) as f64;
    let from_g = ((from >> 16) & 0xff) as f64;
    let from_b = ((from >> 8) & 0xff) as f64;
    let from_a = (from & 0xff) as f64;

    let to_r = ((to >> 24) & 0xff) as f64;
    let to_g = ((to >> 16) & 0xff) as f64;
    let to_b = ((to >> 8) & 0xff) as f64;
    let to_a = (to & 0xff) as f64;

    let r = ((1.0 - t) * from_r + t * to_r) as u32;
    let g = ((1.0 - t) * from_g + t * to_g) as u32;
    let b = ((1.0 - t) * from_b + t * to_b) as u32;
    let a = ((1.0 - t) * from_a + t * to_a) as u32;

    (r << 24) | (g << 16) | (b << 8) | a
}

/// Returns true if the drag card's outline rect intersects the slot's inner rect
pub fn rects_intersect_outline_to_inner(
    slot_x: u32,
    slot_y: u32,
    slot_w: u32,
    slot_h: u32,
    drag_x: u32,
    drag_y: u32,
    drag_w: u32,
    drag_h: u32,
    border_width: u32
) -> bool {
    let inset = border_width / 2;
    let slot_inner_x = slot_x + inset;
    let slot_inner_y = slot_y + inset;
    let slot_inner_w = slot_w.saturating_sub(border_width);
    let slot_inner_h = slot_h.saturating_sub(border_width);
    let slot_bounds = turbo::Bounds::new(slot_inner_x, slot_inner_y, slot_inner_w, slot_inner_h);
    let drag_bounds = turbo::Bounds::new(drag_x, drag_y, drag_w, drag_h);
    slot_bounds.intersects(&drag_bounds)
}

pub struct CardButtonGeometry {
    pub button_w: u32,
    pub button_h: u32,
    pub button_y: u32,
    pub inset: u32,
}

pub fn get_card_button_geometry(y: u32, w: u32, h: u32, padding: u32) -> CardButtonGeometry {
    let inset = padding / 2;
    let button_w = (w - padding * 3) / 2;
    let button_h = button_w;
    let button_y = y + h - inset - button_h;
    CardButtonGeometry { button_w, button_h, button_y, inset }
}
