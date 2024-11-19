use iced::widget::{button, center, row, scrollable, text, Column};
use iced::{Element, Fill};

use crate::icon;
use crate::store::TxInfo;

#[derive(Debug, Clone)]
pub enum Message {
    TxidCopyPress { txid: String },
}

pub fn view<'a>(transactions: &'a [TxInfo]) -> Element<'a, Message> {
    if transactions.is_empty() {
        center(text("No transactions yet")).into()
    } else {
        scrollable(Column::with_children(transactions.into_iter().map(
            |transaction| {
                let txid = transaction.txid.to_string();
                row![
                    text(txid.clone()),
                    button(text(icon::COPY).font(icon::FONT))
                        .on_press(Message::TxidCopyPress { txid })
                ]
                .width(Fill)
                .into()
            },
        )))
        .into()
    }
}
