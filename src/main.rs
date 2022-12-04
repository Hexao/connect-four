use iced::{Application, window::{self, Icon}};
use image::{GenericImageView, io::Reader as ImReader};

// TODO: remove pub
pub mod board_game;
pub mod behaviour;
pub mod animator;

use board_game::Menu;

pub fn main() -> iced::Result {
    const WINDOW_SIZE: u32 = 720;

    let path = "./icon/connect-four.png";
    let icon = match ImReader::open(path) {
        Ok(buffer) => match buffer.decode() {
            Ok(img) => {
                let (width, height) = img.dimensions();
                let rgba = img.into_rgba8().into_raw();

                Icon::from_rgba(rgba, width, height).ok()
            }
            _ => None,
        }
        _ => None,
    };

    Menu::run(iced::Settings {
        antialiasing: true,
        window: window::Settings {
            size: (WINDOW_SIZE, WINDOW_SIZE),
            resizable: false, icon,
            .. Default::default()
        },
        .. Default::default()
    })
}
