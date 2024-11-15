use iced::widget::{button, center, column, container, qr_code, row, text, toggler, Column};
use iced::{Center, Element, Fill, Font};

use crate::icon;
use crate::store::Address;

#[derive(Debug, Clone, Default)]
pub struct State {
    coin_address: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddressKindToggle(bool),
    CopyPress(String),
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    WriteClipboard(String),
}

pub fn update(state: &mut State, message: Message) -> Task {
    match message {
        Message::AddressKindToggle(coin_address) => {
            state.coin_address = coin_address;
            Task::None
        }
        Message::CopyPress(s) => Task::WriteClipboard(s),
    }
}

pub fn view<'a>(
    state: &'a State,
    coin_address: Option<&'a Address>,
    space_address: Option<&'a Address>,
) -> Element<'a, Message> {
    let address = if state.coin_address {
        coin_address
    } else {
        space_address
    };
    Column::new()
        .push(
            container(
                toggler(state.coin_address)
                    .label("Coins only address")
                    .on_toggle(Message::AddressKindToggle),
            )
            .align_x(Center)
            .width(Fill),
        )
        .push_maybe(address.map(|address| {
            center(
                column![
                    row![
                        text(&address.text).font(Font::MONOSPACE),
                        button(text(icon::COPY).font(icon::FONT))
                            .on_press(Message::CopyPress(address.text.clone())),
                    ]
                    .align_y(Center),
                    qr_code(&address.qr_code).cell_size(7),
                ]
                .align_x(Center)
                .spacing(10),
            )
        }))
        .into()
}
