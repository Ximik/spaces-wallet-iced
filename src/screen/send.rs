use iced::widget::{button, column, container, text, text_input};
use iced::{Element, Fill, Task, Theme};

use crate::app::{Message, RpcRequest, Screen};
use crate::store::Amount;

pub fn update(address: &String, amount: &String) -> (bool, Task<Message>) {
    (
        address
            .chars()
            .all(|f| f.is_ascii_digit() || f.is_ascii_lowercase())
            && (amount.is_empty() || amount.parse::<f64>().map_or(false, |n| n >= 0.0)),
        Task::none(),
    )
}

pub fn view<'a>(
    address: &'a String,
    amount: &'a String,
    error: &'a Option<String>,
) -> Element<'a, Message> {
    container(
        column![
            text_input("address", &address).on_input(|address| Message::UpdateScreen(
                Screen::Send {
                    address,
                    amount: amount.to_string(),
                    error: None,
                }
            )),
            text_input("amount", &amount).on_input(|amount| Message::UpdateScreen(Screen::Send {
                address: address.to_string(),
                amount,
                error: None,
            })),
            button("Send").on_press_maybe(
                amount
                    .parse::<f64>()
                    .ok()
                    .and_then(|v| Amount::from_btc(v).ok())
                    .map(|amount| Message::InvokeRpc(RpcRequest::SendCoins {
                        address: address.clone(),
                        amount
                    }))
            ),
        ]
        .push_maybe(error.as_ref().map(|error| {
            container(
                text(error)
                    .style(|theme: &Theme| text::Style {
                        color: Some(theme.extended_palette().danger.base.text),
                    })
                    .center()
                    .width(Fill),
            )
            .style(|theme: &Theme| {
                container::Style::default().background(theme.extended_palette().danger.base.color)
            })
            .width(Fill)
        }))
        .spacing(10),
    )
    .padding(10)
    .height(Fill)
    .into()
}
