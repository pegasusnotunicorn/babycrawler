use turbo::*;
use crate::game::cards::card::Card;

use crate::game::animation::{
    start_tile_rotation_animation,
    start_player_movement_animation,
    start_direct_player_movement_animation,
    start_fireball_animation,
    animate_tile_to_index,
};

use crate::GameState;
use crate::game::map::clear_highlights;
use crate::game::map::fireball::Fireball;
use crate::game::map::tile::Tile;
use crate::game::constants::HAND_SIZE;

pub fn receive_connected_users(game_state: &mut GameState, users: Vec<String>) {
    log!("📨 [RECEIVE] Connected users: {:?}", users);

    game_state.in_lobby = users.clone();
    game_state.user_id_to_player_id.clear();

    // Handle player mapping for any number of players
    if !game_state.in_lobby.is_empty() {
        log!("📨 [RECEIVE] Inserting player 1");
        game_state.user_id_to_player_id.insert(
            game_state.in_lobby[0].clone(),
            crate::game::map::PlayerId::Player1
        );

        if game_state.in_lobby.len() >= 2 {
            log!("📨 [RECEIVE] Inserting player 2");
            game_state.user_id_to_player_id.insert(
                game_state.in_lobby[1].clone(),
                crate::game::map::PlayerId::Player2
            );
        }
    }
}

pub fn receive_board_state(
    game_state: &mut GameState,
    tiles: Vec<crate::game::map::Tile>,
    players: Vec<crate::game::map::Player>,
    monster: Option<crate::game::map::Monster>,
    current_turn: Option<crate::server::CurrentTurn>
) {
    log!("📨 [RECEIVE] Board state, current_turn: {:?}", current_turn);

    // Check if the turn has changed to a different player
    let turn_changed = if
        let (Some(old_turn), Some(new_turn)) = (&game_state.current_turn, &current_turn)
    {
        old_turn.player_id != new_turn.player_id
    } else {
        // If either is None, consider it a change (new game or first turn)
        game_state.current_turn.is_some() != current_turn.is_some()
    };

    // If turn changed to a different player, start a new turn
    if turn_changed {
        log!("📨 [RECEIVE] Turn changed, starting new turn");
        game_state.reset_turn();
    }

    // Update game state
    game_state.tiles = tiles;
    game_state.players = players;
    game_state.monster = monster;
    game_state.current_turn = current_turn.clone();
}

pub fn receive_card_cancelled(game_state: &mut GameState, card: &Card, player_id: &str) {
    log!(
        "📨 [RECEIVE] Card cancelled by {}: {:?}, hand_index: {:?}",
        player_id,
        card.name,
        card.hand_index
    );

    if game_state.user == player_id {
        game_state.selected_card = None;
        clear_highlights(&mut game_state.tiles);
    }
}

pub fn receive_tile_rotation(
    game_state: &mut GameState,
    tile_index: &usize,
    tile: &Tile,
    player_id: &str
) {
    log!(
        "📨 [RECEIVE] Tile rotation: index={}, rotation={}, player={}",
        tile_index,
        tile.current_rotation,
        player_id
    );
    let is_local_player = game_state.user == player_id;
    if !is_local_player {
        start_tile_rotation_animation(game_state, *tile_index, 0.25);
    }
}

pub fn receive_player_moved(
    game_state: &mut GameState,
    player_id: &str,
    new_position: &(usize, usize),
    is_canceled: bool
) {
    log!(
        "📨 [RECEIVE] Player moved: player={}, new_position={:?}, is_canceled={}",
        player_id,
        new_position,
        is_canceled
    );

    // Only start animation for non-local players (local player animation is handled in card_effect)
    let is_local_player = game_state.user == player_id;
    if is_local_player {
        log!("📨 [RECEIVE] Local player moved, skipping animation (already handled)");
        return;
    }

    // Get board layout before mutable borrow
    let (_, _, tile_size, offset_x, offset_y) = game_state.get_board_layout(false);

    // Get the current position of the player
    if let Some(player) = game_state.get_player_by_user_id(player_id) {
        let current_position = player.position;
        log!(
            "📨 [RECEIVE] Player {:?} moving from {:?} to {:?}",
            player.id,
            current_position,
            new_position
        );

        if is_canceled {
            // Use direct animation for canceled movements
            start_direct_player_movement_animation(
                game_state,
                player_id,
                current_position,
                *new_position,
                tile_size,
                offset_x,
                offset_y
            );
        } else {
            // Use path-based animation for normal movements
            start_player_movement_animation(
                game_state,
                player_id,
                current_position,
                *new_position,
                tile_size,
                offset_x,
                offset_y
            );
        }
    } else {
        log!("📨 [RECEIVE] Could not find PlayerId for user_id: {}", player_id);
    }
}

