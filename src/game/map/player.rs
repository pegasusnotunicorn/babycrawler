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

    pub fn draw(&self, tile_size: u32, offset_x: u32, offset_y: u32) {
        let (gx, gy) = self.position;
        let diameter = tile_size / 2;
        let radius = diameter / 2;
        let center_x = offset_x + (gx as u32) * tile_size + tile_size / 2 - radius;
        let center_y = offset_y + (gy as u32) * tile_size + tile_size / 2 - radius;
        let color = match self.id {
            PlayerId::Player1 => PLAYER_1_COLOR,
            PlayerId::Player2 => PLAYER_2_COLOR,
        };

        circ!(d = diameter as u32, x = center_x, y = center_y, color = color);
    }
}
