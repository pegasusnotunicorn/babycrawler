use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Deserialize, Serialize };

use crate::{ constants::{ FLASH_SPEED, FLOOR_COLOR, MAP_SIZE, WALL_COLOR }, util::lerp_color };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Tile {
    pub entrances: Vec<Direction>,
    pub is_highlighted: bool,
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
    pub fn new(entrances: Vec<Direction>) -> Self {
        Self {
            entrances,
            is_highlighted: false,
        }
    }

    /// Given a tile index, return (x, y)
    pub fn position(index: usize) -> (usize, usize) {
        (index % MAP_SIZE, index / MAP_SIZE)
    }

    /// Return x from index
    pub fn x(index: usize) -> usize {
        index % MAP_SIZE
    }

    /// Return y from index
    pub fn y(index: usize) -> usize {
        index / MAP_SIZE
    }

    /// Convert (x, y) to tile index
    pub fn index(x: usize, y: usize) -> usize {
        y * MAP_SIZE + x
    }

    /// Given a tile index and tile_size, return (tx, ty) screen coordinates
    pub fn screen_position(
        index: usize,
        tile_size: u32,
        offset_x: u32,
        offset_y: u32
    ) -> (u32, u32) {
        let (x, y) = Self::position(index);
        let tx = offset_x + (x as u32) * tile_size;
        let ty = offset_y + (y as u32) * tile_size;
        (tx, ty)
    }

    /// Returns all valid indices adjacent to (px, py)
    pub fn get_adjacent_indices(
        origin_index: usize,
        include_diagonals: bool,
        include_self: bool
    ) -> Vec<usize> {
        let (px, py) = Tile::position(origin_index);
        let mut indices = vec![];

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let dx = ((x as isize) - (px as isize)).abs();
                let dy = ((y as isize) - (py as isize)).abs();

                let is_self = dx == 0 && dy == 0;
                if !include_self && is_self {
                    continue;
                }

                let is_diagonal = dx == 1 && dy == 1;
                let is_adjacent = dx + dy == 1;

                if
                    (include_diagonals && (is_adjacent || is_diagonal || is_self)) ||
                    (!include_diagonals && is_adjacent)
                {
                    indices.push(Tile::index(x, y));
                }
            }
        }

        indices
    }

    /// Whether self can move to `other`, based on shared entrances
    pub fn can_move_to(&self, self_index: usize, other: &Tile, other_index: usize) -> bool {
        use Direction::*;

        let (sx, sy) = Tile::position(self_index);
        let (ox, oy) = Tile::position(other_index);

        let dx = (ox as isize) - (sx as isize);
        let dy = (oy as isize) - (sy as isize);

        match (dx, dy) {
            (0, -1) => self.entrances.contains(&Up) && other.entrances.contains(&Down),
            (0, 1) => self.entrances.contains(&Down) && other.entrances.contains(&Up),
            (-1, 0) => self.entrances.contains(&Left) && other.entrances.contains(&Right),
            (1, 0) => self.entrances.contains(&Right) && other.entrances.contains(&Left),
            _ => false,
        }
    }

    /// Swaps state with another tile, including grid position and entrances
    pub fn swap_with(&mut self, other: &mut Tile) {
        std::mem::swap(&mut self.entrances, &mut other.entrances);
        std::mem::swap(&mut self.is_highlighted, &mut other.is_highlighted);
    }

    /// Rotate all entrances clockwise
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

    /// Draws the tile, including walls and floor and optional highlight pulse
    pub fn draw(&self, x: i32, y: i32, tile_size: u32, should_highlight: bool, frame: f64) {
        let wall_width = 5;
        let inner_size = tile_size.saturating_sub((wall_width as u32) * 2);
        let inner_x = x + wall_width;
        let inner_y = y + wall_width;
        let ts = tile_size as i32;

        let t = (frame * FLASH_SPEED).sin() * 0.5 + 0.5;
        let wall_color = if should_highlight {
            lerp_color(0xffffffff, WALL_COLOR, t)
        } else {
            WALL_COLOR
        };

        // 🔳 Step 1: Draw outer border walls
        rect!(x = x, y = y, w = tile_size, h = tile_size, color = wall_color);

        // 🟥 Step 2: Inner floor
        rect!(x = inner_x, y = inner_y, w = inner_size, h = inner_size, color = FLOOR_COLOR);

        // 🔲 Step 3: Draw entrances as gaps in walls
        let seg = ts / 3;
        for dir in &self.entrances {
            match dir {
                Direction::Up => {
                    rect!(
                        x = x + seg,
                        y = y,
                        w = ts - seg * 2,
                        h = wall_width,
                        color = FLOOR_COLOR
                    );
                }
                Direction::Down => {
                    rect!(
                        x = x + seg,
                        y = y + ts - wall_width,
                        w = ts - seg * 2,
                        h = wall_width,
                        color = FLOOR_COLOR
                    );
                }
                Direction::Left => {
                    rect!(
                        x = x,
                        y = y + seg,
                        w = wall_width,
                        h = ts - seg * 2,
                        color = FLOOR_COLOR
                    );
                }
                Direction::Right => {
                    rect!(
                        x = x + ts - wall_width,
                        y = y + seg,
                        w = wall_width,
                        h = ts - seg * 2,
                        color = FLOOR_COLOR
                    );
                }
            }
        }
    }
}

/// Helper function to turn off all highlights
pub fn clear_highlights(tiles: &mut [Tile]) {
    for tile in tiles.iter_mut() {
        tile.is_highlighted = false;
    }
}
