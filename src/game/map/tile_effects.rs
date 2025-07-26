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
            for (i, tile) in tiles.iter_mut().enumerate() {
                if current_tile.can_move_to(current_index, tile, i) {
                    tile.is_highlighted = true;
                }
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
            for i in Tile::get_adjacent_indices(current_index, true, true) {
                tiles[i].is_highlighted = true;
            }
        }
    }
}
