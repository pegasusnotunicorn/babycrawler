use crate::game::map::tile::{ Tile, Direction };
use crate::game::constants::MAP_SIZE;
use crate::GameState;
use turbo::random;

pub fn random_tiles(count: usize) -> Vec<Tile> {
    (0..count)
        .map(|i| {
            let (x, y) = Tile::position(i);
            let mut forbidden = vec![];
            if y == 0 {
                forbidden.push(Direction::Up);
            }
            if y == MAP_SIZE - 1 {
                forbidden.push(Direction::Down);
            }
            if x == 0 {
                forbidden.push(Direction::Left);
            }
            if x == MAP_SIZE - 1 {
                forbidden.push(Direction::Right);
            }
            let mut tile = Tile::random(&forbidden);
            // Ensure at least one entrance remains (should be handled by Tile::random)
            if tile.entrances.is_empty() {
                let mut possible = vec![];
                if y > 0 {
                    possible.push(Direction::Up);
                }
                if y < MAP_SIZE - 1 {
                    possible.push(Direction::Down);
                }
                if x > 0 {
                    possible.push(Direction::Left);
                }
                if x < MAP_SIZE - 1 {
                    possible.push(Direction::Right);
                }
                if !possible.is_empty() {
                    let idx = (random::u32() as usize) % possible.len();
                    tile.entrances.push(possible[idx]);
                }
            }
            tile
        })
        .collect()
}

pub fn draw_board(state: &GameState, frame: f64, tile_size: u32, offset_x: u32, offset_y: u32) {
    for (i, tile) in state.tiles.iter().enumerate() {
        let (tx, ty) = Tile::screen_position(i, tile_size, offset_x, offset_y);
        tile.draw(tx as i32, ty as i32, tile_size, tile.is_highlighted, frame);
    }

    // Draw players
    for player in state.players.iter() {
        player.draw(tile_size, offset_x, offset_y);
    }
}
