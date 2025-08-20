use crate::game::{
    constants::{ MONSTER_HEALTH, MONSTER_DAMAGE },
    map::{ tile::{ Direction, Tile }, Player },
};
use crate::server::broadcast::broadcast_player_damage_from_monster;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Monster {
    pub position: (usize, usize),
    pub original_position: (usize, usize),
    pub health: u32,
    pub max_health: u32,
    pub direction: Direction,
    pub animation_frame: usize,
    pub animation_timer: f32,
    pub is_moving: bool,
    pub damage: u32,
    pub target_player: Option<usize>, // Index of the player we're targeting
}

impl Monster {
    pub fn new() -> Self {
        // Spawn in center of board (MAP_SIZE = 5, so center is at (2, 2))
        let center_x = 2;
        let center_y = 2;

        Self {
            position: (center_x, center_y),
            original_position: (center_x, center_y),
            health: MONSTER_HEALTH,
            max_health: MONSTER_HEALTH,
            direction: Direction::Down, // Default direction
            animation_frame: 0,
            animation_timer: 0.0,
            is_moving: false,
            damage: MONSTER_DAMAGE,
            target_player: None,
        }
    }

    pub fn move_to(&mut self, to_index: usize) {
        let (nx, ny) = Tile::position(to_index);
        self.position = (nx, ny);
    }

    pub fn update_original_position(&mut self) {
        self.original_position = self.position;
    }

    // Health management methods
    pub fn take_damage(&mut self, amount: u32) {
        if amount >= self.health {
            self.health = 0;
        } else {
            self.health -= amount;
        }
    }

