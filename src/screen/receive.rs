use iced::widget::{button, center, column, container, qr_code, text, toggler, Column};
use iced::{Center, Element, Fill, Task};

use crate::app::{Message, RpcRequest};
use crate::store::Store;

pub fn update(legacy_address: bool) -> (bool, Task<Message>) {
    (
        true,
        Task::done(Message::InvokeRpc(RpcRequest::GetAddress {
            legacy: legacy_address,
        })),
    )
}

pub fn view<'a>(store: &'a Store, legacy_address: bool) -> Element<'a, Message> {
    let wallet = store.wallet.as_ref().unwrap();
    let address = if legacy_address {
        wallet.legacy_address.as_ref()
    } else {
        wallet.address.as_ref()
    };
    Column::new()
        .push(
            container(
                toggler(legacy_address)
                    .label("SegWit v0 address")
                    .on_toggle(move |_| {
                        Message::UpdateScreen(crate::app::Screen::Receive {
                            legacy_address: !legacy_address,
                        })
                    }),
            )
            .align_x(Center)
            .width(Fill),
        )
        .push_maybe(address.map(|address| {
            center(
                column![
                    text(&address.text).size(14),
                    qr_code(&address.qr_code).cell_size(7),
                    button("Copy")
                        .padding([10, 20])
                        .on_press(Message::WriteClipboard(address.text.clone())),
                ]
                .align_x(Center)
                .spacing(10),
            )
        }))
        .into()
}
