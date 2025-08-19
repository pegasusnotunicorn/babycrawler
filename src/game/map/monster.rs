use crate::game::{ constants::{ MONSTER_HEALTH, MONSTER_DAMAGE }, map::{ tile::Tile, Player } };
use crate::server::broadcast::broadcast_player_damage_from_monster;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum Direction {
    Down,
    Right,
    Up,
    Left,
}

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
        let heart_spacing = 0;
        let total_hearts_width = (heart_size + heart_spacing) * self.max_health - heart_spacing; // 5 hearts with spacing
        let hearts_start_x = center_x - total_hearts_width / 2;
        // position above the monster circle
        let hearts_y = center_y - radius - heart_size;

        // Draw all 5 heart positions (empty hearts for missing health)
        for i in 0..self.max_health {
            let heart_x = hearts_start_x + (i as u32) * (heart_size + heart_spacing);

            if i < self.health {
                // Draw filled heart sprite
                sprite!(
                    "heart",
                    x = heart_x as i32,
                    y = hearts_y as i32,
                    w = heart_size,
                    h = heart_size,
                    cover = true
                );
            } else {
                // Draw empty heart (gray outline)
                sprite!(
                    "heart_empty",
                    x = heart_x as i32,
                    y = hearts_y as i32,
                    w = heart_size,
                    h = heart_size,
                    cover = true
                );
            }
        }
    }

    pub fn take_turn(&mut self, players: &mut [Player]) {
        if !self.is_alive() {
            return;
        }

        // First, find the nearest player and get their info without borrowing mutably
        let nearest_player_info = self.find_nearest_player_info(players);

        if let Some((player_index, player_pos)) = nearest_player_info {
            let direction = self.calculate_direction_towards(player_pos);
            let new_pos = self.move_in_direction(direction);

            // Check if new position is valid and not occupied
            if self.is_valid_position(new_pos) && !self.is_position_occupied(players, new_pos) {
                self.position = new_pos;
            }

            // if the new position is the same as the player's position, deal damage to the player
            if new_pos == player_pos {
                if let Some(player) = players.get_mut(player_index) {
                    player.take_damage(self.damage);
                    broadcast_player_damage_from_monster(&player.id.to_string(), self.damage);
                }
            }
        }
    }

    /// Find the nearest player to the monster (returns index and position, not reference)
    fn find_nearest_player_info(&self, players: &[Player]) -> Option<(usize, (usize, usize))> {
        players
            .iter()
            .enumerate()
            .min_by_key(|(_, player)| {
                let dx = (player.position.0 as i32) - (self.position.0 as i32);
                let dy = (player.position.1 as i32) - (self.position.1 as i32);
                dx * dx + dy * dy // Manhattan distance squared
            })
            .map(|(index, player)| (index, player.position))
    }

    /// Calculate direction towards target position
    fn calculate_direction_towards(&self, target: (usize, usize)) -> Direction {
        let dx = (target.0 as i32) - (self.position.0 as i32);
        let dy = (target.1 as i32) - (self.position.1 as i32);

        if dx.abs() > dy.abs() {
            if dx > 0 { Direction::Right } else { Direction::Left }
        } else {
            if dy > 0 { Direction::Down } else { Direction::Up }
        }
    }

    /// Move in the given direction
    fn move_in_direction(&self, direction: Direction) -> (usize, usize) {
        match direction {
            Direction::Up => (self.position.0, self.position.1.saturating_sub(1)),
            Direction::Down => (self.position.0, (self.position.1 + 1).min(4)),
            Direction::Left => (self.position.0.saturating_sub(1), self.position.1),
            Direction::Right => ((self.position.0 + 1).min(4), self.position.1),
        }
    }

    /// Check if position is within bounds
    fn is_valid_position(&self, pos: (usize, usize)) -> bool {
        pos.0 < 5 && pos.1 < 5
    }

    /// Check if position is occupied by a player
    fn is_position_occupied(&self, players: &[Player], pos: (usize, usize)) -> bool {
        players.iter().any(|player| player.position == pos)
    }
}
