use crate::game::tile::Tile;
use crate::GameState;

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
