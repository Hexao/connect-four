use super::{Board, Message as BoardMessage};

use iced::{
    Application, Command, Subscription,
    time, Element, Canvas, Length,
};

pub enum Menu {
    Start,
    Game(Board)
}

impl Application for Menu {
    type Executor = iced::executor::Default;
    type Message = BoardMessage;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<BoardMessage>) {
        let p1 = Box::new(crate::behaviour::Human);
        let p2 = Box::new(crate::behaviour::Rollout::default());

        (Menu::Game(Board::new(p1, p2)), Command::none())
    }

    fn title(&self) -> String {
        String::from("Connect four")
    }

    fn update(&mut self, message: BoardMessage) -> Command<BoardMessage> {
        if let Menu::Game(board) = self {
            board.handle_message(message)
        } else {
            Command::none()
        }
    }

    fn subscription(&self) -> Subscription<BoardMessage> {
        use std::time::Duration;

        match self {
            Menu::Start => todo!(),
            Menu::Game(board) => {
                if !board.animation_finished() {
                    time::every(Duration::from_millis(16)).map(BoardMessage::Tick)
                } else {
                    Subscription::none()
                }
            }
        }
    }

    fn view(&mut self) -> Element<BoardMessage> {
        match self {
            Menu::Start => todo!(),
            Menu::Game(board) => {
                Canvas::new(board)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}
