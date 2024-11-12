use iced::widget::{center, text};
use iced::{Element, Task};

use crate::app::Message;

pub fn update() -> (bool, Task<Message>) {
    (true, Task::none())
}

pub fn view<'a>() -> Element<'a, Message> {
    center(text("Transactions")).into()
}
