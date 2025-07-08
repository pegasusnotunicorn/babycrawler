mod board;
mod hand;
mod input;
mod player;
mod tile;
mod card;
mod card_effect;
mod constants;
mod ui;
mod turn;
mod util;

use crate::{
    board::draw_board,
    card::Card,
    constants::*,
    hand::draw_hand,
    input::handle_input,
    player::{ Player, PlayerId },
    tile::{ clear_highlights, Direction, Tile },
    ui::draw_turn_label,
};

use turbo::{ bounds, random::rand, os::server::fs, * };

#[turbo::game]
pub struct GameState {
    pub frame: usize,
    pub tiles: Vec<Tile>,
    pub players: Vec<Player>,
    pub current_turn: usize,
    pub selected_cards: Vec<Card>,
}

impl GameState {
    pub fn new() -> Self {
        let mut tiles = vec![];
        for _ in 0..MAP_SIZE * MAP_SIZE {
            let mut entrances = vec![];
            for dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                if rand() % 2 == 0 {
                    entrances.push(*dir);
                }
            }
            tiles.push(Tile::new(entrances));
        }

        let mut players = vec![
            Player::new(PlayerId::Player1, 0, 0),
            Player::new(PlayerId::Player2, MAP_SIZE - 1, MAP_SIZE - 1)
        ];
        for player in &mut players {
            for _ in 0..HAND_SIZE {
                player.hand.push(Card::random());
            }
        }

        Self {
            frame: 0,
            tiles,
            players,
            current_turn: 0,
            selected_cards: vec![],
        }
    }

    pub fn update(&mut self) {
        clear(GAME_BG_COLOR);
        self.frame += 1;

        let pointer = mouse::screen();
        let pointer_xy = (pointer.x, pointer.y);
        let canvas_width = bounds::canvas().w();
        let tile_size = canvas_width / (MAP_SIZE as u32);
        let offset_x = canvas_width / 2 - (tile_size * (MAP_SIZE as u32)) / 2;
        let offset_y = 0;

        handle_input(self, &pointer, pointer_xy, tile_size, offset_x, offset_y);

        // draw shit
        draw_board(self, self.frame as f64, tile_size, offset_x, offset_y);
        draw_hand(
            &self.players[self.current_turn].hand,
            &self.selected_cards.clone(),
            tile_size,
            self.frame as f64,
            |card| {
                Card::toggle_in(&mut self.selected_cards, card);
                clear_highlights(&mut self.tiles);
                if self.selected_cards.len() == 1 {
                    let card = &self.selected_cards[0];
                    let player = &self.players[self.current_turn];
                    card.effect.highlight_tiles(player.position, &mut self.tiles);
                }
            }
        );
        draw_turn_label(self.current_turn, tile_size);

        fs::write("state", self).ok();
    }
}
