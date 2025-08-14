use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };
use crate::game::map::tile::Direction;

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Fireball {
    pub id: u32,
    pub damage: u32,
    pub position: (usize, usize),
    pub direction: crate::game::map::tile::Direction,
    pub is_active: bool,
    pub shooter_id: crate::game::map::player::PlayerId,
}

impl Fireball {
    pub fn new(
        damage: u32,
        position: (usize, usize),
        direction: crate::game::map::tile::Direction,
        shooter_id: crate::game::map::player::PlayerId
    ) -> Self {
        Self {
            id: random::u32(),
            damage,
            position,
            direction,
            is_active: true,
            shooter_id,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.is_active
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn get_damage(&self) -> u32 {
        self.damage
    }

    pub fn get_position(&self) -> (usize, usize) {
        self.position
    }

    pub fn get_direction(&self) -> crate::game::map::tile::Direction {
        self.direction
    }

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

        let sprite_size = tile_size / 2; // Make fireball sprite smaller than player
        let sprite_x = center_x - sprite_size / 2;
        let sprite_y = center_y - sprite_size / 2;

        // Draw the fireball sprite with rotation based on direction
        // The sprite is designed for traveling down, so we rotate accordingly
        let rotation = match self.direction {
            Direction::Up => 180.0, // Rotate 180° for up
            Direction::Down => 0.0, // No rotation for down (default)
            Direction::Left => 90.0, // Rotate 90° for left
            Direction::Right => 270.0, // Rotate 270° for right
        };

        sprite!(
            "fireball",
            x = sprite_x as i32,
            y = sprite_y as i32,
            w = sprite_size,
            h = sprite_size,
            rotation = rotation
        );
    }
}
