mod random;
mod human;

pub use random::Random;
pub use human::Human;

use crate::board_game::Game;

pub enum Intent {
    None, Waiting,
    Some(u8),
}

pub trait Behaviour {
    fn process_intent(&self) -> bool {
        false
    }

    fn start_process(&mut self, state: Game);
    fn intent(&self) -> Intent;
}
