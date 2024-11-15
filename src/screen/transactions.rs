use iced::widget::{center, text};
use iced::Element;

#[derive(Debug, Clone)]
pub enum Message {
    TransactionClicked(String),
}

pub fn view<'a>() -> Element<'a, Message> {
    center(text("Transactions")).into()
}
