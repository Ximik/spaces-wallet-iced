use iced::widget::{button, center, column, container, text, text_input, Column};
use iced::{Center, Element, Fill, Font, Shrink, Theme};

use crate::store::{Amount, Covenant, Denomination, SLabel};
use crate::widget::icon::{text_input_icon, Icon};

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
    BidPress(SLabel, bool),
    RegisterPress(SLabel),
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    SetSpace {
        space_name: String,
    },
    BidSpace {
        slabel: SLabel,
        amount: Amount,
        open: bool,
    },
    RegisterSpace {
        slabel: SLabel,
    },
}

fn validate(bid_amount: &String) -> Option<Amount> {
    Amount::from_str_in(bid_amount, Denomination::Satoshi).ok()
}

pub fn update(state: &mut State, message: Message) -> Task {
    state.error = None;
    match message {
        Message::SpaceNameInput(space_name) => {
            if space_name
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-')
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
        Message::BidPress(slabel, open) => {
            if let Some(amount) = validate(&state.bid_amount) {
                Task::BidSpace {
                    slabel,
                    amount,
                    open,
                }
            } else {
                Task::None
            }
        }
        Message::RegisterPress(slabel) => Task::RegisterSpace { slabel },
    }
}

pub fn view<'a>(
    state: &'a State,
    tip_height: u32,
    space_name: &'a String,
    space_data: Option<(SLabel, Option<&'a Option<Covenant>>, bool)>,
) -> Element<'a, Message> {
    let bid_form = |slabel: SLabel, total_burned: Option<&Amount>| {
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
                    if let Some(total_burned) = total_burned {
                        text(format!(
                            "The space current bid is {}",
                            total_burned.to_string_with_denomination(Denomination::Satoshi)
                        ))
                    } else {
                        text("This space doesn't exist. You can open it.")
                    },
                    text_input("amount", &state.bid_amount)
                        .on_input(Message::BidAmountInput)
                        .padding(10),
                ]
                .spacing(5),
            )
            .push(
                container(
                    button(if total_burned.is_none() {
                        "Open"
                    } else {
                        "Bid"
                    })
                    .on_press_maybe(
                        validate(&state.bid_amount)
                            .map(|_| Message::BidPress(slabel.clone(), total_burned.is_none())),
                    )
                    .padding([10, 20])
                    .width(Shrink),
                )
                .align_x(Center)
                .width(Fill),
            )
            .spacing(10)
    };

    let register_form = |slabel: SLabel| {
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
            .push(text("You can claim the space.").align_x(Center))
            .push(
                container(
                    button("Register")
                        .on_press(Message::RegisterPress(slabel))
                        .padding([10, 20])
                        .width(Shrink),
                )
                .align_x(Center)
                .width(Fill),
            )
            .spacing(10)
    };

    println!("{:?}", &space_data);

    let main: Element<'a, Message> = match space_data {
        None | Some((_, Some(Some(Covenant::Reserved)), _)) => {
            text("Enter a valid space name in the input above").into()
        }
        Some((_, None, _)) => text("Loading").into(),
        Some((slabel, Some(None), _)) => bid_form(slabel, None).into(),
        Some((
            slabel,
            Some(Some(Covenant::Bid {
                claim_height,
                total_burned,
                ..
            })),
            is_owned,
        )) => {
            if is_owned {
                if claim_height
                    .as_ref()
                    .map_or(false, |height| *height <= tip_height)
                {
                    register_form(slabel).into()
                } else {
                    text("Current highest bid is yours").into()
                }
            } else {
                bid_form(slabel, Some(total_burned)).into()
            }
        }
        Some((_, Some(Some(Covenant::Transfer { .. })), is_owned)) => {
            if is_owned {
                text("The space is registered by you.").into()
            } else {
                text("The space is already registered.").into()
            }
        }
    };

    column![
        container(
            text_input("space", space_name)
                .icon(text_input_icon(
                    Icon::At,
                    None,
                    10.0,
                    text_input::Side::Left
                ))
                .on_input(Message::SpaceNameInput)
                .font(Font::MONOSPACE)
                .padding(10)
        )
        .padding(20),
        center(main).padding(20),
    ]
    .spacing(10)
    .into()
}
