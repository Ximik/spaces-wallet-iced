use std::fmt;
use std::sync::Arc;

use iced::time;
use iced::widget::{button, center, column, container, row, text, Column};
use iced::{clipboard, Center, Element, Fill, Subscription, Task, Theme};

use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use spaced::{
    config::default_spaces_rpc_port,
    rpc::{RpcClient, RpcWalletRequest, RpcWalletTxBuilder, SendCoinsParams, ServerInfo},
};

use crate::screen;
use crate::store::*;

#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Send {
        address: String,
        amount: String,
        error: Option<String>,
    },
    Receive {
        legacy_address: bool,
    },
    Space {
        space_name: String,
    },
    Transactions,
}

#[derive(Debug, Clone)]
enum RpcError {
    Call { code: i32, message: String },
    Global { message: String },
}
impl From<ClientError> for RpcError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Call(e) => RpcError::Call {
                code: e.code(),
                message: e.message().to_string(),
            },
            _ => RpcError::Global {
                message: error.to_string(),
            },
        }
    }
}
impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RpcError::Call { message, .. } => message,
                RpcError::Global { message } => message,
            }
        )
    }
}
type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug, Clone)]
pub enum RpcRequest {
    GetServerInfo,
    GetSpaceInfo { space: String },
    LoadWallet { wallet: String },
    GetBalance,
    GetWalletSpaces,
    GetAddress { legacy: bool },
    SendCoins { address: String, amount: Amount },
}

#[derive(Debug, Clone)]
enum RpcResponse {
    GetServerInfo {
        result: RpcResult<ServerInfo>,
    },
    GetSpaceInfo {
        space: String,
        result: RpcResult<Option<FullSpaceOut>>,
    },
    LoadWallet {
        wallet: String,
        result: RpcResult<()>,
    },
    GetBalance {
        wallet: String,
        result: RpcResult<Balance>,
    },
    GetWalletSpaces {
        wallet: String,
        result: RpcResult<Vec<WalletOutput>>,
    },
    GetAddress {
        wallet: String,
        legacy: bool,
        result: RpcResult<String>,
    },
    SendCoins {
        result: RpcResult<()>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    WriteClipboard(String),
    UpdateScreen(Screen),
    InvokeRpc(RpcRequest),
    #[allow(private_interfaces)]
    HandleRpc(RpcResponse),
}

pub struct App {
    rpc_client: Arc<HttpClient>,
    rpc_error: Option<String>,
    screen: Screen,
    store: Store,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let spaced_rpc_url = format!(
            "http://127.0.0.1:{}",
            default_spaces_rpc_port(&ExtendedNetwork::Testnet4)
        );
        let rpc_client: Arc<HttpClient> =
            Arc::new(HttpClientBuilder::default().build(spaced_rpc_url).unwrap());
        (
            Self {
                rpc_client,
                rpc_error: None,
                screen: Screen::Home,
                store: Default::default(),
            },
            Task::done(Message::InvokeRpc(RpcRequest::LoadWallet {
                wallet: "default".into(),
            })),
        )
    }

