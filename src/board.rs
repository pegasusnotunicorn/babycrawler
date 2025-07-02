use crate::{ GameState, constants::* };
use crate::util::point_in_bounds;
use turbo::*;

pub fn draw_board(
    state: &mut GameState,
    frame: f64,
    pointer: mouse::ScreenMouse,
    pointer_xy: (i32, i32),
    tile_size: u32,
    offset_x: u32
) {
    let tile_positions: Vec<(usize, u32, u32)> = state.tiles
        .iter()
        .map(|tile| {
            let tx = offset_x + (tile.grid_x as u32) * tile_size;
            let ty = (tile.grid_y as u32) * tile_size;
            (tile.grid_y * MAP_SIZE + tile.grid_x, tx, ty)
        })
        .collect();

    let selected_card = state.selected_card.clone(); // Clone early to avoid borrow conflicts

    let mut highlight_tiles: Vec<usize> = vec![];
    if let Some(card) = &selected_card {
        let player = &state.players[state.current_turn];
        highlight_tiles = card.effect.highlight_tiles(player.position, &state.tiles);
    }

    let mut hovered_index: Option<usize> = None;
    let mut effect_tile_index: Option<usize> = None;

    for (_i, (index, tx, ty)) in tile_positions.iter().enumerate() {
        let tile = &state.tiles[*index];
        let bounds = Bounds::new(*tx, *ty, tile_size, tile_size);
        let mx = pointer_xy.0;
        let my = pointer_xy.1;
        let is_hovered = point_in_bounds(mx, my, &bounds);
        let should_highlight = highlight_tiles.contains(index);

        if is_hovered {
            hovered_index = Some(*index);

            if pointer.just_pressed() && should_highlight {
                effect_tile_index = Some(*index); // defer mutable action
            }

            continue;
        }

        if should_highlight {
            tile.draw_highlighted(*tx as i32, *ty as i32, tile_size, false, frame);
        } else {
            tile.draw_at_absolute(0x444444ff, *tx as i32, *ty as i32, tile_size, false);
        }
    }

    // Perform mutable action safely after loop
    if let (Some(tile_idx), Some(card)) = (effect_tile_index, selected_card) {
        card.effect.apply_effect(state, tile_idx);
    }

    if let Some(index) = hovered_index {
        let (_, tx, ty) = tile_positions[index];
        let tile = &state.tiles[index];
        let should_highlight = highlight_tiles.contains(&index);

        if should_highlight {
            tile.draw_highlighted(tx as i32, ty as i32, tile_size, true, frame);
        } else {
            tile.draw_at_absolute(0x444444ff, tx as i32, ty as i32, tile_size, true);
        }
    }

    // Draw players
    for (_i, player) in state.players.iter().enumerate() {
        player.draw(tile_size, offset_x);
    }
}
