use super::{Behaviour, Intent};
use crate::board_game::Game;

pub struct Human;

impl Behaviour for Human {
    fn process_intent(&self) -> bool {
        true
    }

    fn start_process(&mut self, _state: Game) {
    }

    fn intent(&self) -> Intent {
        Intent::None
    }
}
