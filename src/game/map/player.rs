use crate::game::constants::{ PLAYER_1_COLOR, PLAYER_2_COLOR, PLAYER_HEALTH };
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
    pub health: u32,
}

impl Player {
    pub fn new(id: PlayerId, x: usize, y: usize, hand_size: usize, is_dummy: bool) -> Self {
        let hand = Self::new_hand(hand_size, is_dummy);

        Self {
            id,
            position: (x, y),
            original_position: (x, y),
            hand,
            health: PLAYER_HEALTH,
        }
    }

    // No duplicates in hand
    pub fn new_hand(hand_size: usize, is_dummy: bool) -> Vec<Card> {
        if is_dummy {
            (0..hand_size).map(|_| Card::dummy_card()).collect()
        } else {
            let mut all_cards = Card::get_unique_cards();

            for i in (1..all_cards.len()).rev() {
                let j = (random::u32() as usize) % (i + 1);
                all_cards.swap(i, j);
            }

            all_cards.into_iter().take(hand_size).collect()
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

    // Health management methods
    pub fn take_damage(&mut self, amount: u32) {
        if amount >= self.health {
            self.health = 0;
        } else {
            self.health -= amount;
        }
    }

    pub fn heal(&mut self, amount: u32) {
        self.health = self.health.saturating_add(amount);
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0
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

        // Draw health bar below the player
        let health_bar_width = tile_size / 3;
        let health_bar_height = 3;
        let health_bar_x = center_x - health_bar_width / 2;
        let health_bar_y = center_y + radius + 4; // Position below the player circle

        // Draw health bar background (dark gray)
        rect!(
            x = health_bar_x,
            y = health_bar_y,
            w = health_bar_width,
            h = health_bar_height,
            color = 0x444444ff,
            border_radius = 1
        );

        // Calculate health percentage and draw filled health bar
        let health_percentage = (self.health as f32) / (PLAYER_HEALTH as f32);
        let filled_width = ((health_bar_width as f32) * health_percentage) as u32;

        if filled_width > 0 {
            let health_color = if health_percentage > 0.5 {
                0x00ff00ff // Green when health > 50%
            } else if health_percentage > 0.25 {
                0xffff00ff // Yellow when health 25-50%
            } else {
                0xff0000ff // Red when health < 25%
            };

            rect!(
                x = health_bar_x,
                y = health_bar_y,
                w = filled_width,
                h = health_bar_height,
                color = health_color,
                border_radius = 1
            );
        }
    }
}
