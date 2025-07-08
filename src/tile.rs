use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };

use crate::{ constants::{ FLASH_SPEED, FLOOR_COLOR, WALL_COLOR }, util::lerp_color };

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
        absolute_x: i32,
        absolute_y: i32,
        tile_size: u32,
        should_highlight: bool,
        frame: f64
    ) {
        let wall_width = 5 as i32;
        let inner_size = tile_size.saturating_sub((wall_width as u32) * 2);
        let inner_x = absolute_x + (wall_width as i32);
        let inner_y = absolute_y + (wall_width as i32);
        let ts = tile_size as i32;

        let t = (frame * FLASH_SPEED).sin() * 0.5 + 0.5;
        let wall_color = if should_highlight {
            lerp_color(0xffffffff, WALL_COLOR, t)
        } else {
            WALL_COLOR
        };

        // 🔳 Step 1: Draw outer border walls as a big rect
        rect!(x = absolute_x, y = absolute_y, w = tile_size, h = tile_size, color = wall_color);

        // 🟥 Step 2: Draw inner background as floor
        rect!(x = inner_x, y = inner_y, w = inner_size, h = inner_size, color = FLOOR_COLOR);

        // 🔲 Step 3: Draw gaps in the walls (entrances) as small rects "cutting through" walls
        let seg = ts / 3;

        for dir in &self.entrances {
            match dir {
                Direction::Up => {
                    rect!(
                        x = absolute_x + seg,
                        y = absolute_y,
                        w = ts - seg * 2,
                        h = wall_width,
                        color = FLOOR_COLOR
                    );
                }
                Direction::Down => {
                    rect!(
                        x = absolute_x + seg,
                        y = absolute_y + ts - wall_width,
                        w = ts - seg * 2,
                        h = wall_width,
                        color = FLOOR_COLOR
                    );
                }
                Direction::Left => {
                    rect!(
                        x = absolute_x,
                        y = absolute_y + seg,
                        w = wall_width,
                        h = ts - seg * 2,
                        color = FLOOR_COLOR
                    );
                }
                Direction::Right => {
                    rect!(
                        x = absolute_x + ts - wall_width,
                        y = absolute_y + seg,
                        w = wall_width,
                        h = ts - seg * 2,
                        color = FLOOR_COLOR
                    );
                }
            }
        }
    }
}
