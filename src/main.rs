use std::sync::Arc;

use iced::widget::{button, center, column, container, row, text};
use iced::{clipboard, Center, Element, Fill, Task};

use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use spaced::{
    config::{default_spaces_rpc_port, ExtendedNetwork},
    rpc::RpcClient,
};

mod types;
use types::*;

mod screen;

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .window_size((800.0, 500.0))
        .run_with(App::new)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Home,
    Receive,
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
    home_screen: screen::home::Component,
    receive_screen: screen::receive::Component,
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
                home_screen: screen::home::Component::new(),
                receive_screen: screen::receive::Component::new(),
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
                    Screen::Home => text("HOME").into(),
                    Screen::Receive => self.receive_screen.view().map(|msg| {
                        match msg {
                            screen::receive::Message::AddressLoad(kind) => {
                                Message::AddressLoad(kind)
                            }
                            screen::receive::Message::ClipboardWrite(text) => {
                                Message::ClipboardWrite(text)
                            }
                        }
                    }),
                }
            ]
            .into()
        } else {
            center(text("LOADING").align_x(Center)).into()
        }
    }
}

fn navbar<'a>(current_screen: Screen) -> Element<'a, Message> {
    let navbar_button = |label, current_screen, screen| {
        let label = text(label);
        let button = button(label)
            .style(if current_screen == screen {
                button::primary
            } else {
                button::text
            })
            .padding(10)
            .width(Fill);
        button.on_press(Message::ScreenSet(screen))
    };

    column![
        navbar_button("Home", current_screen, Screen::Home),
        navbar_button("Receive", current_screen, Screen::Receive),
    ]
    .width(200)
    .into()
}
