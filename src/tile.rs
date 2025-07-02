use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Tile {
    pub grid_x: usize,
    pub grid_y: usize,
    pub entrances: Vec<Direction>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    BorshDeserialize,
    BorshSerialize,
    Serialize,
    Deserialize
)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Tile {
    pub fn new(x: usize, y: usize, entrances: Vec<Direction>) -> Self {
        Self { grid_x: x, grid_y: y, entrances }
    }

    pub fn can_move_to(&self, other: &Tile) -> bool {
        use Direction::*;
        let dx = (other.grid_x as isize) - (self.grid_x as isize);
        let dy = (other.grid_y as isize) - (self.grid_y as isize);

        match (dx, dy) {
            (0, -1) => self.entrances.contains(&Up) && other.entrances.contains(&Down),
            (0, 1) => self.entrances.contains(&Down) && other.entrances.contains(&Up),
            (-1, 0) => self.entrances.contains(&Left) && other.entrances.contains(&Right),
            (1, 0) => self.entrances.contains(&Right) && other.entrances.contains(&Left),
            _ => false, // not adjacent
        }
    }

    pub fn draw_highlighted(
        &self,
        absolute_x: i32,
        absolute_y: i32,
        tile_size: u32,
        hovered: bool,
        frame: f64
    ) {
        let t = (frame * 0.2).sin() * 0.5 + 0.5;
        let alpha = (t * 255.0) as u32;
        let flash_color = (0xff << 24) | (0xff << 16) | (0xff << 8) | alpha;

        self.draw_at_absolute(flash_color, absolute_x, absolute_y, tile_size, hovered);
    }

    pub fn rotate_clockwise(&mut self, times: usize) {
        use Direction::*;
        self.entrances = self.entrances
            .iter()
            .map(|dir| {
                let mut n = match dir {
                    Up => 0,
                    Right => 1,
                    Down => 2,
                    Left => 3,
                };
                n = (n + times) % 4;
                match n {
                    0 => Up,
                    1 => Right,
                    2 => Down,
                    3 => Left,
                    _ => unreachable!(),
                }
            })
            .collect();
    }

    pub fn draw_at_absolute(
        &self,
        base_color: u32,
        absolute_x: i32,
        absolute_y: i32,
        tile_size: u32,
        hovered: bool
    ) {
        let color = if hovered { 0xffff00ff } else { 0xffffffff };
        let wall_width = 1;
        let segment = (tile_size / 3) as i32;
        let i32_tile_size = tile_size as i32;

        // Draw background
        rect!(x = absolute_x, y = absolute_y, w = tile_size, h = tile_size, color = base_color);

        // UP wall
        if self.entrances.contains(&Direction::Up) {
            path!(
                start = (absolute_x, absolute_y),
                end = (absolute_x + segment, absolute_y),
                size = wall_width,
                color = color
            );
            path!(
                start = (absolute_x + i32_tile_size - segment, absolute_y),
                end = (absolute_x + i32_tile_size, absolute_y),
                size = wall_width,
                color = color
            );
        } else {
            path!(
                start = (absolute_x, absolute_y),
                end = (absolute_x + i32_tile_size, absolute_y),
                size = wall_width,
                color = color
            );
        }

        // DOWN wall
        let y = absolute_y + i32_tile_size - wall_width;
        if self.entrances.contains(&Direction::Down) {
            path!(
                start = (absolute_x, y),
                end = (absolute_x + segment, y),
                size = wall_width,
                color = color
            );
            path!(
                start = (absolute_x + i32_tile_size - segment, y),
                end = (absolute_x + i32_tile_size, y),
                size = wall_width,
                color = color
            );
        } else {
            path!(
                start = (absolute_x, y),
                end = (absolute_x + i32_tile_size, y),
                size = wall_width,
                color = color
            );
        }

        // LEFT wall
        if self.entrances.contains(&Direction::Left) {
            path!(
                start = (absolute_x, absolute_y),
                end = (absolute_x, absolute_y + segment),
                size = wall_width,
                color = color
            );
            path!(
                start = (absolute_x, absolute_y + i32_tile_size - segment),
                end = (absolute_x, absolute_y + i32_tile_size),
                size = wall_width,
                color = color
            );
        } else {
            path!(
                start = (absolute_x, absolute_y),
                end = (absolute_x, absolute_y + i32_tile_size),
                size = wall_width,
                color = color
            );
        }

        // RIGHT wall
        let x = absolute_x + i32_tile_size - wall_width;
        if self.entrances.contains(&Direction::Right) {
            path!(
                start = (x, absolute_y),
                end = (x, absolute_y + segment),
                size = wall_width,
                color = color
            );
            path!(
                start = (x, absolute_y + i32_tile_size - segment),
                end = (x, absolute_y + i32_tile_size),
                size = wall_width,
                color = color
            );
        } else {
            path!(
                start = (x, absolute_y),
                end = (x, absolute_y + i32_tile_size),
                size = wall_width,
                color = color
            );
        }
    }
}
