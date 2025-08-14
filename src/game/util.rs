use turbo::Bounds;

pub fn point_in_bounds(x: i32, y: i32, bounds: &Bounds) -> bool {
    let bx = bounds.x();
    let by = bounds.y();
    let bw = bounds.w() as i32;
    let bh = bounds.h() as i32;

    x >= bx && x < bx + bw && y >= by && y < by + bh
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
