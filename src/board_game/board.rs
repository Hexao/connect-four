use crate::animator::{Animation, Builder};
use crate::behaviour::{Behaviour, Intent};
use super::{Game, Player, PlayResult};

use iced::{
    widget::canvas, Point, Vector, Rectangle,
    Command, Color,
};

use std::time::Instant;

#[derive(Debug)]
pub enum Message {
    Tick(Instant),
    Slide(u8),
    Play(u8),
    Restart,
}

pub struct Board {
    game_state: canvas::Cache,
    animator: canvas::Cache,
    board: canvas::Cache,

    sector: u8,
    now: Instant,
    animation: Animation,
    board_state: BoardState,

    game: Game,
    p1: Box<dyn Behaviour>,
    p2: Box<dyn Behaviour>,
}

impl Board {
    const YELLOW_PLAYER: Color = Color::from_rgb(0.8, 0.8, 0.1);
    const RED_PLAYER: Color = Color::from_rgb(0.8, 0.1, 0.1);

    const BACKGROUND: Color = Color::from_rgb(0.275, 0.47, 0.785);
    const BOARD_COLOR: Color = Color::from_rgb(0.1, 0.1, 0.5);
    const WIN_COLOR: Color = Color::from_rgb(0.1, 1.0, 0.1);

    const GRID_OPENING: f32 = 0.8;
    const COIN_SIZE: f32 = 0.85;

    pub fn new(mut p1: Box<dyn Behaviour>, p2: Box<dyn Behaviour>) -> Self {
        let animation = Builder::default()
            .move_curve(Point::new(Game::COL as f32 / 2.0, -0.5), Vector::new(0.0, 1.0))
            .anim_duration(0.5).build();

        let game = Game::default();
        p1.start_process(game);

        Self {
            game_state: canvas::Cache::default(),
            animator: canvas::Cache::default(),
            board: canvas::Cache::default(),

            sector: 3,
            now: Instant::now(),
            animation,
            board_state: BoardState::Initialize,

            game, p1, p2,
        }
    }

    pub fn handle_message(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(now) => {
                return self.update(now);
            },
            Message::Slide(sector) => {
                self.slide(sector);
            },
            Message::Play(sector) => {
                self.play(sector);
            },
            Message::Restart => {
                self.restart();
            },
        }

