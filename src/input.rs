use crate::{ GameState, tile::Tile, card::CardEffect, constants::* };
use crate::turn::commands::NextTurn;
use crate::util::point_in_bounds;
use turbo::*;

pub fn handle_input(
    state: &mut GameState,
    pointer: &mouse::ScreenMouse,
    pointer_xy: (i32, i32),
    tile_size: u32,
    offset_x: u32
) {
    let Some(card) = &state.selected_card else {
        return;
    };

    let player = &mut state.players[state.current_turn];
    let (px, py) = player.position;
    let current_index = py * MAP_SIZE + px;

    let (current_tile, rest): (&Tile, &mut [Tile]) = if current_index < state.tiles.len() {
        let (left, right) = state.tiles.split_at_mut(current_index);
        (&right[0], left)
    } else {
        return;
    };

    for tile in rest.iter_mut() {
        if !current_tile.can_move_to(tile) {
            continue;
        }

        let tx = offset_x + (tile.grid_x as u32) * tile_size;
        let ty = (tile.grid_y as u32) * tile_size;
        let bounds = Bounds::new(tx, ty, tile_size, tile_size);
        let mx = pointer_xy.0;
        let my = pointer_xy.1;

        if pointer.just_pressed() && point_in_bounds(mx, my, &bounds) {
            match card.effect {
                CardEffect::MoveOneTile => {
                    player.move_to(tile);
                }
                CardEffect::RotateCard => {
                    tile.rotate_clockwise(1);
                }
            }

            state.selected_card = None;
            let _ = NextTurn.exec();
            break;
        }
    }
}
