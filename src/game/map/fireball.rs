use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Fireball {
    pub id: u32,
    pub damage: u32,
    pub position: (usize, usize),
    pub direction: crate::game::map::tile::Direction,
    pub is_active: bool,
}

impl Fireball {
    pub fn new(
        damage: u32,
        position: (usize, usize),
        direction: crate::game::map::tile::Direction
    ) -> Self {
        Self {
            id: random::u32(),
            damage,
            position,
            direction,
            is_active: true,
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

        let diameter = tile_size / 4; // Make fireball smaller than player
        let radius = diameter / 2;
        let fireball_color = 0xff4400ff; // Orange-red fire color

        // Adjust coordinates to account for circle being drawn from top-left corner
        let circle_x = center_x - radius;
        let circle_y = center_y - radius;

        circ!(d = diameter as u32, x = circle_x, y = circle_y, color = fireball_color);
    }
}
