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
            // Set the original location for this tile
            tile.original_location = i;
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
    // Phase 1: Draw all non-animated tiles at their logical grid positions
    for (i, tile) in state.tiles.iter().enumerate() {
        // Skip animated tiles - we'll draw them separately
        let is_animated = state.animated_tiles.iter().any(|anim| anim.tile_index == i);
        if is_animated {
            continue;
        }

        // Draw tiles at their current index positions (they haven't been swapped yet)
        let (tx, ty) = Tile::screen_position(i, tile_size, offset_x, offset_y);
        let is_swap_selected = state.swap_tiles_selected.contains(&i);
        tile.draw(tx as i32, ty as i32, tile_size, tile.is_highlighted, frame, is_swap_selected);
    }

    // Phase 2: Draw all animated tiles on top
    for anim in &state.animated_tiles {
        let tile = &state.tiles[anim.tile_index];
        let is_swap_selected = state.swap_tiles_selected.contains(&anim.tile_index);
        tile.draw(
            anim.pos.0 as i32,
            anim.pos.1 as i32,
            tile_size,
            tile.is_highlighted,
            frame,
            is_swap_selected
        );
    }

    // Phase 3: Draw players on top of everything
    for player in state.players.iter() {
        // Check if this player is being animated
        let animated_pos = if let Some(anim) = &state.animated_player {
            // Find the user_id that maps to this player's PlayerId
            let user_id_for_player = state.user_id_to_player_id
                .iter()
                .find(|(_, player_id)| **player_id == player.id)
                .map(|(user_id, _)| user_id);

            if let Some(user_id) = user_id_for_player {
                if anim.player_id == *user_id { Some(anim.pos) } else { None }
            } else {
                None
            }
        } else {
            None
        };

        player.draw(tile_size, offset_x, offset_y, animated_pos);
    }
}
