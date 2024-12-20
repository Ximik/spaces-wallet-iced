use iced::widget::{button, center, column, container, text, text_input, Column};
use iced::Alignment::Center;
use iced::Length::Shrink;
use iced::{Element, Fill, Theme};

use crate::store::{Amount, Denomination};

#[derive(Debug, Clone, Default)]
pub struct State {
    recipient: String,
    amount: String,
    error: Option<String>,
}

impl State {
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    RecipientInput(String),
    AmountInput(String),
    SendPress,
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    SendCoins { recipient: String, amount: Amount },
}

fn validate(recipient: &String, amount: &String) -> Option<(String, Amount)> {
    if recipient.is_empty() {
        return None;
    }
    Amount::from_str_in(amount, Denomination::Satoshi)
        .ok()
        .map(|amount| (recipient.clone(), amount))
}

pub fn update(state: &mut State, message: Message) -> Task {
    match message {
        Message::RecipientInput(recipient) => {
            if recipient
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '@')
            {
                state.recipient = recipient;
            }
            Task::None
        }
        Message::AmountInput(amount) => {
            if amount.chars().all(|c| c.is_digit(10)) {
                state.amount = amount
            }
            Task::None
        }
        Message::SendPress => {
            state.error = None;
            if let Some((recipient, amount)) = validate(&state.recipient, &state.amount) {
                Task::SendCoins { recipient, amount }
            } else {
                Task::None
            }
        }
    }
}

pub fn view<'a>(state: &'a State) -> Element<'a, Message> {
    center(
        Column::new()
            .push_maybe(state.error.as_ref().map(|error| {
                container(
                    text(error)
                        .style(|theme: &Theme| text::Style {
                            color: Some(theme.extended_palette().danger.base.text),
                        })
                        .center()
                        .width(Fill),
                )
                .style(|theme: &Theme| {
                    container::Style::default()
                        .background(theme.extended_palette().danger.base.color)
                })
                .width(Fill)
                .padding([10, 30])
            }))
            .push(
                column![
                    text("Recipient address"),
                    text_input("", &state.recipient)
                        .on_input(Message::RecipientInput)
                        .padding(10),
                    text("Amount in SAT"),
                    text_input("", &state.amount)
                        .on_input(Message::AmountInput)
                        .padding(10),
                ]
                .spacing(5),
            )
            .push(
                container(
                    button("Send")
                        .on_press_maybe(
                            validate(&state.recipient, &state.amount).map(|_| Message::SendPress),
                        )
                        .padding([10, 20])
                        .width(Shrink),
                )
                .align_x(Center)
                .width(Fill),
            )
            .spacing(10),
    )
    .padding(20)
    .into()
}
