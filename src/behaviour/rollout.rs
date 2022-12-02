use super::{Behaviour, Intent};
use crate::board_game::Game;

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
                    Ok(None) => (),
                    Err(_) => {
                        *score = LOSE_SCORE * 2.0;
                        continue;
                    },
                    Ok(Some(_)) => {
                        *score = WIN_SCORE;
                        continue;
                    },
                }

                for _ in 0..iter {
                    let mut game = start_state;

                    for _ in 1..deep {
                        let possibilities = (0..Game::COL)
                            .filter(|&col| !game.col_full(col))
                            .collect::<Vec<usize>>();

                        let Some(&col) = possibilities.choose(&mut rand) else {
                            break;
                        };

                        match game.play_col(col) {
                            Err(_) => unreachable!(),
                            Ok(None) => (),
                            Ok(Some(_)) => {
                                intent_score += if game.player_turn() == whoami {
                                    // on game.play_col() player turn change. So if player
                                    // turn is mine, this mean i just lose the game.
                                    LOSE_SCORE
                                } else {
                                    WIN_SCORE
                                };

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
                    if score - max >= -f32::EPSILON {
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
            self.handle = Some(handle);
            Intent::Waiting
        }
    }
}

impl Default for Rollout {
    fn default() -> Self {
        Self { iter: 250, deep: 5, handle: None }
    }
}
