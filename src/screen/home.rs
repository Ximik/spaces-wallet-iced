use iced::widget::{button, column, scrollable, text, Column};
use iced::{Element, Fill};

use crate::icon;
use crate::store::{Amount, Covenant, SLabel};

#[derive(Debug, Clone)]
pub enum Message {
    SpaceClicked { space_name: String },
}

pub fn view<'a>(
    balance: Amount,
    spaces: impl Iterator<Item = (&'a SLabel, &'a Covenant)>,
) -> Element<'a, Message> {
    column![
        text("Balance (SAT)"),
        text(balance.to_sat()),
        text("Your spaces"),
        scrollable(Column::with_children(spaces.map(|(slabel, _)| {
            button(text(slabel.to_string()))
                .on_press(Message::SpaceClicked {
                    space_name: slabel.to_string()[1..].to_string(),
                })
                .width(Fill)
                .padding([10, 20])
                .into()
        })))
    ]
    .spacing(5)
    .padding(10)
    .height(Fill)
    .width(Fill)
    .into()
}