pub fn receive_card_confirmed(game_state: &mut GameState, card: &Card, player_id: &str) {
    log!("📨 [RECEIVE] Card confirmed by {}: {:?}", player_id, card);

    if game_state.user == player_id {
        game_state.selected_card = None;
        clear_highlights(&mut game_state.tiles);
    }
}

pub fn receive_tiles_swapped(
    game_state: &mut GameState,
    tile_index_1: &usize,
    tile_index_2: &usize
) {
    log!("📨 [RECEIVE] Tiles swapped: {} <-> {}", tile_index_1, tile_index_2);
    // For tile swaps, we animate first, then swap when animation completes
    // This keeps the indices consistent during animation

    // Track this swap to be performed when animation completes
    game_state.pending_swaps.push((*tile_index_1, *tile_index_2));

    // Start animations for both tiles
    animate_tile_to_index(game_state, *tile_index_1, *tile_index_2);
    animate_tile_to_index(game_state, *tile_index_2, *tile_index_1);
}

pub fn receive_fireball_shot(
    game_state: &mut GameState,
    player_id: &str,
    tile_index: &usize,
    direction: &crate::game::map::tile::Direction
) {
    log!("📨 [RECEIVE] Fireball shot: index={}, direction={:?}", tile_index, direction);

    let is_local_player = game_state.user == player_id;
    if is_local_player {
        return;
    }

    let player = game_state.get_player_by_user_id(player_id).unwrap();
    let position = player.position;

    // Create fireball
    let fireball = Fireball::new(
        10,
        Tile::position(*tile_index),
        direction.clone(),
        player.id.clone()
    );
    let fireball_id = fireball.id;
    game_state.fireballs.push(fireball);

    start_fireball_animation(game_state, fireball_id, position, direction.clone(), *tile_index);
}

pub fn receive_fireball_hit_result(
    game_state: &mut GameState,
    player_id: &str,
    target_id: &str,
    damage_dealt: &u32,
    monster_damage: Option<u32>
) {
    log!(
        "📨 [RECEIVE] Fireball hit result: player={}, target={:?}, damage={}, monster_damage={:?}",
        player_id,
        target_id,
        damage_dealt,
        monster_damage
    );

    // Handle monster damage if present
    if let Some(damage) = monster_damage {
        if let Some(monster) = &mut game_state.monster {
            log!("📨 [RECEIVE] Monster took {} damage from fireball", damage);
            monster.take_damage(damage);
        }
    }

    // Handle player damage if target is a player
    if let Some(player) = game_state.get_player_by_user_id(target_id) {
        log!("📨 [RECEIVE] Player {} took {} damage from fireball", target_id, damage_dealt);
        player.take_damage(*damage_dealt);
    }

    game_state.fireballs.clear();
    game_state.animated_fireballs.clear();
}

pub fn receive_player_damage_from_monster(
    game_state: &mut GameState,
    player_id: &str,
    damage_dealt: u32
) {
    log!("📨 [RECEIVE] Player {} took {} damage from monster", player_id, damage_dealt);
    if let Some(player) = game_state.get_player_by_user_id(player_id) {
        player.take_damage(damage_dealt);
    }
}

pub fn receive_game_over(game_state: &mut GameState, winner_ids: &[String], loser_ids: &[String]) {
    if winner_ids.len() > 1 && loser_ids.is_empty() {
        log!("🏆 [RECEIVE] Game Over! Both players win: {:?}", winner_ids);
        // Both players defeated the monster together
        game_state.game_over_cooperative(winner_ids, &[]);
    } else if winner_ids.is_empty() && loser_ids.len() > 1 {
        log!("💀 [RECEIVE] Game Over! Both players lose: {:?}", loser_ids);
        // Both players were defeated by the monster
        game_state.game_over_cooperative(&[], loser_ids);
    } else {
        log!(
            "❓ [RECEIVE] Game Over! Unexpected state - winners: {:?}, losers: {:?}",
            winner_ids,
            loser_ids
        );
        game_state.game_over_cooperative(&[], &["Error".to_string()]);
    }
}

pub fn receive_reset_game(game_state: &mut GameState) {
    log!("🔄 [RECEIVE] Game reset received");
    game_state.animated_card = None;
    game_state.animated_player = None;
    game_state.animated_tiles.clear();
    game_state.animated_fireballs.clear();
    game_state.fireballs.clear();
    game_state.selected_card = None;
    game_state.swap_tiles_selected.clear();
    game_state.pending_swaps.clear();
    game_state.reset_turn();
    game_state.play_area.clear();
    game_state.play_area = vec![Card::dummy_card(); HAND_SIZE];
}