    pub fn title(&self) -> String {
        "Spaces Wallet".into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WriteClipboard(string) => clipboard::write(string),
            Message::UpdateScreen(screen) => {
                let (valide, task) = match &screen {
                    Screen::Home => screen::home::update(),
                    Screen::Send {
                        address, amount, ..
                    } => screen::send::update(address, amount),
                    Screen::Receive { legacy_address } => screen::receive::update(*legacy_address),
                    Screen::Space { space_name } => screen::space::update(&space_name),
                    Screen::Transactions => screen::transactions::update(),
                };
                if valide {
                    self.screen = screen;
                }
                task
            }
            Message::InvokeRpc(request) => {
                let client = self.rpc_client.clone();
                match request {
                    RpcRequest::GetServerInfo => Task::perform(
                        async move {
                            let result = client.get_server_info().await.map_err(RpcError::from);
                            RpcResponse::GetServerInfo { result }
                        },
                        Message::HandleRpc,
                    ),
                    RpcRequest::GetSpaceInfo { space } => Task::perform(
                        async move {
                            use protocol::{
                                hasher::{KeyHasher, SpaceKey},
                                sname::{NameLike, SName},
                            };
                            use spaced::store::Sha256;
                            use std::str::FromStr;

                            let mut name = String::from("@");
                            name.push_str(&space);
                            let sname = SName::from_str(&name).unwrap();
                            let spacehash = SpaceKey::from(Sha256::hash(sname.to_bytes()));
                            let spacehash = hex::encode(spacehash.as_slice());
                            let result = client.get_space(&spacehash).await.map_err(RpcError::from);
                            RpcResponse::GetSpaceInfo { space, result }
                        },
                        Message::HandleRpc,
                    ),
                    RpcRequest::LoadWallet { wallet } => Task::perform(
                        async move {
                            let result = client.wallet_load(&wallet).await.map_err(RpcError::from);
                            RpcResponse::LoadWallet { wallet, result }
                        },
                        Message::HandleRpc,
                    ),
                    RpcRequest::GetBalance => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_get_balance(&wallet)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetBalance { wallet, result }
                                },
                                Message::HandleRpc,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::GetWalletSpaces => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_list_spaces(&wallet)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetWalletSpaces { wallet, result }
                                },
                                Message::HandleRpc,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::GetAddress { legacy } => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_get_new_address(
                                            &wallet,
                                            if legacy {
                                                AddressKind::Coin
                                            } else {
                                                AddressKind::Space
                                            },
                                        )
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetAddress {
                                        wallet,
                                        legacy,
                                        result,
                                    }
                                },
                                Message::HandleRpc,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::SendCoins { address, amount } => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_send_request(
                                            &wallet,
                                            RpcWalletTxBuilder {
                                                auction_outputs: None,
                                                requests: vec![RpcWalletRequest::SendCoins(
                                                    SendCoinsParams {
                                                        amount,
                                                        to: address,
                                                    },
                                                )],
                                                fee_rate: None,
                                                dust: None,
                                                force: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::SendCoins { result }
                                },
                                Message::HandleRpc,
                            )
                        } else {
                            Task::none()
                        }
                    }
                }
            }
            Message::HandleRpc(response) => {
                self.rpc_error = None;
                match response {
                    RpcResponse::GetServerInfo { result } => {
                        match result {
                            Ok(server_info) => {
                                self.store.tip_height = server_info.tip.height;
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::GetSpaceInfo { space, result } => {
                        match result {
                            Ok(out) => {
                                self.store.spaces.insert(
                                    space,
                                    out.map(|out| out.spaceout.space.unwrap().covenant),
                                );
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::LoadWallet { wallet, result } => match result {
                        Ok(_) => {
                            self.store.wallet = Some(Wallet::new(wallet));
                            Task::done(Message::UpdateScreen(Screen::Home))
                        }
                        Err(e) => {
                            self.rpc_error = Some(e.to_string());
                            Task::none()
                        }
                    },
                    RpcResponse::GetBalance { wallet, result } => {
                        match result {
                            Ok(balance) => {
                                if let Some(wallet) = self.store.get_wallet_with_name(&wallet) {
                                    wallet.balance = balance.balance;
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::GetWalletSpaces { wallet, result } => {
                        match result {
                            Ok(spaces) => {
                                let space_names: Vec<_> = spaces
                                    .into_iter()
                                    .map(|out| {
                                        let space = out.space.unwrap();
                                        let space_name = space.name.to_string()[1..].to_string();
                                        self.store
                                            .spaces
                                            .insert(space_name.clone(), Some(space.covenant));
                                        space_name
                                    })
                                    .collect();
                                if let Some(wallet) = self.store.get_wallet_with_name(&wallet) {
                                    wallet.space_names = space_names;
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::GetAddress {
                        wallet,
                        legacy,
                        result,
                    } => {
                        match result {
                            Ok(address) => {
                                if let Some(wallet) = self.store.get_wallet_with_name(&wallet) {
                                    let address = Address::new(address);
                                    if legacy {
                                        wallet.legacy_address = Some(address);
                                    } else {
                                        wallet.address = Some(address);
                                    }
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::SendCoins { result } => {
                        match result {
                            Ok(_) => Task::done(Message::UpdateScreen(Screen::Transactions)),
                            Err(RpcError::Call { code, message }) => {
                                if code == -1 {
                                    match &self.screen {
                                        Screen::Send {
                                            address, amount, ..
                                        } => {
                                            self.screen = Screen::Send {
                                                address: address.clone(),
                                                amount: amount.clone(),
                                                error: Some(message),
                                            }
                                        }
                                        _ => {}
                                    };
                                } else {
                                    self.rpc_error = Some(message);
                                }
                                Task::none()
                            }
                            Err(e) => {
                                // TODO: show method errors
                                self.rpc_error = Some(e.to_string());
                                Task::none()
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let main: Element<Message> = if self.store.wallet.is_some() {
            row![
                navbar(&self.screen),
                container(match &self.screen {
                    Screen::Home => screen::home::view(&self.store),
                    Screen::Send {
                        address,
                        amount,
                        error,
                    } => screen::send::view(address, amount, error),
                    Screen::Receive { legacy_address } =>
                        screen::receive::view(&self.store, *legacy_address),
                    Screen::Space { space_name } => screen::space::view(&self.store, space_name),
                    Screen::Transactions => screen::transactions::view(),
                })
                .style(|theme: &Theme| {
                    container::Style::default()
                        .background(theme.extended_palette().background.weak.color)
                })
            ]
            .into()
        } else {
            center(text("LOADING").align_x(Center)).into()
        };
        Column::new()
            .push_maybe(self.rpc_error.as_ref().map(errorbar))
            .push(main)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.store.wallet.is_some() && self.rpc_error.is_none() {
            time::every(time::Duration::from_secs(5))
                .map(|_| Message::InvokeRpc(RpcRequest::GetServerInfo))
        } else {
            Subscription::none()
        }
    }
}

fn errorbar<'a>(error: &'a String) -> Element<'a, Message> {
    container(
        text(error)
            .style(|theme: &Theme| text::Style {
                color: Some(theme.extended_palette().danger.base.text),
            })
            .center()
            .width(Fill),
    )
    .style(|theme: &Theme| {
        container::Style::default().background(theme.extended_palette().danger.base.color)
    })
    .width(Fill)
    .padding(10)
    .into()
}

fn navbar<'a>(current_screen: &'a Screen) -> Element<'a, Message> {
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
        button.on_press(Message::UpdateScreen(screen))
    };

    container(column![
        navbar_button("Home", matches!(current_screen, Screen::Home), Screen::Home),
        navbar_button(
            "Send",
            matches!(current_screen, Screen::Send { .. }),
            Screen::Send {
                address: String::new(),
                amount: String::new(),
                error: None,
            }
        ),
        navbar_button(
            "Receive",
            matches!(current_screen, Screen::Receive { .. }),
            Screen::Receive {
                legacy_address: false
            }
        ),
        navbar_button(
            "Spaces",
            matches!(current_screen, Screen::Space { .. }),
            Screen::Space {
                space_name: String::new()
            }
        ),
        navbar_button(
            "Transactions",
            matches!(current_screen, Screen::Transactions),
            Screen::Transactions
        ),
    ])
    .width(200)
    .into()
}
