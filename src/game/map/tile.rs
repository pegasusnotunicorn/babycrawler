use crate::game::constants::{
    DEBUG_MODE,
    FLASH_SPEED,
    MAP_SIZE,
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
    #[serde(skip, default)]
    pub pending_rotation: Option<PendingRotation>,
    pub original_rotation: u8, // 0=0deg, 1=90deg, 2=180deg, 3=270deg
    pub original_location: usize, // original tile index position
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

#[derive(Clone, Debug, Default, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PendingRotation {
    pub target: u8,
    pub timer: f64,
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
            pending_rotation: None,
            original_rotation: 0,
            original_location: 0,
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

    /// Find the shortest walkable path between two tiles using BFS
    pub fn find_walkable_path(
        start_index: usize,
        target_index: usize,
        tiles: &[Tile]
    ) -> Option<Vec<usize>> {
        if start_index == target_index {
            return Some(vec![start_index]);
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parent = std::collections::HashMap::new();

        queue.push_back(start_index);
        visited.insert(start_index);

        while let Some(current_index) = queue.pop_front() {
            if current_index == target_index {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = target_index;
                while current != start_index {
                    path.push(current);
                    current = parent[&current];
                }
                path.push(start_index);
                path.reverse();
                return Some(path);
            }

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
                    parent.insert(next_index, current_index);
                    queue.push_back(next_index);
                }
            }
        }

        None // No path found
    }

    /// Check if this tile is connected to an adjacent tile in the given direction
    /// Returns true if both tiles have entrances that connect to each other
    pub fn is_connected_in_direction(&self, direction: Direction, adjacent_tile: &Tile) -> bool {
        match direction {
            Direction::Up => {
                self.entrances.contains(&Direction::Up) &&
                    adjacent_tile.entrances.contains(&Direction::Down)
            }
            Direction::Down => {
                self.entrances.contains(&Direction::Down) &&
                    adjacent_tile.entrances.contains(&Direction::Up)
            }
            Direction::Left => {
                self.entrances.contains(&Direction::Left) &&
                    adjacent_tile.entrances.contains(&Direction::Right)
            }
            Direction::Right => {
                self.entrances.contains(&Direction::Right) &&
                    adjacent_tile.entrances.contains(&Direction::Left)
            }
        }
    }

    /// Finds all tiles in a straight line that have connected entrances
    /// Returns a vector of tile indices that form a connected path
    pub fn find_connected_line(
        start_index: usize,
        direction: Direction,
        tiles: &[Tile],
        max_distance: Option<usize>
    ) -> Vec<usize> {
        let mut connected_tiles = vec![start_index];
        let mut current_index = start_index;
        let max_dist = max_distance.unwrap_or(MAP_SIZE);

        for _ in 0..max_dist {
            let next_index = match direction {
                Direction::Up => {
                    if current_index < MAP_SIZE {
                        break;
                    }
                    current_index - MAP_SIZE
                }
                Direction::Down => {
                    let new_index = current_index + MAP_SIZE;
                    if new_index >= tiles.len() {
                        break;
                    }
                    new_index
                }
                Direction::Left => {
                    if current_index % MAP_SIZE == 0 {
                        break;
                    }
                    current_index - 1
                }
                Direction::Right => {
                    if (current_index + 1) % MAP_SIZE == 0 {
                        break;
                    }
                    current_index + 1
                }
            };

            // Check if we've reached the edge or if the tiles are connected
            if next_index == current_index || next_index >= tiles.len() {
                break;
            }

            let current_tile = &tiles[current_index];
            let next_tile = &tiles[next_index];

            // Check if the tiles have matching entrances
            let has_connection = match direction {
                Direction::Up => {
                    current_tile.entrances.contains(&Direction::Up) &&
                        next_tile.entrances.contains(&Direction::Down)
                }
                Direction::Down => {
                    current_tile.entrances.contains(&Direction::Down) &&
                        next_tile.entrances.contains(&Direction::Up)
                }
                Direction::Left => {
                    current_tile.entrances.contains(&Direction::Left) &&
                        next_tile.entrances.contains(&Direction::Right)
                }
                Direction::Right => {
                    current_tile.entrances.contains(&Direction::Right) &&
                        next_tile.entrances.contains(&Direction::Left)
                }
            };

            if has_connection {
                connected_tiles.push(next_index);
                current_index = next_index;
            } else {
                break; // No connection, stop the line
            }
        }

        connected_tiles
    }

    /// Swaps state with another tile, including grid position and entrances
    pub fn swap_with(&mut self, other: &mut Tile) {
        std::mem::swap(&mut self.entrances, &mut other.entrances);
        std::mem::swap(&mut self.is_highlighted, &mut other.is_highlighted);
    }

    // Rotate the entrances to a specific rotation
    pub fn rotate_entrances(&mut self, rotation: u8) {
        use Direction::*;

        // Calculate how many 90-degree rotations we need to apply from the current state
        let rotations_needed = (4 + (rotation as i32) - (self.current_rotation as i32)) % 4;

        self.entrances = self.entrances
            .iter()
            .map(|dir| {
                let mut n = match dir {
                    Up => 0,
                    Right => 1,
                    Down => 2,
                    Left => 3,
                };
                // Apply the calculated rotations
                n = (n + (rotations_needed as usize)) % 4;
                let result = match n {
                    0 => Up,
                    1 => Right,
                    2 => Down,
                    3 => Left,
                    _ => unreachable!(),
                };
                result
            })
            .collect();

        self.current_rotation = rotation;
    }

    pub fn draw(
        &self,
        x: i32,
        y: i32,
        tile_size: u32,
        should_highlight: bool,
        frame: f64,
        is_swap_selected: bool
    ) {
        let (sprite_name, rotation_offset) = self.get_wall_sprite_and_rotation();

        // Calculate the final rotation including any animation
        let final_rotation = if let Some(anim) = &self.rotation_anim {
            anim.current_angle + rotation_offset
        } else {
            // Use the rotation offset from sprite selection
            rotation_offset
        };

        sprite!(sprite_name, x = x, y = y, w = tile_size, h = tile_size, rotation = final_rotation);

        if DEBUG_MODE {
            // Draw entrance text (URDL) on the tile
            let entrance_text = self.get_entrance_text();
            if !entrance_text.is_empty() {
                let text_x = x + (tile_size as i32) / 2;
                let text_y = y + (tile_size as i32) / 2;

                text!(
                    &entrance_text,
                    x = text_x,
                    y = text_y,
                    font = "small",
                    color = 0xffffffff // White text
                );
            }
        }

        // Apply highlight effect if needed
        if should_highlight {
            let t = (frame * FLASH_SPEED).sin() * 0.5 + 0.5;
            sprite!(
                "tile_selection",
                x = x,
                y = y,
                w = tile_size,
                h = tile_size,
                opacity = t as f32
            );
        }

        // Draw X marker if tile is selected for swapping
        if is_swap_selected {
            let x_color = 0xff0000ff; // Red color for X
            let x_width = 8;
            let x_offset = ((tile_size as i32) - x_width) / 2;

            // Draw diagonal lines to form an X
            // Line from top-left to bottom-right
            rect!(
                x = x + x_offset,
                y = y + x_offset,
                w = x_width,
                h = x_width,
                color = x_color,
                rotation = 45
            );

            // Line from top-right to bottom-left
            rect!(
                x = x + x_offset,
                y = y + x_offset,
                w = x_width,
                h = x_width,
                color = x_color,
                rotation = -45
            );
        }
    }

    /// Returns a string representation of the tile's entrances (URDL format)
    fn get_entrance_text(&self) -> String {
        let mut text = String::new();

        if self.entrances.contains(&Direction::Up) {
            text.push('U');
        }
        if self.entrances.contains(&Direction::Right) {
            text.push('R');
        }
        if self.entrances.contains(&Direction::Down) {
            text.push('D');
        }
        if self.entrances.contains(&Direction::Left) {
            text.push('L');
        }

        text
    }

    /// Determines which wall sprite to use and what rotation offset to apply
    /// based on the tile's current entrances
    fn get_wall_sprite_and_rotation(&self) -> (&'static str, f32) {
        // Sort entrances to ensure consistent sprite selection
        let mut sorted_entrances = self.entrances.clone();
        sorted_entrances.sort_by(|a, b| {
            use Direction::*;
            match (a, b) {
                (Up, _) => std::cmp::Ordering::Less,
                (Down, Up) => std::cmp::Ordering::Greater,
                (Down, _) => std::cmp::Ordering::Less,
                (Left, Up | Down) => std::cmp::Ordering::Greater,
                (Left, _) => std::cmp::Ordering::Less,
                (Right, _) => std::cmp::Ordering::Greater,
            }
        });

        // Count entrances
        let entrance_count = sorted_entrances.len();

        match entrance_count {
            0 => ("wall_0", 0.0), // No entrances
            1 => {
                // Single entrance - determine sprite and rotation
                match sorted_entrances[0] {
                    Direction::Up => ("wall_1", 0.0), // Top entrance
                    Direction::Down => ("wall_1", 180.0), // Bottom entrance (rotate 180°)
                    Direction::Left => ("wall_1", 270.0), // Left entrance (rotate 270°)
                    Direction::Right => ("wall_1", 90.0), // Right entrance (rotate 90°)
                }
            }
            2 => {
                // Two entrances - check if they're opposite (I-shape) or adjacent (L-shape)
                let (first, second) = (sorted_entrances[0], sorted_entrances[1]);
                match (first, second) {
                    (Direction::Up, Direction::Down) => ("wall_2I", 0.0), // I-shape vertical
                    (Direction::Left, Direction::Right) => ("wall_2I", 90.0), // I-shape horizontal
                    (Direction::Up, Direction::Right) => ("wall_2L", 0.0), // L-shape top-right (default)
                    (Direction::Up, Direction::Left) => ("wall_2L", 270.0), // L-shape top-left (rotate 270°)
                    (Direction::Down, Direction::Right) => ("wall_2L", 90.0), // L-shape bottom-right (rotate 90°)
                    (Direction::Down, Direction::Left) => ("wall_2L", 180.0), // L-shape bottom-left (rotate 180°)
                    _ => ("wall_2L", 0.0), // Fallback
                }
            }
            3 => {
                // Three entrances - determine which one is missing
                let all_directions = [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ];
                let missing = all_directions
                    .iter()
                    .find(|d| !sorted_entrances.contains(d))
                    .unwrap();

                match missing {
                    Direction::Up => ("wall_3", 90.0), // Missing top (rotate 90° from default)
                    Direction::Down => ("wall_3", 270.0), // Missing bottom (rotate 270° from default)
                    Direction::Left => ("wall_3", 0.0), // Missing left (default orientation)
                    Direction::Right => ("wall_3", 180.0), // Missing right (rotate 180° from default)
                }
            }
            4 => ("wall_4", 0.0), // All entrances
            _ => ("wall_0", 0.0), // Fallback
        }
    }

    /// Check if a fireball would hit a wall when moving in the given direction
    /// from the current tile, considering the fireball's radius
    pub fn would_fireball_hit_wall(
        &self,
        current_index: usize,
        direction: Direction,
        tiles: &[Tile]
    ) -> bool {
        let (tile_x, tile_y) = Tile::position(current_index);

        // Check if we're at the map edge
        let at_edge = match direction {
            Direction::Up => tile_y == 0,
            Direction::Down => tile_y == 4,
            Direction::Left => tile_x == 0,
            Direction::Right => tile_x == 4,
        };

        if at_edge {
            return true; // Hit map boundary
        }

        // Calculate next tile index
        let next_tile_index = match direction {
            Direction::Up => current_index - 5,
            Direction::Down => current_index + 5,
            Direction::Left => current_index - 1,
            Direction::Right => current_index + 1,
        };

        // Check if next tile exists and is connected
        if next_tile_index >= tiles.len() {
            return true; // Out of bounds
        }

        let next_tile = &tiles[next_tile_index];
        !self.is_connected_in_direction(direction, next_tile)
    }

    /// Check if a fireball has reached the far edge of a tile when moving in a given direction
    /// This is used to determine when to check for wall collisions
    pub fn has_fireball_reached_far_edge(
        current_index: usize,
        direction: Direction,
        new_pos: (f32, f32),
        fireball_radius: f32,
        tile_size: u32,
        offset_x: u32,
        offset_y: u32
    ) -> bool {
        let (tile_x, tile_y) = Tile::position(current_index);

        match direction {
            Direction::Up => {
                let tile_top_edge = (offset_y as f32) + (tile_y as f32) * (tile_size as f32);
                new_pos.1 <= tile_top_edge + fireball_radius
            }
            Direction::Down => {
                let tile_bottom_edge =
                    (offset_y as f32) + ((tile_y + 1) as f32) * (tile_size as f32);
                new_pos.1 >= tile_bottom_edge - fireball_radius
            }
            Direction::Left => {
                let tile_left_edge = (offset_x as f32) + (tile_x as f32) * (tile_size as f32);
                new_pos.0 <= tile_left_edge + fireball_radius
            }
            Direction::Right => {
                let tile_right_edge =
                    (offset_x as f32) + ((tile_x + 1) as f32) * (tile_size as f32);
                new_pos.0 >= tile_right_edge - fireball_radius
            }
        }
    }

    /// Check if a fireball at the given screen position would hit a player
    /// This checks if the fireball's circular area overlaps with any player's position
    pub fn would_fireball_hit_player(
        fireball_pos: (f32, f32),
        fireball_radius: f32,
        player_positions: &[(usize, usize)],
        tile_size: u32,
        offset_x: u32,
        offset_y: u32
    ) -> Option<usize> {
        for (player_index, &player_pos) in player_positions.iter().enumerate() {
            // Convert player tile position to screen position (center of tile)
            let player_screen_x =
                (offset_x as f32) +
                (player_pos.0 as f32) * (tile_size as f32) +
                (tile_size as f32) / 2.0;
            let player_screen_y =
                (offset_y as f32) +
                (player_pos.1 as f32) * (tile_size as f32) +
                (tile_size as f32) / 2.0;

            // Calculate distance between fireball center and player center
            let dx = fireball_pos.0 - player_screen_x;
            let dy = fireball_pos.1 - player_screen_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Check if fireball radius overlaps with player (assuming player takes up most of tile)
            let player_radius = (tile_size as f32) / 3.0; // Player is roughly 2/3 of tile size
            let collision_distance = fireball_radius + player_radius;

            if distance <= collision_distance {
                return Some(player_index);
            }
        }

        None // No player hit
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
