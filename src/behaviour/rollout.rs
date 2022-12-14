use crate::board_game::{Game, PlayResult};
use super::{Behaviour, Intent};

use rand::{prelude::SliceRandom, seq::IteratorRandom};

pub struct Rollout {
    iter: u16,
    deep: u16,

    handle: Option<std::thread::JoinHandle<u8>>,
}

impl Behaviour for Rollout {
    fn start_process(&mut self, state: Game) {
        let Rollout { iter, deep, .. } = *self;
        let whoami = state.player_turn();

        const LOSE_SCORE: f32 = -5.0;
        const WIN_SCORE: f32 = 1.0;

        let handle = std::thread::spawn(move || {
            let mut rand = rand::thread_rng();
            let mut score = [0.0; Game::COL];

            for (intent, score) in score.iter_mut().enumerate() {
                let mut start_state = state;
                let mut intent_score = 0.0;

                match start_state.play_col(intent) {
                    PlayResult::Pass => (),
                    PlayResult::Error => {
                        *score = LOSE_SCORE * 2.0;
                        continue;
                    },
                    PlayResult::Win(_) => {
                        *score = WIN_SCORE;
                        continue;
                    },
                }

                for _ in 0..iter {
                    let mut game = start_state;

                    for actual_deep in 1..deep {
                        let possibilities = (0..Game::COL)
                            .filter(|&col| !game.col_full(col))
                            .collect::<Vec<usize>>();

                        let Some(&col) = possibilities.choose(&mut rand) else {
                            break;
                        };

                        match game.play_col(col) {
                            PlayResult::Error => unreachable!(),
                            PlayResult::Pass => (),
                            PlayResult::Win(_) => {
                                let coef =  (deep - actual_deep) as f32 / deep as f32;
                                let score = if game.player_turn() == whoami {
                                    // on game.play_col() player turn change. So if player
                                    // turn is mine, this mean i just lose the game.
                                    LOSE_SCORE
                                } else {
                                    WIN_SCORE
                                };

                                intent_score += score * coef;
                                break;
                            }
                        }
                    }
                }

                *score = intent_score / iter as f32;
            }

            let max = *score.iter()
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            score.into_iter()
                .enumerate()
                .filter_map(|(intent, score)| {
                    if max - score <= f32::EPSILON {
                        Some(intent)
                    } else {
                        None
                    }
                })
                .choose(&mut rand)
                .unwrap() as u8
        });

        self.handle = Some(handle);
    }

    fn intent(&mut self) -> Intent {
        let Some(handle) = self.handle.take() else {
            return Intent::None;
        };

        if handle.is_finished() {
            let Ok(play) = handle.join() else {
                panic!("Rollout::intent: failed to join thread !");
            };

            Intent::Some(play)
        } else {
            Intent::Waiting(handle)
        }
    }
}

impl Default for Rollout {
    fn default() -> Self {
        Self { iter: 250, deep: 5, handle: None }
    }
}
