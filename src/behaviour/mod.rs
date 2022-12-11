mod rollout;
mod random;
mod human;

pub use rollout::Rollout;
pub use random::Random;
pub use human::Human;

use crate::board_game::Game;

pub enum Intent {
    Waiting(std::thread::JoinHandle<u8>),
    Some(u8),
    None,
}

pub trait Behaviour {
    fn process_intent(&self) -> bool {
        false
    }

    fn start_process(&mut self, state: Game);
    fn intent(&mut self) -> Intent;
}
