use turbo::os::server::{ self, command::CommandHandler };
use crate::{ GameState, Card, HAND_SIZE };

#[turbo::program]
pub mod commands {
    use super::*;

    #[turbo::command(name = "next_turn")]
    pub struct NextTurn;

    impl turbo::CommandHandler for NextTurn {
        fn run(&mut self, _user_id: &str) -> Result<(), std::io::Error> {
            let mut state: GameState = server::fs::read("state")?;
            state.current_turn = (state.current_turn + 1) % state.players.len();
            server::fs::write("state", &state)?;
            Ok(())
        }
    }

    #[turbo::command(name = "deal_card")]
    pub struct DealCard;

    impl turbo::CommandHandler for DealCard {
        fn run(&mut self, _user_id: &str) -> Result<(), std::io::Error> {
            let mut state: GameState = server::fs::read("state")?;
            let player = &mut state.players[state.current_turn];
            if player.hand.len() < HAND_SIZE {
                player.hand.push(Card::move_card());
            }
            server::fs::write("state", &state)?;
            Ok(())
        }
    }
}
