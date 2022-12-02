use super::{Behaviour, Intent};
use crate::board_game::Game;

use rand::prelude::SliceRandom;
const INVALID: u8 = Game::COL as u8;

pub struct Random {
    generator: rand::rngs::ThreadRng,
    last_gen: u8,
}

impl Behaviour for Random {
    fn start_process(&mut self, state: Game) {
        let possibilities = (0..Game::COL)
            .filter(|&col| !state.col_full(col))
            .collect::<Vec<usize>>();

        let col = possibilities
            .choose(&mut self.generator)
            .unwrap_or(&3);

        self.last_gen = *col as u8;
    }

    fn intent(&mut self) -> Intent {
        if self.last_gen == INVALID {
            Intent::None
        } else {
            Intent::Some(std::mem::replace(&mut self.last_gen, INVALID))
        }
    }
}

impl Default for Random {
    fn default() -> Self {
        Self { generator: rand::thread_rng(), last_gen: INVALID }
    }
}
