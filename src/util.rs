use turbo::Bounds;

pub fn point_in_bounds(x: i32, y: i32, bounds: &Bounds) -> bool {
    let bx = bounds.x();
    let by = bounds.y();
    let bw = bounds.w() as i32;
    let bh = bounds.h() as i32;

    x >= bx && x < bx + bw && y >= by && y < by + bh
}
