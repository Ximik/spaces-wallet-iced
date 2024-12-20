use iced::widget::{center, Column};
use iced::Element;

use crate::helper::input;
use crate::{
    helper::input::*,
    widget::{block::error, form::Form},
};

#[derive(Debug, Clone, Default)]
pub struct State {
    recipient: String,
    amount: String,
    fee_rate: String,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    RecipientInput(String),
    AmountInput(String),
    FeeRateInput(String),
    SendSubmit,
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    SendCoins {
        recipient: String,
        amount: Amount,
        fee_rate: Option<FeeRate>,
    },
}

impl State {
    pub fn set_error(&mut self, message: String) {
        self.error = Some(message);
    }

    pub fn update(&mut self, message: Message) -> Task {
        match message {
            Message::RecipientInput(recipient) => {
                if input::recipient_chars(&recipient) {
                    self.recipient = recipient;
                }
                Task::None
            }
            Message::AmountInput(amount) => {
                if amount_chars(&amount) {
                    self.amount = amount
                }
                Task::None
            }
            Message::FeeRateInput(fee_rate) => {
                if fee_rate_chars(&fee_rate) {
                    self.fee_rate = fee_rate
                }
                Task::None
            }
            Message::SendSubmit => {
                self.error = None;
                Task::SendCoins {
                    recipient: recipient_value(&self.recipient).unwrap(),
                    amount: amount_value(&self.amount).unwrap(),
                    fee_rate: fee_rate_value(&self.fee_rate).unwrap(),
                }
            }
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        center(
            Column::new()
                .push_maybe(self.error.as_ref().map(error))
                .push(
                    Form::new(
                        "Send",
                        (recipient_value(&self.recipient).is_some()
                            && amount_value(&self.amount).is_some()
                            && fee_rate_value(&self.fee_rate).is_some())
                        .then_some(Message::SendSubmit),
                    )
                    .add_labeled_input("Amount", "sat", &self.amount, Message::AmountInput)
                    .add_labeled_input(
                        "To",
                        "bitcoin address or @space",
                        &self.recipient,
                        Message::RecipientInput,
                    )
                    .add_labeled_input(
                        "Fee rate",
                        "sat/vB (auto if empty)",
                        &self.fee_rate,
                        Message::FeeRateInput,
                    ),
                ),
        )
        .padding(20)
        .into()
    }
}
