use iced::widget::{button, column, scrollable, text, Column};
use iced::{Element, Fill};

use crate::store::{Amount, Covenant};

#[derive(Debug, Clone)]
pub enum Message {
    SpaceClicked { space_name: String },
}

pub fn view<'a>(
    balance: Amount,
    spaces: impl Iterator<Item = (&'a String, &'a Option<Covenant>)>,
) -> Element<'a, Message> {
    column![
        text("Balance (BTC)"),
        text(balance.to_btc()),
        text("Your spaces"),
        scrollable(Column::with_children(spaces.map(|(space_name, _)| {
            button(text(space_name))
                .on_press(Message::SpaceClicked {
                    space_name: space_name.clone(),
                })
                .width(Fill)
                .padding([10, 20])
                .into()
        })))
    ]
    .padding(10)
    .height(Fill)
    .into()
}
