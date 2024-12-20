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

#[derive(Debug, Clone)]
pub enum Message {
    RecipientInput(String),
    AmountInput(String),
    SendSubmit,
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

impl State {
    pub fn set_error(&mut self, message: String) {
        self.error = Some(message);
    }

    pub fn update(&mut self, message: Message) -> Task {
        match message {
            Message::RecipientInput(recipient) => {
                if recipient
                    .chars()
                    .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '@')
                {
                    self.recipient = recipient;
                }
                Task::None
            }
            Message::AmountInput(amount) => {
                if amount.chars().all(|c| c.is_digit(10)) {
                    self.amount = amount
                }
                Task::None
            }
            Message::SendSubmit => {
                self.error = None;
                if let Some((recipient, amount)) = validate(&self.recipient, &self.amount) {
                    Task::SendCoins { recipient, amount }
                } else {
                    Task::None
                }
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let is_valid = validate(&self.recipient, &self.amount).is_some();
        let send_submit = is_valid.then_some(Message::SendSubmit);
        center(
            Column::new()
                .push_maybe(self.error.as_ref().map(error))
                .push(
                    column![
                        labeled_input(
                            "Amount",
                            "amount in sat",
                            &self.amount,
                            Message::AmountInput,
                            send_submit.clone(),
                        ),
                        labeled_input(
                            "To",
                            "bitcoin address or @space",
                            &self.recipient,
                            Message::RecipientInput,
                            send_submit.clone(),
                        ),
                    ]
                    .spacing(5),
                )
                .push(submit_button("Send", send_submit))
                .spacing(10),
        )
        .padding(20)
        .into()
    }
}
