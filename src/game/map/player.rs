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
            Card::get_unique_cards().into_iter().take(hand_size).collect()
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

        let diameter = tile_size / 3;
        let radius = diameter / 2;
        let color = match self.id {
            PlayerId::Player1 => PLAYER_1_COLOR,
            PlayerId::Player2 => PLAYER_2_COLOR,
        };

        // Adjust coordinates to account for circle being drawn from top-left corner
        let circle_x = center_x - radius;
        let circle_y = center_y - radius;

        circ!(d = diameter as u32, x = circle_x, y = circle_y, color = color);

        // Draw hearts instead of health bar
        let heart_size = 12;
        let heart_spacing = 0;
        let total_hearts_width = (heart_size + heart_spacing) * 3 - heart_spacing; // 3 hearts with spacing
        let hearts_start_x = center_x - total_hearts_width / 2;
        // position above the player circle
        let hearts_y = center_y - radius - heart_size;

        // Draw all 3 heart positions (empty hearts for missing health)
        for i in 0..3 {
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
}
