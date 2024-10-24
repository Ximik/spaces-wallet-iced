use iced::widget::{button, center, column, qr_code, row, text};
use iced::{Element, Fill};
use spaced::wallets::AddressKind;

#[derive(Debug, Clone)]
pub enum Event {
    Loading(AddressKind),
    Loaded(AddressKind, String),
}

#[derive(Debug, Clone)]
pub enum Message {
    AddressLoad(AddressKind),
    ClipboardWrite(String),
}

#[derive(Debug)]
pub struct Component {
    kind: AddressKind,
    address: Option<(String, qr_code::Data)>,
}

impl Component {
    pub fn new() -> Self {
        Self {
            kind: AddressKind::Coin,
            address: None,
        }
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Loading(kind) => {
                self.kind = kind;
                self.address = None;
            }
            Event::Loaded(kind, address) => {
                self.kind = kind;
                self.address = qr_code::Data::new(&address)
                    .ok()
                    .and_then(|data| Some((address, data)));
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let tab_button = |label: String, current_kind: AddressKind, kind: AddressKind| {
            let label = text(label);
            let button = button(label)
                // FIXME
                .style(if current_kind as u8 == kind as u8 {
                    button::primary
                } else {
                    button::text
                })
                .padding(10)
                .width(Fill);
            button.on_press(Message::AddressLoad(kind))
        };

        

        column![row![
            tab_button("Coins".to_string(), self.kind, AddressKind::Coin),
            tab_button("Spaces".to_string(), self.kind, AddressKind::Space),
        ]]
        .push_maybe(self.address.as_ref().map(|address| {
            center(column![
                center(qr_code(&address.1).cell_size(5)),
                text(address.0.to_string()),
                button("Copy").on_press(Message::ClipboardWrite(address.0.to_string())),
            ])
        }))
        .into()
    }
}
