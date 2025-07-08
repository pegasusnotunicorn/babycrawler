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
