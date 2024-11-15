use iced::widget::{button, center, column, container, text, text_input};
use iced::{Element, Fill, Font, Theme};
use protocol::Covenant;

use crate::icon;
use crate::store::{Amount, Denomination};

#[derive(Debug, Clone, Default)]
pub struct State {
    bid_amount: String,
    error: Option<String>,
}

impl State {
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    SpaceNameInput(String),
    BidAmountInput(String),
    BidPress(String),
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    SetSpace {
        space_name: String,
    },
    BidSpace {
        space_name: String,
        bid_amount: Amount,
    },
}

fn validate(bid_amount: &String) -> Option<Amount> {
    Amount::from_str_in(bid_amount, Denomination::Bitcoin).ok()
}

pub fn update(state: &mut State, message: Message) -> Task {
    match message {
        Message::SpaceNameInput(space_name) => {
            if space_name
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
            {
                Task::SetSpace { space_name }
            } else {
                Task::None
            }
        }
        Message::BidAmountInput(bid_amount) => {
            if bid_amount.chars().all(|c| c.is_digit(10) || c == '.') {
                state.bid_amount = bid_amount
            }
            Task::None
        }
        Message::BidPress(space_name) => {
            state.error = None;
            if let Some(bid_amount) = validate(&state.bid_amount) {
                Task::BidSpace {
                    space_name,
                    bid_amount,
                }
            } else {
                Task::None
            }
        }
    }
}

pub fn view<'a>(
    state: &'a State,
    space_name: &'a String,
    space_covenant: Option<&'a Option<Covenant>>,
    in_wallet: bool,
) -> Element<'a, Message> {
    let bid_form = |is_new: bool| {
        column![
            text(if is_new {
                "This space doesn't exist. You can open it."
            } else {
                "This space exists. You can bid on it."
            }),
            text_input("amount", &state.bid_amount).on_input(Message::BidAmountInput),
            button(if is_new { "Open" } else { "Bid" }).on_press_maybe(
                validate(&state.bid_amount).map(|_| Message::BidPress(space_name.clone()))
            ),
        ]
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
                container::Style::default().background(theme.extended_palette().danger.base.color)
            })
            .width(Fill)
        }))
    };

    let main: Element<'a, Message> = match space_covenant {
        None => text("Loading").into(),
        Some(None) => bid_form(true).into(),
        Some(Some(Covenant::Bid { .. })) => bid_form(false).into(),
        Some(Some(Covenant::Transfer { .. })) => text("This space is owned").into(),
        Some(Some(Covenant::Reserved)) => text("Reserved state").into(),
    };

    column![
        text_input("space", space_name)
            .on_input(Message::SpaceNameInput)
            .font(Font::MONOSPACE)
            .icon(text_input::Icon {
                font: icon::FONT,
                code_point: icon::AT,
                size: None,
                spacing: 10.0,
                side: text_input::Side::Left,
            })
            .padding(10),
        center(main),
    ]
    .spacing(10)
    .into()
}
