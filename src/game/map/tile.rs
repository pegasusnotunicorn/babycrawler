use crate::game::constants::{
    FLASH_SPEED,
    FLOOR_COLOR,
    MAP_SIZE,
    WALL_COLOR,
    ENTRANCE_COUNT_WEIGHT_1,
    ENTRANCE_COUNT_WEIGHT_2,
    ENTRANCE_COUNT_WEIGHT_3,
    ENTRANCE_COUNT_WEIGHT_4,
};
use crate::game::util::lerp_color;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Tile {
    pub entrances: Vec<Direction>,
    pub is_highlighted: bool,
    #[serde(skip, default)]
    pub rotation_anim: Option<TileRotationAnim>,
    pub original_rotation: u8, // 0=0deg, 1=90deg, 2=180deg, 3=270deg
    pub current_rotation: u8, // 0=0deg, 1=90deg, 2=180deg, 3=270deg
}

#[derive(Clone, Debug, Default, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct TileRotationAnim {
    pub from_angle: f32,
    pub to_angle: f32,
    pub current_angle: f32,
    pub duration: f64,
    pub elapsed: f64,
    pub clockwise: bool,
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
            rotation_anim: None,
            original_rotation: 0,
            current_rotation: 0,
        }
    }

    pub fn random(forbidden: &[Direction]) -> Self {
        use crate::game::map::tile::Direction::*;
        let all_directions = [Up, Down, Left, Right];
        let allowed: Vec<Direction> = all_directions
            .iter()
            .cloned()
            .filter(|d| !forbidden.contains(d))
            .collect();
        let max_count = allowed.len().max(1); // always at least 1
        let all_weights = [
            ENTRANCE_COUNT_WEIGHT_1,
            ENTRANCE_COUNT_WEIGHT_2,
            ENTRANCE_COUNT_WEIGHT_3,
            ENTRANCE_COUNT_WEIGHT_4,
        ];
        let weights = &all_weights[..max_count];
        let entrance_count = random_weighted_entrance_count_dynamic(weights) as usize;
        let mut dirs = allowed.clone();
        random::shuffle(&mut dirs);
        let mut entrances = dirs.into_iter().take(entrance_count).collect::<Vec<_>>();
        // Ensure at least one entrance
        if entrances.is_empty() && !allowed.is_empty() {
            entrances.push(allowed[(random::u32() as usize) % allowed.len()]);
        }
        Tile::new(entrances)
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

    /// Find all reachable tiles from the current position through connected entrances
    pub fn find_reachable_tiles(&self, start_index: usize, tiles: &[Tile]) -> Vec<usize> {
        let mut visited = std::collections::HashSet::new();
        let mut reachable = Vec::new();
        let mut to_visit = vec![start_index];

        visited.insert(start_index);

        while let Some(current_index) = to_visit.pop() {
            reachable.push(current_index);

            let current_tile = &tiles[current_index];
            let (cx, cy) = Tile::position(current_index);

            // Check all four directions
            let directions = [
                (0, -1, Direction::Up, Direction::Down), // Up
                (0, 1, Direction::Down, Direction::Up), // Down
                (-1, 0, Direction::Left, Direction::Right), // Left
                (1, 0, Direction::Right, Direction::Left), // Right
            ];

            for (dx, dy, from_dir, to_dir) in directions {
                let nx = ((cx as isize) + dx) as usize;
                let ny = ((cy as isize) + dy) as usize;

                // Check bounds
                if nx >= MAP_SIZE || ny >= MAP_SIZE {
                    continue;
                }

                let next_index = Tile::index(nx, ny);

                // Skip if already visited
                if visited.contains(&next_index) {
                    continue;
                }

                let next_tile = &tiles[next_index];

                // Check if entrances connect
                if
                    current_tile.entrances.contains(&from_dir) &&
                    next_tile.entrances.contains(&to_dir)
                {
                    visited.insert(next_index);
                    to_visit.push(next_index);
                }
            }
        }

        reachable
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
        self.current_rotation = (self.current_rotation + (times as u8)) % 4;
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

        let angle = self.rotation_anim
            .as_ref()
            .map(|a| a.current_angle)
            .unwrap_or(0.0) as i32;
        let seg = ts / 3;

        // 🔳 Step 1: Draw outer border walls
        rect!(x = x, y = y, w = tile_size, h = tile_size, color = wall_color, rotation = angle);

        // 🟥 Step 2: Inner floor
        rect!(
            x = inner_x,
            y = inner_y,
            w = inner_size,
            h = inner_size,
            color = FLOOR_COLOR,
            rotation = angle
        );

        // 🔲 Step 3: Draw entrances as gaps in walls (only if not animating)
        if self.rotation_anim.is_none() {
            for dir in &self.entrances {
                match dir {
                    Direction::Up => {
                        rect!(
                            x = x + seg,
                            y = y,
                            w = ts - seg * 2,
                            h = wall_width,
                            color = FLOOR_COLOR,
                            rotation = angle
                        );
                    }
                    Direction::Down => {
                        rect!(
                            x = x + seg,
                            y = y + ts - wall_width,
                            w = ts - seg * 2,
                            h = wall_width,
                            color = FLOOR_COLOR,
                            rotation = angle
                        );
                    }
                    Direction::Left => {
                        rect!(
                            x = x,
                            y = y + seg,
                            w = wall_width,
                            h = ts - seg * 2,
                            color = FLOOR_COLOR,
                            rotation = angle
                        );
                    }
                    Direction::Right => {
                        rect!(
                            x = x + ts - wall_width,
                            y = y + seg,
                            w = wall_width,
                            h = ts - seg * 2,
                            color = FLOOR_COLOR,
                            rotation = angle
                        );
                    }
                }
            }
        }
    }
}

fn random_weighted_entrance_count_dynamic(weights: &[f32]) -> u8 {
    let total: f32 = weights.iter().sum();
    let mut pick = random::f32() * total;
    for (i, &weight) in weights.iter().enumerate() {
        if pick < weight {
            return (i + 1) as u8;
        }
        pick -= weight;
    }
    weights.len() as u8 // fallback, should not happen
}

pub fn clear_highlights(tiles: &mut [Tile]) {
    for tile in tiles.iter_mut() {
        tile.is_highlighted = false;
    }
}
