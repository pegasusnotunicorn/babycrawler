use crate::game::cards::card_effect::CardEffect;
use crate::game::map::tile::Tile;
use crate::game::constants::MAP_SIZE;

pub fn highlight_tiles_for_effect(
    effect: &CardEffect,
    player_pos: (usize, usize),
    tiles: &mut [Tile]
) {
    let current_index = player_pos.1 * MAP_SIZE + player_pos.0;
    let current_tile = tiles[current_index].clone();

    match effect {
        CardEffect::Dummy => {}

        CardEffect::MoveOneTile => {
            // Find all reachable tiles through connected entrances
            let reachable_indices = current_tile.find_reachable_tiles(current_index, tiles);
            for &index in &reachable_indices {
                tiles[index].is_highlighted = true;
            }
        }

        CardEffect::RotateCard => {
            // Store current rotation for all tiles
            for tile in tiles.iter_mut() {
                tile.original_rotation = tile.current_rotation;
            }
            for i in Tile::get_adjacent_indices(current_index, true, true) {
                tiles[i].is_highlighted = true;
            }
        }

        CardEffect::SwapCard => {
            // Highlight current tile and all adjacent + diagonal tiles (9 total)
            tiles[current_index].is_highlighted = true; // Current tile
            for i in Tile::get_adjacent_indices(current_index, true, true) {
                tiles[i].is_highlighted = true;
            }
        }

        CardEffect::FireCard => {
            // Highlight all tiles in straight lines from player's position that are connected by entrances
            // Check all four directions: Up, Down, Left, Right
            use crate::game::map::tile::Direction;

            let directions = [Direction::Up, Direction::Down, Direction::Left, Direction::Right];

            for direction in directions {
                let connected_indices = Tile::find_connected_line(
                    current_index,
                    direction,
                    tiles,
                    None
                );
                for &index in &connected_indices {
                    tiles[index].is_highlighted = true;
                }
            }
        }
    }
}
