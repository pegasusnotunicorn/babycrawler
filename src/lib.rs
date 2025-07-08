mod board;
mod hand;
mod input;
mod player;
mod tile;
mod card;
mod constants;
mod ui;
mod turn;
mod util;

use crate::{
    board::draw_board,
    hand::draw_hand,
    input::handle_input,
    player::{ Player, PlayerId },
    tile::{ Tile, Direction },
    card::{ Card },
    constants::*,
    ui::draw_turn_label,
};

use turbo::{ bounds, random::rand, os::server::fs, * };

#[turbo::game]
pub struct GameState {
    pub frame: usize,
    pub tiles: Vec<Tile>,
    pub players: Vec<Player>,
    pub current_turn: usize,
    pub selected_card: Option<Card>,
}

impl GameState {
    pub fn new() -> Self {
        let mut tiles = vec![];
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let mut entrances = vec![];
                for dir in &[Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
                    if rand() % 2 == 0 {
                        entrances.push(*dir);
                    }
                }
                tiles.push(Tile::new(x, y, entrances));
            }
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
            selected_card: None,
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

        handle_input(self, &pointer, pointer_xy, tile_size, offset_x);
        draw_board(self, self.frame as f64, pointer, pointer_xy, tile_size, offset_x, offset_y);

        let selected = self.selected_card.clone();

        draw_hand(
            &self.players[self.current_turn].hand,
            &selected,
            tile_size,
            self.frame as f64,
            |card| {
                if self.selected_card.as_ref() == Some(card) {
                    self.selected_card = None; // deselect
                } else {
                    self.selected_card = Some(card.clone()); // select
                }
            }
        );

        draw_turn_label(self.current_turn, tile_size);
        fs::write("state", self).ok();
    }
}