    pub fn heal(&mut self, amount: u32) {
        self.health = self.health.saturating_add(amount).min(self.max_health);
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn set_moving(&mut self, moving: bool) {
        self.is_moving = moving;
        if !moving {
            // Reset to idle frame when stopping
            self.animation_frame = 0;
            self.animation_timer = 0.0;
        }
    }

    pub fn update_animation(&mut self, delta_time: f32) {
        if self.is_moving {
            // Update animation timer only when moving
            self.animation_timer += delta_time;

            // Change animation frame every 0.15 seconds (about 6-7 FPS for smooth walking)
            if self.animation_timer >= 0.15 {
                self.animation_timer = 0.0;
                self.animation_frame = (self.animation_frame + 1) % 4; // Cycle through 4 frames
            }

            // Auto-stop moving after 0.6 seconds (4 frames * 0.15 seconds) for network movement
            if self.animation_timer >= 0.6 {
                self.set_moving(false);
            }
        } else {
            // Idle animation - cycle between frames 0 and 2
            self.animation_timer += delta_time;

            // Change idle frame every 0.8 seconds (slower, more relaxed)
            if self.animation_timer >= 0.8 {
                self.animation_timer = 0.0;
                // Toggle between frame 0 and frame 2
                if self.animation_frame == 0 {
                    self.animation_frame = 2;
                } else {
                    self.animation_frame = 0;
                }
            }
        }
    }

    /// Calculate and set direction based on movement from one position to another
    pub fn draw(
        &self,
        tile_size: u32,
        offset_x: u32,
        offset_y: u32,
        animated_pos: Option<(f32, f32)>
    ) {
        let (center_x, center_y) = if let Some((ax, ay)) = animated_pos {
            // Use animated position (already center coordinates)
            (ax as u32, ay as u32)
        } else {
            // Use regular tile-based position
            let (gx, gy) = self.position;
            let center_x = offset_x + (gx as u32) * tile_size + tile_size / 2;
            let center_y = offset_y + (gy as u32) * tile_size + tile_size / 2;
            (center_x, center_y)
        };

        // Draw the monster sprite using the spritesheet
        // Select the row based on direction
        let sprite_y = match self.direction {
            Direction::Down => 0, // Top row - moving down
            Direction::Right => 1, // Second row - moving right
            Direction::Up => 2, // Third row - moving up
            Direction::Left => 3, // Bottom row - moving left
        };
        let sprite_x = self.animation_frame; // Use animation frame to cycle through frames

        // Calculate texture coordinates for the 4x4 spritesheet
        let frame_width = 36;
        let frame_height = 36;

        // Calculate the offset within the spritesheet for the specific frame
        let tx = -((sprite_x as i32) * frame_width) as i32;
        let ty = -((sprite_y as i32) * frame_height) as i32;

        // Draw the monster sprite at the center position
        sprite!(
            "monster",
            x = center_x - 18,
            y = center_y - 14,
            w = 36,
            h = 36,
            tx = tx,
            ty = ty,
            cover = false
        );

        self.draw_hearts(center_x, center_y, tile_size / 6);
    }

    fn draw_hearts(&self, center_x: u32, center_y: u32, radius: u32) {
        let heart_size = 12;
        let heart_spacing = 2;
        let hearts_per_row = 3;

        // Calculate how many hearts go in each row
        let first_row_hearts = self.max_health.min(hearts_per_row);
        let second_row_hearts = if self.max_health > hearts_per_row {
            self.max_health - hearts_per_row
        } else {
            0
        };

        // Calculate total width for each row to center them
        let first_row_width = (heart_size + heart_spacing) * first_row_hearts - heart_spacing;
        let second_row_width = if second_row_hearts > 0 {
            (heart_size + heart_spacing) * second_row_hearts - heart_spacing
        } else {
            0
        };

        let first_row_start_x = center_x - first_row_width / 2;
        let second_row_start_x = center_x - second_row_width / 2;

        // Position above the monster circle
        let first_row_y = center_y - radius - heart_size - 1; // Top row
        let second_row_y = center_y - radius - heart_size - heart_size; // Bottom row (if needed)

        // Draw first row (up to 3 hearts)
        for i in 0..first_row_hearts {
            let heart_x = first_row_start_x + (i as u32) * (heart_size + heart_spacing);
            let sprite_name = if i < self.health { "heart_monster" } else { "heart_empty" };

            sprite!(
                sprite_name,
                x = heart_x as i32,
                y = first_row_y as i32,
                w = heart_size,
                h = heart_size,
                cover = true
            );
        }

        // Draw second row (remaining hearts, if any)
        if second_row_hearts > 0 {
            for i in 0..second_row_hearts {
                let heart_x = second_row_start_x + (i as u32) * (heart_size + heart_spacing);
                let heart_index = i + hearts_per_row;
                let sprite_name = if heart_index < self.health {
                    "heart_monster"
                } else {
                    "heart_empty"
                };

                sprite!(
                    sprite_name,
                    x = heart_x as i32,
                    y = second_row_y as i32,
                    w = heart_size,
                    h = heart_size,
                    cover = true
                );
            }
        }
    }

    pub fn take_turn(&mut self, players: &mut [Player], tiles: &[Tile]) {
        if !self.is_alive() {
            return;
        }

        // First, find the nearest player and get their info without borrowing mutably
        let nearest_player_info = self.find_nearest_player_info(players, tiles);
        log!("nearest_player_info: {:?}", nearest_player_info);

        if let Some((player_index, player_pos)) = nearest_player_info {
            // Set this player as our target
            self.target_player = Some(player_index);
            log!("self position: {:?}, player position: {:?}", self.position, player_pos);

            // Check if we're already on the same tile as the player and can deal damage
            if self.position == player_pos {
                log!("Monster is on player tile, dealing damage!");
                if let Some(player) = players.get_mut(player_index) {
                    player.take_damage(self.damage);
                    broadcast_player_damage_from_monster(&player.id.to_string(), self.damage);
                    return; // Don't move if we're already on the player
                }
            }

            // Check if we're adjacent to the player (within 1 tile) and connected
            let dx = (self.position.0 as i32) - (player_pos.0 as i32);
            let dy = (self.position.1 as i32) - (player_pos.1 as i32);
            let is_adjacent = dx.abs() <= 1 && dy.abs() <= 1 && dx.abs() + dy.abs() <= 1;

            if is_adjacent && (dx != 0 || dy != 0) {
                // Skip if same position (already handled above)
                // Only check tile connections if we're actually adjacent
                let monster_tile_index = Tile::index(self.position.0, self.position.1);
                let player_tile_index = Tile::index(player_pos.0, player_pos.1);

                if
                    let (Some(monster_tile), Some(player_tile)) = (
                        tiles.get(monster_tile_index),
                        tiles.get(player_tile_index),
                    )
                {
                    // Determine the direction from monster to player
                    let direction_to_player = if dx > 0 {
                        Direction::Left
                    } else if dx < 0 {
                        Direction::Right
                    } else if dy > 0 {
                        Direction::Up
                    } else {
                        Direction::Down
                    };

                    // Check if the tiles are connected in the direction to the player
                    if
                        monster_tile.is_connected_in_direction(
                            direction_to_player,
                            player_tile,
                            self.position,
                            player_pos
                        )
                    {
                        log!("Monster is on tile connected to player tile, dealing damage!");
                        if let Some(player) = players.get_mut(player_index) {
                            player.take_damage(self.damage);
                            broadcast_player_damage_from_monster(
                                &player.id.to_string(),
                                self.damage
                            );
                            return; // Don't move if we're connected to the player
                        }
                    }
                }
            }

            if let Some(direction) = self.calculate_direction_towards(player_pos, tiles) {
                // Only move if the tiles are connected
                if let Some(new_pos) = self.move_in_direction(direction, tiles) {
                    // Check if new position is valid and not occupied
                    if !self.is_position_occupied(players, new_pos) {
                        let old_pos = self.position;
                        self.position = new_pos;
                        log!("Monster moved from {:?} to {:?}", old_pos, new_pos);

                        // Check if the new position is the same as the player's position
                        if self.position == player_pos {
                            log!("Monster moved onto player tile, dealing damage!");
                            if let Some(player) = players.get_mut(player_index) {
                                player.take_damage(self.damage);
                                broadcast_player_damage_from_monster(
                                    &player.id.to_string(),
                                    self.damage
                                );
                            }
                        }
                    }
                }
            }
        } else {
            // No players found, clear target
            self.target_player = None;

            // If no accessible players, move randomly to an adjacent connected tile
            if let Some(random_direction) = self.get_random_available_direction(tiles) {
                if let Some(new_pos) = self.move_in_direction(random_direction, tiles) {
                    // Check if new position is valid and not occupied
                    if !self.is_position_occupied(players, new_pos) {
                        let old_pos = self.position;
                        self.position = new_pos;
                        log!(
                            "Monster randomly moved from {:?} to {:?} (no accessible players)",
                            old_pos,
                            new_pos
                        );
                    }
                }
            }
        }
    }

    /// Get the current target player index
    pub fn get_target_player(&self) -> Option<usize> {
        self.target_player
    }

    /// Set a specific player as target
    pub fn set_target_player(&mut self, player_index: Option<usize>) {
        self.target_player = player_index;
    }

    /// Clear the current target
    pub fn clear_target(&mut self) {
        self.target_player = None;
    }

    /// Find the nearest player to the monster (returns index and position, not reference)
    fn find_nearest_player_info(
        &self,
        players: &[Player],
        tiles: &[Tile]
    ) -> Option<(usize, (usize, usize))> {
        if players.is_empty() {
            return None;
        }

        // If we already have a target, check if they're still alive and accessible
        if let Some(current_target) = self.target_player {
            if let Some(player) = players.get(current_target) {
                if player.is_alive() {
                    // Check if there's still a valid path to our current target
                    let start_index = Tile::index(self.position.0, self.position.1);
                    let target_index = Tile::index(player.position.0, player.position.1);

                    if Tile::find_walkable_path(start_index, target_index, tiles).is_some() {
                        // Keep our current target if still accessible
                        return Some((current_target, player.position));
                    }
                }
            }
        }

        // Find all players and their distances, but only if they're accessible
        let mut player_distances: Vec<(usize, (usize, usize), u32)> = players
            .iter()
            .enumerate()
            .filter_map(|(index, player)| {
                if player.is_alive() {
                    // Check if there's a valid path to this player
                    let start_index = Tile::index(self.position.0, self.position.1);
                    let target_index = Tile::index(player.position.0, player.position.1);

                    if Tile::find_walkable_path(start_index, target_index, tiles).is_some() {
                        let dx = (player.position.0 as i32) - (self.position.0 as i32);
                        let dy = (player.position.1 as i32) - (self.position.1 as i32);
                        let distance = (dx * dx + dy * dy) as u32; // Manhattan distance squared, cast to u32
                        Some((index, player.position, distance))
                    } else {
                        None // Player is not accessible
                    }
                } else {
                    None
                }
            })
            .collect();

        if player_distances.is_empty() {
            return None;
        }

        // Sort by distance (closest first)
        player_distances.sort_by_key(|&(_, _, distance)| distance);

        let closest_distance = player_distances[0].2;

        // Find all players at the closest distance
        let closest_players: Vec<(usize, (usize, usize))> = player_distances
            .into_iter()
            .filter(|&(_, _, distance)| distance == closest_distance)
            .map(|(index, position, _)| (index, position))
            .collect();

        if closest_players.len() == 1 {
            // Only one closest accessible player
            Some(closest_players[0])
        } else {
            // Multiple accessible players at same distance
            if let Some(current_target) = self.target_player {
                // If we already have a target, try to keep them if they're among the closest accessible
                if
                    let Some(&(index, position)) = closest_players
                        .iter()
                        .find(|&&(idx, _)| idx == current_target)
                {
                    Some((index, position))
                } else {
                    // Our current target is no longer among the closest accessible, pick from closest using simple hash
                    let random_index =
                        (current_target + self.position.0 + self.position.1) %
                        closest_players.len();
                    Some(closest_players[random_index])
                }
            } else {
                // No current target, pick from closest accessible players using position-based hash
                let random_index = (self.position.0 + self.position.1) % closest_players.len();
                Some(closest_players[random_index])
            }
        }
    }

    /// Calculate direction towards target position using pathfinding
    fn calculate_direction_towards(
        &self,
        target: (usize, usize),
        tiles: &[Tile]
    ) -> Option<Direction> {
        let start_index = Tile::index(self.position.0, self.position.1);
        let target_index = Tile::index(target.0, target.1);

        if let Some(path) = Tile::find_walkable_path(start_index, target_index, tiles) {
            if path.len() >= 2 {
                let next_index = path[1];
                let (next_x, next_y) = Tile::position(next_index);
                let (current_x, current_y) = self.position;

                // Determine direction from current to next position
                if next_x > current_x {
                    Some(Direction::Right)
                } else if next_x < current_x {
                    Some(Direction::Left)
                } else if next_y > current_y {
                    Some(Direction::Down)
                } else if next_y < current_y {
                    Some(Direction::Up)
                } else {
                    None // Same position
                }
            } else {
                None // No path or already at target
            }
        } else {
            None // No path found
        }
    }

    /// Move in the given direction (only if tiles are connected)
    fn move_in_direction(&self, direction: Direction, tiles: &[Tile]) -> Option<(usize, usize)> {
        let current_tile_index = Tile::index(self.position.0, self.position.1);
        let current_tile = &tiles[current_tile_index];

        let new_pos = match direction {
            Direction::Up => (self.position.0, self.position.1.saturating_sub(1)),
            Direction::Down => (self.position.0, (self.position.1 + 1).min(4)),
            Direction::Left => (self.position.0.saturating_sub(1), self.position.1),
            Direction::Right => ((self.position.0 + 1).min(4), self.position.1),
        };

        // Check if new position is within bounds
        if !self.is_valid_position(new_pos) {
            return None;
        }

        let target_tile_index = Tile::index(new_pos.0, new_pos.1);
        let target_tile = &tiles[target_tile_index];

        // Check if the tiles are connected in the direction we want to move
        if
            !current_tile.is_connected_in_direction(
                direction.into(),
                target_tile,
                self.position,
                new_pos
            )
        {
            return None;
        }

        Some(new_pos)
    }

    /// Check if position is within bounds
    fn is_valid_position(&self, pos: (usize, usize)) -> bool {
        pos.0 < 5 && pos.1 < 5
    }

    /// Check if position is occupied by a player
    fn is_position_occupied(&self, players: &[Player], pos: (usize, usize)) -> bool {
        players.iter().any(|player| player.position == pos)
    }

    /// Get a random available direction from the current position
    fn get_random_available_direction(&self, tiles: &[Tile]) -> Option<Direction> {
        let current_tile_index = Tile::index(self.position.0, self.position.1);
        let current_tile = &tiles[current_tile_index];

        let mut available_directions = Vec::new();

        // Check Up
        if self.position.1 > 0 {
            let up_pos = (self.position.0, self.position.1 - 1);
            if
                current_tile.is_connected_in_direction(
                    Direction::Up.into(),
                    &tiles[Tile::index(up_pos.0, up_pos.1)],
                    self.position,
                    up_pos
                )
            {
                available_directions.push(Direction::Up);
            }
        }
        // Check Down
        if self.position.1 < 4 {
            let down_pos = (self.position.0, self.position.1 + 1);
            if
                current_tile.is_connected_in_direction(
                    Direction::Down.into(),
                    &tiles[Tile::index(down_pos.0, down_pos.1)],
                    self.position,
                    down_pos
                )
            {
                available_directions.push(Direction::Down);
            }
        }
        // Check Left
        if self.position.0 > 0 {
            let left_pos = (self.position.0 - 1, self.position.1);
            if
                current_tile.is_connected_in_direction(
                    Direction::Left.into(),
                    &tiles[Tile::index(left_pos.0, left_pos.1)],
                    self.position,
                    left_pos
                )
            {
                available_directions.push(Direction::Left);
            }
        }
        // Check Right
        if self.position.0 < 4 {
            let right_pos = (self.position.0 + 1, self.position.1);
            if
                current_tile.is_connected_in_direction(
                    Direction::Right.into(),
                    &tiles[Tile::index(right_pos.0, right_pos.1)],
                    self.position,
                    right_pos
                )
            {
                available_directions.push(Direction::Right);
            }
        }

        if available_directions.is_empty() {
            None
        } else {
            // Use turbo's built-in random function
            let random_index = (random::u32() as usize) % available_directions.len();
            Some(available_directions[random_index].clone())
        }
    }
}
