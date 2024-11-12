use iced::widget::{center, column, text, text_input};
use iced::{Element, Task};

use crate::app::{Message, RpcRequest, Screen};
use crate::store::Store;

pub fn update(space_name: &String) -> (bool, Task<Message>) {
    let valid = space_name
        .chars()
        .all(|f| f.is_ascii_digit() || f.is_ascii_lowercase());
    (
        valid,
        if valid && !&space_name.is_empty() {
            Task::done(Message::InvokeRpc(RpcRequest::GetSpaceInfo {
                space: space_name.clone(),
            }))
        } else {
            Task::none()
        },
    )
}

pub fn view<'a>(store: &'a Store, space_name: &'a String) -> Element<'a, Message> {
    let wallet = store.wallet.as_ref().unwrap();
    let in_wallet = wallet.space_names.contains(space_name);
    column![
        text_input("space", space_name)
            .on_input(|space_name| Message::UpdateScreen(Screen::Space { space_name }))
            .padding(10),
        center(match store.spaces.get(space_name) {
            None => text("Loading"),
            Some(None) => text("No such space"),
            Some(Some(_)) => text(if in_wallet { "mine " } else { "not mine" }),
        }),
    ]
    .spacing(10)
    .into()
}