        Command::none()
    }

    pub fn animation_finished(&self) -> bool {
        self.animation.finished_at(self.now)
    }

    fn update(&mut self, now: Instant) -> Command<Message> {
        self.animator.clear();
        self.now = now;

        if self.animation.finished_at(now) {
            let action =  match self.board_state {
                BoardState::Initialize => {
                    match self.behaviour_mut().intent() {
                        Intent::None => BoardState::Waiting,
                        Intent::Some(sector) => {
                            self.board_state.new_action(BoardState::Waiting);
                            self.play(sector);

                            return Command::none();
                        },
                        Intent::Waiting => {
                            let handle = self.behaviour_mut().handle();
                            self.board_state.new_action(BoardState::Waiting);

                            return Command::perform(wait_handle(handle), Message::Play);
                        }
                    }
                },
                BoardState::SlideThenPlay => {
                    let height = self.game.col_height(self.sector as usize);

                    if height != Game::ROW {
                        self.play_current_sector(height);
                        BoardState::Playing
                    } else {
                        BoardState::Waiting
                    }
                },
                BoardState::Playing => {
                    self.game_state.clear();

                    if let PlayResult::Win([x, y, dx, dy]) = self.game.play_col(self.sector as usize) {
                        self.sliding_curve();

                        let start = Point { x: 0.5 + x as f32, y: 0.5 + Game::ROW as f32 - y as f32 };
                        let end = Point { x: 0.5 + dx as f32, y: 0.5 + Game::ROW as f32 - dy as f32 };
                        let direction = end - start;

                        self.animation.update_axis(start, direction);
                        self.animation.update_duration(1.0);
                        self.animation.restart();
                        BoardState::Win
                    } else if self.game.grid_full() {
                        BoardState::Finished
                    } else {
                        self.initialize_coin();
                        self.sector = 3;

                        let state = self.game;
                        self.behaviour_mut().start_process(state);

                        BoardState::Initialize
                    }
                }
                _ => BoardState::Waiting
            };

            self.board_state.new_action(action);
        }

        Command::none()
    }

    fn slide(&mut self, sector: u8) {
        let Some(old) = self.board_state.new_action(BoardState::Sliding) else {
            return;
        };

        self.slide_sector(sector);

        if old == BoardState::Sliding {
            self.animation.update_ctrl(
                Point { x: 0.4, y: 0.4 },
                Point { x: 0.5, y: 1.0 }
            );
        } else {
            self.sliding_curve();
        }
    }

    fn play(&mut self, sector: u8) {
        if self.sector == sector && self.board_state == BoardState::Waiting {
            let height = self.game.col_height(self.sector as usize);
            if height == Game::ROW { return; }

            if self.board_state.new_action(BoardState::Playing).is_some() {
                self.play_current_sector(height);
            };
        } else if self.board_state.new_action(BoardState::SlideThenPlay).is_some() {
            self.slide_sector(sector);
        }
    }

    fn restart(&mut self) {
        if self.board_state.finished() {
            self.game.restart();

            let game = self.game;
            self.behaviour_mut().start_process(game);

            self.board_state.new_action(BoardState::Initialize);
            self.animation.update_duration(0.5);
            self.animation.restart();
            self.game_state.clear();

            self.initialize_coin();
        }
    }

    fn initialize_coin(&mut self) {
        self.sliding_curve();
        self.animation.restart();
        self.animation.update_axis(
            Point::new(Game::COL as f32 / 2.0, -0.5),
            Vector::new(0.0, 1.0),
        );
    }

    fn sliding_curve(&mut self) {
        self.animation.update_ctrl(
            Point { x: 0.5, y: 0.0 },
            Point { x: 0.5, y: 1.0 }
        );
    }

    fn playing_curve(&mut self) {
        self.animation.update_ctrl(
            Point { x: 0.65, y: 0.0 },
            Point { x: 0.75, y: 0.5 }
        );
    }

    fn slide_sector(&mut self, sector: u8) {
        self.sector = sector;

        let start = self.animation.point_at(self.now);
        let coin_x = sector as f32 + 0.5;
        let coin_y = 0.5;

        let direction = Point{ x: coin_x, y: coin_y } - start;
        self.animation.update_axis(start, direction);
        self.animation.restart();
    }

    fn play_current_sector(&mut self, height: usize) {
        self.playing_curve();
        self.animation.restart();
        self.animation.update_axis(
            Point { x: self.sector as f32 + 0.5, y: 0.5 },
            Vector { x: 0.0, y: (Game::ROW - height) as f32 }
        );
    }

    fn behaviour(&self) -> &dyn Behaviour {
        match self.game.player_turn() {
            Player::Yellow => self.p2.as_ref(),
            Player::Red => self.p1.as_ref(),
        }
    }

    fn behaviour_mut(&mut self) -> &mut Box<dyn Behaviour> {
        match self.game.player_turn() {
            Player::Yellow => &mut self.p2,
            Player::Red => &mut self.p1,
        }
    }
}

fn offset_and_chunk_size(bounds: iced::Size) -> (Point, f32) {
    let iced::Size{ width, height } = bounds;
    if width >= height {
        (Point{ x: (width - height) / 2.0, y: 0.0 }, height / Game::COL as f32)
    } else {
        (Point{ x: 0.0, y: (height - width) / 2.0 }, width / Game::COL as f32)
    }
}

impl canvas::Program<Message> for Board {
    type State = ();

