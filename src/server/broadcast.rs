use turbo::*;
use borsh::BorshSerialize;
use serde::Serialize;
use crate::server::{ ServerToClient, CurrentTurn };
use crate::game::map::{ Tile, Player, Monster };
use crate::game::cards::card::Card;

pub fn broadcast_generic<T: Serialize + BorshSerialize>(msg: T) {
    if let Err(e) = os::server::channel::broadcast(msg) {
        log!("[GameChannel] Error broadcasting message: {e}");
    }
}

pub fn broadcast_turn(
    players: &[String],
    current_turn_index: usize,
    current_turn: &mut Option<CurrentTurn>,
    board_tiles: &[Tile],
    board_players: &[Player],
    board_monster: &Option<Monster>
) {
    if let Some(user_id) = players.get(current_turn_index) {
        // Update current_turn with the new player
        *current_turn = Some(CurrentTurn {
            player_id: user_id.clone(),
            selected_card: None,
            selected_card_index: 0,
        });
        broadcast_board_state(board_tiles, board_players, board_monster, current_turn);
    }
}

pub fn broadcast_board_state(
    board_tiles: &[Tile],
    board_players: &[Player],
    board_monster: &Option<Monster>,
    current_turn: &Option<CurrentTurn>
) {
    broadcast_generic(ServerToClient::BoardState {
        tiles: board_tiles.to_vec(),
        players: board_players.to_vec(),
        monster: board_monster.clone(),
        current_turn: current_turn.clone(),
    });
}

pub fn broadcast_card_cancelled(card: &Card, player_id: &str) {
    broadcast_generic(ServerToClient::CardCancelled {
        card: card.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_card_confirmed(card: &Card, player_id: &str) {
    broadcast_generic(ServerToClient::CardConfirmed {
        card: card.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_tile_rotation(tile_index: usize, tile: &Tile, player_id: &str) {
    broadcast_generic(ServerToClient::TileRotated {
        tile_index,
        tile: tile.clone(),
        player_id: player_id.to_string(),
    });
}

pub fn broadcast_player_moved(player_id: &str, new_position: (usize, usize), is_canceled: bool) {
    broadcast_generic(ServerToClient::PlayerMoved {
        player_id: player_id.to_string(),
        new_position,
        is_canceled,
    });
}

pub fn broadcast_tiles_swapped(tile_index_1: usize, tile_index_2: usize) {
    broadcast_generic(ServerToClient::TilesSwapped {
        tile_index_1,
        tile_index_2,
    });
}

pub fn broadcast_fireball_shot(
    player_id: &str,
    target_tile: usize,
    direction: &crate::game::map::tile::Direction
) {
    broadcast_generic(ServerToClient::FireballShot {
        player_id: player_id.to_string(),
        tile_index: target_tile,
        direction: direction.clone(),
    });
}

pub fn broadcast_fireball_hit_result(
    shooter_id: &str,
    target_player_id: &str,
    damage_dealt: &u32,
    monster_damage: Option<u32>
) {
    broadcast_generic(ServerToClient::FireballHit {
        player_id: shooter_id.to_string(),
        target_id: target_player_id.to_string(),
        damage_dealt: *damage_dealt,
        monster_damage,
    });
}

pub fn broadcast_player_damage_from_monster(player_id: &str, damage_dealt: u32) {
    broadcast_generic(ServerToClient::PlayerDamageFromMonster {
        player_id: player_id.to_string(),
        damage_dealt,
    });
}

pub fn broadcast_game_over(winner_ids: &[String], loser_ids: &[String]) {
    broadcast_generic(ServerToClient::GameOver {
        winner_ids: winner_ids.to_vec(),
        loser_ids: loser_ids.to_vec(),
    });
}
