use iced::widget::{button, column, scrollable, text, Column};
use iced::{Element, Fill, Task};

use crate::app::{Message, RpcRequest, Screen};
use crate::store::Store;

pub fn update() -> (bool, Task<Message>) {
    (
        true,
        Task::batch([
            Task::done(Message::InvokeRpc(RpcRequest::GetBalance)),
            Task::done(Message::InvokeRpc(RpcRequest::GetWalletSpaces)),
        ]),
    )
}

pub fn view<'a>(store: &'a Store) -> Element<'a, Message> {
    let wallet = store.wallet.as_ref().unwrap();
    column![
        text("Balance (BTC)"),
        text(wallet.balance.to_btc()),
        text("Your spaces"),
        scrollable(Column::with_children(wallet.space_names.iter().map(
            |space_name| {
                button(text(space_name.clone()))
                    .on_press(Message::UpdateScreen(Screen::Space {
                        space_name: space_name.to_string(),
                    }))
                    .width(Fill)
                    .padding([10, 20])
                    .into()
            }
        )))
    ]
    .padding(10)
    .height(Fill)
    .into()
}