    fn draw(&self, _state: &Self::State, _theme: &iced::Theme, bounds: Rectangle, _cursor: canvas::Cursor) -> Vec<canvas::Geometry> {
        let (offset, chunk_size) = offset_and_chunk_size(bounds.size());

        let game_state = self.game_state.draw(bounds.size(), |frame| {
            let background = canvas::Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Self::BACKGROUND);

            let (mut x, mut y) = (0.0, Game::ROW as f32);

            for cell in &self.game.grid() {
                let coin = canvas::Path::rectangle(
                    Point { x: x * chunk_size, y: y * chunk_size },
                    iced::Size { width: chunk_size, height: chunk_size }
                );

                y -= 1.0;

                if y < 0.5 {
                    y = Game::ROW as f32;
                    x += 1.0;
                }

                let color = match cell {
                    Some(Player::Yellow) => Self::YELLOW_PLAYER,
                    Some(Player::Red) => Self::RED_PLAYER,
                    None => continue,
                };

                frame.fill(&coin, color);
            }
        });

        let animator = self.animator.draw(bounds.size(), |frame| {
            match self.board_state {
                BoardState::Finished => (),
                BoardState::Win => {
                    let rad = chunk_size * Self::COIN_SIZE * 0.1;

                    let start_coef = self.animation.start_point() - Point::ORIGIN;
                    let start_vec = start_coef * chunk_size;
                    let start_pos = offset + start_vec;

                    let end_coef = self.animation.point_at(self.now) - Point::ORIGIN;
                    let end_vec = end_coef * chunk_size;
                    let end_pos = offset + end_vec;

                    let start = canvas::Path::circle(start_pos, rad);
                    let end = canvas::Path::circle(end_pos, rad);

                    frame.fill(&start, Self::WIN_COLOR);
                    frame.fill(&end, Self::WIN_COLOR);

                    let Vector { x, y } = end_pos - start_pos;
                    let len = (x*x + y*y).sqrt();

                    frame.translate(start_pos - Point::ORIGIN);
                    frame.rotate(y.atan2(x));

                    let line = canvas::Path::rectangle(Point { x: 0.0, y: -rad }, iced::Size { width: len, height: rad * 2.0 });
                    frame.fill(&line, Self::WIN_COLOR);
                }
                _ => {
                    let coin_rad = chunk_size * Self::COIN_SIZE * 0.5;

                    let coin_coef = self.animation.point_at(self.now) - Point::ORIGIN;
                    let coin_vec = coin_coef * chunk_size;
                    let coin_pos = offset + coin_vec;

                    let coin = canvas::Path::circle(coin_pos, coin_rad);
                    let coin_color = match self.game.player_turn() {
                        Player::Yellow => Self::YELLOW_PLAYER,
                        Player::Red => Self::RED_PLAYER,
                    };

                    frame.fill(&coin, coin_color);
                }
            }
        });

        let board = self.board.draw(bounds.size(), |frame| {
            let as_f32 = |var| var as f32;
            let Point { x: ox, y: oy } = &offset;
            use std::f32::consts::TAU;

            for mut j in (0..Game::ROW).map(as_f32) {
                j += 1.5;

                for mut i in (0..Game::COL).map(as_f32) {
                    i += 0.5;

                    let tile = canvas::Path::new(|builder| {
                        builder.arc(canvas::path::Arc {
                            center: Point { x: ox + i * chunk_size, y: oy + j * chunk_size },
                            radius: chunk_size * 0.5 * Self::GRID_OPENING,
                            start_angle: 0.0, end_angle: TAU,
                        });

                        builder.line_to(Point { x: ox + (i + 0.5) * chunk_size, y: oy + (j - 0.0) * chunk_size });
                        builder.line_to(Point { x: ox + (i + 0.5) * chunk_size, y: oy + (j - 0.5) * chunk_size });
                        builder.line_to(Point { x: ox + (i - 0.5) * chunk_size, y: oy + (j - 0.5) * chunk_size });
                        builder.line_to(Point { x: ox + (i - 0.5) * chunk_size, y: oy + (j + 0.5) * chunk_size });
                        builder.line_to(Point { x: ox + (i + 0.5) * chunk_size, y: oy + (j + 0.5) * chunk_size });
                        builder.line_to(Point { x: ox + (i + 0.5) * chunk_size, y: oy + (j + 0.0) * chunk_size });
                    });

                    frame.fill(&tile, Self::BOARD_COLOR);
                }
            }
        });

