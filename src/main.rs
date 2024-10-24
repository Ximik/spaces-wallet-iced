use std::sync::Arc;

use iced::widget::{button, center, column, container, qr_code, row, text, Column};
use iced::{clipboard, Center, Element, Fill, Task};

use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use spaced::{
    config::{default_spaces_rpc_port, ExtendedNetwork},
    rpc::RpcClient,
};

mod types;
use types::*;

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .window_size((800.0, 500.0))
        .run_with(App::new)
}

#[derive(Debug, Clone, Copy)]
enum Screen {
    Home,
    Receive(AddressKind),
}

#[derive(Debug, Clone)]
enum Message {
    ClipboardWrite(String),
    ScreenSet(Screen),
    WalletLoad(String),
    WalletLoaded(Result<String, String>),
    AddressLoad(AddressKind),
    AddressLoaded(Result<(AddressKind, String), String>),
}

#[derive(Debug)]
struct App {
    client: Arc<HttpClient>,
    wallet: Option<Wallet>,
    screen: Screen,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let spaced_rpc_url = format!(
            "http://127.0.0.1:{}",
            default_spaces_rpc_port(&ExtendedNetwork::Testnet4)
        );
        let client = Arc::new(HttpClientBuilder::default().build(spaced_rpc_url).unwrap());
        (
            Self {
                client,
                wallet: None,
                screen: Screen::Home,
            },
            Task::done(Message::WalletLoad("default".into())),
        )
    }

    fn title(&self) -> String {
        "Spaces Wallet".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ClipboardWrite(string) => clipboard::write(string),
            Message::ScreenSet(screen) => {
                self.screen = screen;
                match screen {
                    Screen::Receive(address_kind) => Task::done(Message::AddressLoad(address_kind)),
                    _ => Task::none(),
                }
            }
            Message::WalletLoad(wallet) => {
                let client = self.client.clone();
                Task::perform(
                    async move {
                        match client.wallet_load(&wallet).await {
                            Ok(_) => Ok(wallet),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::WalletLoaded,
                )
            }
            Message::WalletLoaded(Err(e)) => {
                panic!("{}", e)
            }
            Message::WalletLoaded(Ok(wallet_name)) => {
                self.wallet = Some(Wallet::new(wallet_name));
                Task::none()
            }
            Message::AddressLoad(address_kind) => {
                if let Some(wallet) = self.wallet.as_ref() {
                    let client = self.client.clone();
                    let wallet_name = wallet.get_name();
                    Task::perform(
                        async move {
                            match client
                                .wallet_get_new_address(&wallet_name, address_kind)
                                .await
                            {
                                Ok(address) => Ok((address_kind, address)),
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        Message::AddressLoaded,
                    )
                } else {
                    Task::none()
                }
            }
            Message::AddressLoaded(Err(e)) => {
                eprintln!("{}", e);
                Task::none()
            }
            Message::AddressLoaded(Ok((address_kind, address))) => {
                if let Some(wallet) = self.wallet.as_mut() {
                    wallet.set_address(address_kind, address);
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if let Some(wallet) = self.wallet.as_ref() {
            row![
                navbar(self.screen),
                match self.screen {
                    Screen::Home => center(text("HOME")),
                    Screen::Receive(address_kind) => container(receive_page(address_kind, wallet)),
                }
            ]
            .into()
        } else {
            center(text("LOADING").align_x(Center)).into()
        }
    }
}

fn navbar<'a>(current_screen: Screen) -> Element<'a, Message> {
    let navbar_button = |label, is_active, screen| {
        let label = text(label);
        let button = button(label)
            .style(if is_active {
                button::primary
            } else {
                button::text
            })
            .padding(10)
            .width(Fill);
        button.on_press(Message::ScreenSet(screen))
    };

    column![
        navbar_button("Home", matches!(current_screen, Screen::Home), Screen::Home),
        navbar_button(
            "Receive",
            matches!(current_screen, Screen::Receive(_)),
            Screen::Receive(AddressKind::Coin)
        ),
    ]
    .width(200)
    .into()
}

fn receive_page<'a>(current_address_kind: AddressKind, wallet: &'a Wallet) -> Element<'a, Message> {
    let tab_button = |label, current_address_kind, address_kind| {
        let label = text(label);
        let button = button(label)
            // FIXME
            .style(if current_address_kind as u8 == address_kind as u8 {
                button::primary
            } else {
                button::text
            })
            .padding(10)
            .width(Fill);
        button.on_press(Message::ScreenSet(Screen::Receive(address_kind)))
    };

    let address = wallet.get_address(current_address_kind);

    Column::new()
        .push(row![
            tab_button("Coins", current_address_kind, AddressKind::Coin),
            tab_button("Spaces", current_address_kind, AddressKind::Space),
        ])
        .push_maybe(address.map(|address| {
            center(column![
                center(qr_code(&address.qr_code).cell_size(5)),
                text(&address.text),
                button("Copy").on_press(Message::ClipboardWrite(address.text.clone())),
            ])
        }))
        .into()
}
