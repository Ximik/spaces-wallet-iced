use crate::store::{Amount, Covenant, Denomination, SLabel};

#[derive(Debug, Clone, Default)]
pub struct State {
    amount: String,
    fee_rate: String,
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
    AmountInput(String),
    FeeRateInput(String),
    BidPress(SLabel, bool),
    RegisterPress(SLabel),
}

#[derive(Debug, Clone)]
pub enum Task {
    None,
    SetSpace { space_name: String },
    OpenSpace { slabel: SLabel, amount: Amount },
    BidSpace { slabel: SLabel, amount: Amount },
    RegisterSpace { slabel: SLabel },
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
        Message::AmountInput(amount) => {
            if amount.chars().all(|c| c.is_digit(10)) {
                state.amount = amount
            }
            Task::None
        }
        Message::FeeRateInput(fee_rate) => {
            if fee_rate.chars().all(|c| c.is_digit(10) || c == '.') {
                state.fee_rate = fee_rate
            }
            Task::None
        }
        Message::BidPress(slabel, open) => {
            if let Some(amount) = validate(&state.amount) {
                Task::BidSpace { slabel, amount }
            } else {
                Task::None
            }
        }
        Message::RegisterPress(slabel) => Task::RegisterSpace { slabel },
    }
}

mod timeline {
    use crate::widget::rect::*;
    use iced::{
        widget::{text, Column, Row},
        Border, Center, Element, Fill, Theme,
    };

    const CIRCLE_RADIUS: f32 = 20.0;
    const LINE_WIDTH: f32 = 3.0;
    const LINE_HEIGHT: f32 = 50.0;
    const ROW_SPACING: f32 = 10.0;

    fn circle<'a>(filled: bool, border: bool, inner: bool) -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, CIRCLE_RADIUS * 2.0).style(move |theme: &Theme| {
            let palette = theme.palette();
            Style {
                border: Border {
                    color: if border {
                        palette.primary
                    } else {
                        palette.text
                    },
                    width: LINE_WIDTH,
                    radius: CIRCLE_RADIUS.into(),
                },
                background: if filled {
                    Some(palette.primary.into())
                } else {
                    None
                },
                inner: if inner {
                    Some(Inner {
                        border: Border {
                            radius: CIRCLE_RADIUS.into(),
                            ..Border::default()
                        },
                        background: Some(palette.primary.into()),
                        padding: (CIRCLE_RADIUS / 2.0).into(),
                    })
                } else {
                    None
                },
            }
        })
    }

    fn line<'a>(filled: bool) -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, LINE_HEIGHT).style(move |theme: &Theme| {
            let palette = theme.palette();
            Style {
                inner: Some(Inner {
                    background: Some(
                        if filled {
                            palette.primary
                        } else {
                            palette.text
                        }
                        .into(),
                    ),
                    padding: [0.0, CIRCLE_RADIUS - LINE_WIDTH / 2.0].into(),
                    ..Inner::default()
                }),
                ..Style::default()
            }
        })
    }

    fn space<'a>() -> Rect<'a> {
        Rect::new(CIRCLE_RADIUS * 2.0, LINE_HEIGHT)
    }

    pub fn view<'a, Message: 'a>(
        state: u8,
        label: impl text::IntoFragment<'a> + Clone,
    ) -> Element<'a, Message> {
        const LABELS: [&str; 4] = ["Open", "Pre-auction", "Auction", "Claim"];
        if state > LABELS.len() as u8 {
            panic!("state is out of range");
        }
        Column::from_iter((0..(LABELS.len() as u8) * 2).map(|i| {
            let c = i % 2 == 0;
            let n = i / 2;
            let o = n.cmp(&state);
            let row = Row::new()
                .push(if c {
                    circle(o.is_lt(), o.is_le(), o.is_eq())
                } else if n == LABELS.len() as u8 - 1 {
                    space()
                } else {
                    line(o.is_lt())
                })
                .push_maybe(if c {
                    Some(text(LABELS[n as usize]))
                } else if (state == LABELS.len() as u8 && state - n == 1) || o.is_eq() {
                    Some(text(label.clone()))
                } else {
                    None
                })
                .spacing(ROW_SPACING);
            if c { row.align_y(Center) } else { row }.into()
        }))
        .width(Fill)
        .into()
    }
}