        if self.board_state.finished() {
            vec![game_state, board, animator]
        } else {
            vec![game_state, animator, board]
        }
    }

    fn update(&self, _state: &mut Self::State, event: canvas::Event, bounds: Rectangle, cursor: canvas::Cursor) -> (canvas::event::Status, Option<Message>) {
        if !self.board_state.finished() && !self.behaviour().process_intent() {
            return (canvas::event::Status::Ignored, None);
        }

        let mut message = None;

        match event {
            canvas::Event::Mouse(ms_event) => match ms_event {
                iced::mouse::Event::CursorMoved { position } => {
                    let (offset, chunk_size) = offset_and_chunk_size(bounds.size());
                    let sector = ((position.x - offset.x) / chunk_size).clamp(0.0, Game::COL as f32 - 1.0) as u8;

                    if sector != self.sector {
                        message = Some(Message::Slide(sector));
                    }
                },
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    if let canvas::Cursor::Available(position) = cursor {
                        let (offset, chunk_size) = offset_and_chunk_size(bounds.size());
                        let sector = ((position.x - offset.x) / chunk_size).clamp(0.0, Game::COL as f32 - 1.0) as u8;

                        message = Some(Message::Play(sector));
                    }
                }
                _ => (),
            }
            canvas::Event::Keyboard(kb_event) => {
                if let iced::keyboard::Event::KeyPressed { key_code: iced::keyboard::KeyCode::R, .. } = kb_event {
                    message = Some(Message::Restart);
                }
            }
            canvas::Event::Touch(_) => (),
        }

        (canvas::event::Status::Ignored, message)
    }
}

#[derive(PartialEq, Eq)]
enum BoardState {
    Initialize,
    Waiting,
    Sliding,
    SlideThenPlay,
    Playing,
    Win,
    Finished,
}

impl BoardState {
    fn new_action(&mut self, action: Self) -> Option<Self> {
        match (self, action) {
            (init @ Self::Initialize, Self::Waiting) => {
                *init = Self::Waiting;
                Some(Self::Initialize)
            }
            (wait @ Self::Waiting, action) if action != Self::Initialize => {
                *wait = action;
                Some(Self::Waiting)
            },
            (slide @ Self::Sliding, action @ (Self::Waiting | Self::Sliding | Self::SlideThenPlay)) => {
                *slide = action;
                Some(Self::Sliding)
            },
            (slide @ Self::Sliding, Self::Playing) => {
                *slide = Self::SlideThenPlay;
                Some(Self::Sliding)
            },
            (slide @ Self::SlideThenPlay, action @ (Self::Playing | Self::Waiting)) => {
                *slide = action;
                Some(Self::SlideThenPlay)
            },
            (play @ Self::Playing, action @ (Self::Initialize | Self::Win | Self::Finished)) => {
                *play = action;
                Some(Self::Playing)
            },
            (win @ (Self::Win | Self::Finished), Self::Initialize) => {
                Some(std::mem::replace(win, Self::Initialize))
            },
            _ => None,
        }
    }

    fn finished(&self) -> bool {
        *self == BoardState::Win || *self == BoardState::Finished
    }
}

async fn wait_handle(handle: std::thread::JoinHandle<u8>) -> u8 {
    let Ok(play) = handle.join() else {
        panic!("wait_handle: failed to join thread !");
    };

    play
}
