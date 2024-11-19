use std::fmt;
use std::sync::Arc;

use crate::icon;
use iced::time;
use iced::widget::{button, center, column, container, row, text, Column};
use iced::{clipboard, Center, Element, Fill, Subscription, Task, Theme};

use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use spaced::{
    config::default_spaces_rpc_port,
    rpc::{
        BidParams, OpenParams, RpcClient, RpcWalletRequest, RpcWalletTxBuilder, SendCoinsParams,
        ServerInfo,
    },
};

use crate::screen;
use crate::store::*;

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
enum RpcRequest {
    GetServerInfo,
    GetSpaceInfo {
        space: String,
    },
    LoadWallet {
        wallet: String,
    },
    GetBalance,
    GetWalletSpaces,
    GetTransactions,
    GetAddress {
        address_kind: AddressKind,
    },
    SendCoins {
        recipient: String,
        amount: Amount,
    },
    BidSpace {
        space_name: String,
        bid_amount: Amount,
    },
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
    GetTransactions {
        wallet: String,
        result: RpcResult<Vec<TxInfo>>,
    },
    GetWalletSpaces {
        wallet: String,
        result: RpcResult<Vec<WalletOutput>>,
    },
    GetAddress {
        wallet: String,
        address_kind: AddressKind,
        result: RpcResult<String>,
    },
    SendCoins {
        result: RpcResult<()>,
    },
    BidSpace {
        result: RpcResult<()>,
    },
}

#[derive(Debug, Clone)]
enum Screen {
    Home,
    Send,
    Receive,
    Space(String),
    Transactions,
}

#[derive(Debug, Clone)]
enum Message {
    RpcRequest(RpcRequest),
    RpcResponse(RpcResponse),
    SetScreen(Screen),
    ScreenHome(screen::home::Message),
    ScreenSend(screen::send::Message),
    ScreenReceive(screen::receive::Message),
    ScreenSpace(screen::space::Message),
    ScreenTransactions(screen::transactions::Message),
}

pub struct App {
    rpc_client: Arc<HttpClient>,
    rpc_error: Option<String>,
    store: Store,
    screen: Screen,
    screen_send: screen::send::State,
    screen_receive: screen::receive::State,
    screen_space: screen::space::State,
}

impl App {
    pub fn run() -> iced::Result {
        let icon =
            iced::window::icon::from_rgba(include_bytes!("../assets/spaces.rgba").to_vec(), 64, 64)
                .expect("Failed to load icon");
        let icons_font = include_bytes!("../assets/icons.ttf").as_slice();
        iced::application(Self::title, Self::update, Self::view)
            .font(icons_font)
            .subscription(Self::subscription)
            .window(iced::window::Settings {
                size: (900.0, 500.0).into(),
                icon: Some(icon),
                ..Default::default()
            })
            .run_with(Self::new)
    }

