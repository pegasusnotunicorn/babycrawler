use crate::game::constants::{ PLAYER_1_COLOR, PLAYER_2_COLOR };
use crate::game::map::tile::Tile;
use crate::game::cards::card::Card;
use turbo::{ borsh::{ BorshDeserialize, BorshSerialize }, * };
use serde::{ Serialize, Deserialize };

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum PlayerId {
    Player1,
    Player2,
}

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub position: (usize, usize),
    pub original_position: (usize, usize),
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(id: PlayerId, x: usize, y: usize, hand_size: usize) -> Self {
        let mut hand = Vec::new();
        for _ in 0..hand_size {
            hand.push(Card::random());
        }

        Self {
            id,
            position: (x, y),
            original_position: (x, y),
            hand,
        }
    }

    pub fn move_to(&mut self, to_index: usize) {
        let (nx, ny) = Tile::position(to_index);
        self.position = (nx, ny);
    }

    pub fn draw_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    pub fn update_original_position(&mut self) {
        self.original_position = self.position;
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

        let diameter = tile_size / 3; // Make player smaller to fit better in tile
        let radius = diameter / 2;
        let color = match self.id {
            PlayerId::Player1 => PLAYER_1_COLOR,
            PlayerId::Player2 => PLAYER_2_COLOR,
        };

        // Adjust coordinates to account for circle being drawn from top-left corner
        let circle_x = center_x - radius;
        let circle_y = center_y - radius;

        circ!(d = diameter as u32, x = circle_x, y = circle_y, color = color);
    }
}
