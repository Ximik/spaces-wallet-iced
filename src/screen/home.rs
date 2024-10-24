use iced::widget::{center, text};
use iced::Element;

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug)]
pub struct Component {}

impl Component {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: Message) {}

    pub fn view(&self) -> Element<Message> {
        center(text("HOME")).into()
    }
}