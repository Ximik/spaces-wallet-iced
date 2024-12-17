use iced::widget::{center, column, Column};
use iced::Element;

use crate::store::{Amount, Denomination};
use crate::widget::{
    block::error,
    form::{labeled_input, submit_button},
};

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
    let is_valid = validate(&state.recipient, &state.amount).is_some();
    let maybe_submit = is_valid.then_some(Message::SendPress);
    center(
        Column::new()
            .push_maybe(state.error.as_ref().map(error))
            .push(
                column![
                    labeled_input(
                        "Amount",
                        "amount in sat",
                        &state.amount,
                        Message::AmountInput,
                        maybe_submit.clone(),
                    ),
                    labeled_input(
                        "To",
                        "bitcoin address or @space",
                        &state.recipient,
                        Message::RecipientInput,
                        maybe_submit.clone(),
                    ),
                ]
                .spacing(5),
            )
            .push(submit_button("Send", maybe_submit))
            .spacing(10),
    )
    .padding(20)
    .into()
}