use crate::{
    helper::height_to_est,
    widget::{
        form::Form,
        icon::{text_input_icon, Icon},
    },
};
use iced::{
    widget::{button, center, column, container, row, text, text_input, Space},
    Center, Element, Fill, Font, Shrink, Theme,
};
use wallet::bitcoin::absolute::Height;

fn open_view<'a>(slabel: SLabel) -> Element<'a, Message> {
    row![
        timeline::view(0, "Make an open to propose the space for auction"),
        Form::new("Open", None).add_labeled_input(
            "Amount",
            "amount in sat",
            "100",
            Message::AmountInput
        )
    ]
    .into()
}

fn bid_view<'a>(
    slabel: SLabel,
    tip_height: u32,
    claim_height: Option<u32>,
    current_bid: Amount,
    is_owned: bool,
) -> Element<'a, Message> {
    row![
        timeline::view(
            if claim_height.is_none() { 1 } else { 2 },
            claim_height.map_or(
                "Make a bid to improve the chance of moving the space to auction".to_string(),
                |height| format!("Auction ends {}", height_to_est(height, tip_height))
            )
        ),
        column![
            text(format!("Current bid: {} sat", current_bid.to_sat())),
            text(if is_owned {
                "It is yours bid"
            } else {
                "It is not yours bid"
            }),
            Form::new("Bid", None).add_labeled_input(
                "Amount",
                "amount in sat",
                "100",
                Message::AmountInput
            )
        ]
    ]
    .into()
}

fn claim_view<'a>(slabel: SLabel, current_bid: Amount, is_owned: bool) -> Element<'a, Message> {
    row![
        timeline::view(
            3,
            if is_owned {
                "You can claim the space"
            } else {
                "The auction is ended, but you still can outbid"
            }
        ),
        if is_owned {
            column![Form::new("Claim", None)]
        } else {
            column![
                text(format!("Current bid: {} sat", current_bid.to_sat())),
                Form::new("Bid", None).add_labeled_input(
                    "Amount",
                    "amount in sat",
                    "100",
                    Message::AmountInput
                ),
            ]
        }
    ]
    .into()
}

fn registered_view<'a>(
    slabel: SLabel,
    tip_height: u32,
    expire_height: u32,
    is_owned: bool,
) -> Element<'a, Message> {
    row![
        timeline::view(
            4,
            format!(
                "The space registration expires {}",
                height_to_est(expire_height, tip_height)
            )
        ),
        if is_owned {
            column![Form::new("Send", None)]
        } else {
            column![Space::new(Fill, Fill)]
        }
    ]
    .into()
}

pub fn view<'a>(
    state: &'a State,
    tip_height: u32,
    space_name: &'a String,
    space_data: Option<(SLabel, Option<&'a Option<Covenant>>, bool)>,
) -> Element<'a, Message> {
    let main: Element<'a, Message> = match space_data {
        None | Some((_, Some(Some(Covenant::Reserved)), _)) => {
            text("Enter a valid space name in the input above").into()
        }
        Some((_, None, _)) => Space::new(Fill, Fill).into(),
        Some((slabel, Some(None), _)) => open_view(slabel),
        Some((
            slabel,
            Some(Some(Covenant::Bid {
                claim_height,
                total_burned,
                ..
            })),
            is_owned,
        )) => {
            if claim_height.map_or(false, |height| height <= tip_height) {
                claim_view(slabel, *total_burned, is_owned)
            } else {
                bid_view(slabel, tip_height, *claim_height, *total_burned, is_owned)
            }
        }
        Some((slabel, Some(Some(Covenant::Transfer { expire_height, .. })), is_owned)) => {
            registered_view(slabel, tip_height, *expire_height, is_owned)
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