    fn new() -> (Self, Task<Message>) {
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
                store: Default::default(),
                screen: Screen::Home,
                screen_send: Default::default(),
                screen_receive: Default::default(),
                screen_space: Default::default(),
            },
            Task::done(Message::RpcRequest(RpcRequest::LoadWallet {
                wallet: "default".into(),
            })),
        )
    }

    fn title(&self) -> String {
        "Spaces Wallet".into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RpcRequest(request) => {
                let client = self.rpc_client.clone();
                match request {
                    RpcRequest::GetServerInfo => Task::perform(
                        async move {
                            let result = client.get_server_info().await.map_err(RpcError::from);
                            RpcResponse::GetServerInfo { result }
                        },
                        Message::RpcResponse,
                    ),
                    RpcRequest::GetSpaceInfo { space } => Task::perform(
                        async move {
                            use protocol::{hasher::KeyHasher, slabel::SLabel};
                            use spaced::store::Sha256;
                            use std::str::FromStr;

                            let mut name = String::from("@");
                            name.push_str(&space);
                            let sname = SLabel::from_str(&name).unwrap();
                            let spacehash = hex::encode(Sha256::hash(sname.as_ref()));
                            let result = client.get_space(&spacehash).await.map_err(RpcError::from);
                            RpcResponse::GetSpaceInfo { space, result }
                        },
                        Message::RpcResponse,
                    ),
                    RpcRequest::LoadWallet { wallet } => Task::perform(
                        async move {
                            let result = client.wallet_load(&wallet).await.map_err(RpcError::from);
                            RpcResponse::LoadWallet { wallet, result }
                        },
                        Message::RpcResponse,
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
                                Message::RpcResponse,
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
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::GetTransactions => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_list_transactions(&wallet, 100, 0)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetTransactions { wallet, result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::GetAddress { address_kind } => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_get_new_address(&wallet, address_kind)
                                        .await
                                        .map_err(RpcError::from);
                                    RpcResponse::GetAddress {
                                        wallet,
                                        address_kind,
                                        result,
                                    }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::SendCoins { recipient, amount } => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_send_request(
                                            &wallet,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![RpcWalletRequest::SendCoins(
                                                    SendCoinsParams {
                                                        amount,
                                                        to: recipient,
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
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                    RpcRequest::BidSpace {
                        space_name,
                        bid_amount,
                    } => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            let name = format!("@{}", space_name);
                            let amount = bid_amount.to_sat();
                            let is_new = self
                                .store
                                .spaces
                                .get(&space_name)
                                .map_or(true, |inner| inner.is_none());
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_send_request(
                                            &wallet,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![if is_new {
                                                    RpcWalletRequest::Open(OpenParams {
                                                        name,
                                                        amount,
                                                    })
                                                } else {
                                                    RpcWalletRequest::Bid(BidParams {
                                                        name,
                                                        amount,
                                                    })
                                                }],
                                                fee_rate: None,
                                                dust: None,
                                                force: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::BidSpace { result }
                                },
                                Message::RpcResponse,
                            )
                        } else {
                            Task::none()
                        }
                    }
                }
            }
            Message::RpcResponse(response) => {
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
                            Task::done(Message::SetScreen(Screen::Home))
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
                    RpcResponse::GetTransactions { wallet, result } => {
                        match result {
                            Ok(transactions) => {
                                if let Some(wallet) = self.store.get_wallet_with_name(&wallet) {
                                    wallet.transactions = transactions;
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
                        address_kind,
                        result,
                    } => {
                        match result {
                            Ok(address) => {
                                if let Some(wallet) = self.store.get_wallet_with_name(&wallet) {
                                    let address = Address::new(address);
                                    match address_kind {
                                        AddressKind::Coin => wallet.coin_address = Some(address),
                                        AddressKind::Space => wallet.space_address = Some(address),
                                    }
                                }
                            }
                            Err(e) => {
                                self.rpc_error = Some(e.to_string());
                            }
                        }
                        Task::none()
                    }
                    RpcResponse::SendCoins { result } => match result {
                        Ok(_) => Task::done(Message::SetScreen(Screen::Transactions)),
                        Err(RpcError::Call { code, message }) => {
                            if code == -1 {
                                self.screen_send.set_error(message);
                            } else {
                                self.rpc_error = Some(message);
                            }
                            Task::none()
                        }
                        Err(e) => {
                            self.rpc_error = Some(e.to_string());
                            Task::none()
                        }
                    },
                    RpcResponse::BidSpace { result } => match result {
                        Ok(_) => Task::done(Message::SetScreen(Screen::Transactions)),
                        Err(RpcError::Call { code, message }) => {
                            if code == -1 {
                                self.screen_space.set_error(message);
                            } else {
                                self.rpc_error = Some(message);
                            }
                            Task::none()
                        }
                        Err(e) => {
                            self.rpc_error = Some(e.to_string());
                            Task::none()
                        }
                    },
                }
            }
            Message::SetScreen(screen) => {
                self.screen = screen;
                match self.screen {
                    Screen::Home => Task::batch([
                        Task::done(Message::RpcRequest(RpcRequest::GetBalance)),
                        Task::done(Message::RpcRequest(RpcRequest::GetWalletSpaces)),
                    ]),
                    Screen::Send => Task::none(),
                    Screen::Receive => Task::batch([
                        Task::done(Message::RpcRequest(RpcRequest::GetAddress {
                            address_kind: AddressKind::Coin,
                        })),
                        Task::done(Message::RpcRequest(RpcRequest::GetAddress {
                            address_kind: AddressKind::Space,
                        })),
                    ]),
                    Screen::Space(ref space_name) => {
                        if !space_name.is_empty() {
                            Task::done(Message::RpcRequest(RpcRequest::GetSpaceInfo {
                                space: space_name.clone(),
                            }))
                        } else {
                            Task::none()
                        }
                    }
                    Screen::Transactions => {
                        Task::done(Message::RpcRequest(RpcRequest::GetTransactions))
                    }
                }
            }
            Message::ScreenHome(message) => match message {
                screen::home::Message::SpaceClicked { space_name } => {
                    Task::done(Message::SetScreen(Screen::Space(space_name)))
                }
            },
            Message::ScreenSend(message) => {
                match screen::send::update(&mut self.screen_send, message) {
                    screen::send::Task::SendCoins { recipient, amount } => {
                        Task::done(Message::RpcRequest(RpcRequest::SendCoins {
                            recipient,
                            amount,
                        }))
                    }
                    screen::send::Task::None => Task::none(),
                }
            }
            Message::ScreenReceive(message) => {
                match screen::receive::update(&mut self.screen_receive, message) {
                    screen::receive::Task::WriteClipboard(s) => clipboard::write(s),
                    screen::receive::Task::None => Task::none(),
                }
            }
            Message::ScreenSpace(message) => {
                match screen::space::update(&mut self.screen_space, message) {
                    screen::space::Task::SetSpace { space_name } => {
                        Task::done(Message::SetScreen(Screen::Space(space_name)))
                    }
                    screen::space::Task::BidSpace {
                        space_name,
                        bid_amount,
                    } => Task::done(Message::RpcRequest(RpcRequest::BidSpace {
                        space_name,
                        bid_amount,
                    })),
                    screen::space::Task::None => Task::none(),
                }
            }
            Message::ScreenTransactions(message) => match message {
                screen::transactions::Message::TxidCopyPress { txid } => clipboard::write(txid),
            },
        }
    }

    fn view(&self) -> Element<Message> {
        let main: Element<Message> = if self.store.wallet.is_some() {
            row![
                navbar(&self.screen),
                container(match self.screen {
                    Screen::Home => screen::home::view(
                        self.store.wallet.as_ref().unwrap().balance,
                        self.store.get_wallet_spaces().unwrap(),
                    )
                    .map(Message::ScreenHome),
                    Screen::Send => screen::send::view(&self.screen_send).map(Message::ScreenSend),
                    Screen::Receive => screen::receive::view(
                        &self.screen_receive,
                        self.store.wallet.as_ref().unwrap().coin_address.as_ref(),
                        self.store.wallet.as_ref().unwrap().space_address.as_ref(),
                    )
                    .map(Message::ScreenReceive),
                    Screen::Space(ref space_name) => screen::space::view(
                        &self.screen_space,
                        space_name,
                        self.store.spaces.get(space_name),
                        self.store
                            .wallet
                            .as_ref()
                            .unwrap()
                            .space_names
                            .contains(space_name),
                    )
                    .map(Message::ScreenSpace),
                    Screen::Transactions => screen::transactions::view(
                        &self.store.wallet.as_ref().unwrap().transactions
                    )
                    .map(Message::ScreenTransactions),
                })
                .style(|theme: &Theme| {
                    container::Style::default()
                        .background(theme.extended_palette().background.weak.color)
                })
            ]
            .into()
        } else {
            center(text("Loading").align_x(Center)).into()
        };
        Column::new()
            .push_maybe(self.rpc_error.as_ref().map(errorbar))
            .push(main)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.store.wallet.is_some() && self.rpc_error.is_none() {
            match self.screen {
                Screen::Transactions => time::every(time::Duration::from_secs(5))
                    .map(|_| Message::RpcRequest(RpcRequest::GetTransactions)),
                _ => time::every(time::Duration::from_secs(5))
                    .map(|_| Message::RpcRequest(RpcRequest::GetServerInfo)),
            }
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
    let navbar_button = |label, icon: char, is_active, screen| {
        let button = button(row![text(icon).font(icon::FONT).size(18), text(label)].spacing(10))
            .style(if is_active {
                button::primary
            } else {
                button::text
            })
            .padding(10)
            .width(Fill);
        button.on_press(Message::SetScreen(screen))
    };

    container(column![
        navbar_button(
            "Home",
            icon::ARTBOARD,
            matches!(current_screen, Screen::Home),
            Screen::Home
        ),
        navbar_button(
            "Send",
            icon::ARROW_DOWN_FROM_ARC,
            matches!(current_screen, Screen::Send),
            Screen::Send
        ),
        navbar_button(
            "Receive",
            icon::ARROW_DOWN_TO_ARC,
            matches!(current_screen, Screen::Receive),
            Screen::Receive
        ),
        navbar_button(
            "Space",
            icon::AT,
            matches!(current_screen, Screen::Space(..)),
            Screen::Space(String::new())
        ),
        navbar_button(
            "Transactions",
            icon::ARROWS_EXCHANGE,
            matches!(current_screen, Screen::Transactions),
            Screen::Transactions
        ),
    ])
    .width(200)
    .into()
}
