use std::fmt;
use std::sync::Arc;

use iced::time;
use iced::widget::{button, center, column, container, image, row, text, vertical_rule, Column};
use iced::{clipboard, Center, Element, Fill, Subscription, Task};

use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use spaced::rpc::{
    BidParams, OpenParams, RegisterParams, RpcClient, RpcWalletRequest, RpcWalletTxBuilder,
    SendCoinsParams, ServerInfo,
};

use crate::screen;
use crate::store::*;
use crate::widget::{
    block::error,
    icon::{text_icon, Icon},
};

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
        slabel: SLabel,
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
        slabel: SLabel,
        amount: Amount,
        open: bool,
    },
    RegisterSpace {
        slabel: SLabel,
    },
}

#[derive(Debug, Clone)]
enum RpcResponse {
    GetServerInfo {
        result: RpcResult<ServerInfo>,
    },
    GetSpaceInfo {
        slabel: SLabel,
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
    RegisterSpace {
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

const LOGO: &[u8] = include_bytes!("../assets/logo.png");

impl App {
    pub fn run(args: crate::Args) -> iced::Result {
        let icon = iced::window::icon::from_file_data(LOGO, None).expect("Failed to load icon");
        let icons_font = include_bytes!("../assets/icons.ttf").as_slice();
        iced::application(Self::title, Self::update, Self::view)
            .font(icons_font)
            .subscription(Self::subscription)
            .window(iced::window::Settings {
                size: (1000.0, 500.0).into(),
                min_size: Some((1000.0, 500.0).into()),
                icon: Some(icon),
                ..Default::default()
            })
            .run_with(move || Self::new(args))
    }

    fn new(args: crate::Args) -> (Self, Task<Message>) {
        let rpc_client: Arc<HttpClient> = Arc::new(
            HttpClientBuilder::default()
                .build(args.spaced_rpc_url.unwrap())
                .unwrap(),
        );
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
                wallet: args.wallet.into(),
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
                    RpcRequest::GetSpaceInfo { slabel } => Task::perform(
                        async move {
                            use protocol::hasher::KeyHasher;
                            use spaced::store::Sha256;

                            let hash = hex::encode(Sha256::hash(slabel.as_ref()));
                            let result = client.get_space(&hash).await.map_err(RpcError::from);
                            RpcResponse::GetSpaceInfo { slabel, result }
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
                                                confirmed_only: false,
                                                skip_tx_check: false,
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
                        slabel,
                        amount,
                        open,
                    } => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let name = slabel.to_string();
                                    let amount = amount.to_sat();
                                    let result = client
                                        .wallet_send_request(
                                            &wallet,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![if open {
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
                                                confirmed_only: false,
                                                skip_tx_check: false,
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
                    RpcRequest::RegisterSpace { slabel } => {
                        if let Some(wallet) = self.store.get_wallet_name() {
                            Task::perform(
                                async move {
                                    let result = client
                                        .wallet_send_request(
                                            &wallet,
                                            RpcWalletTxBuilder {
                                                bidouts: None,
                                                requests: vec![RpcWalletRequest::Register(
                                                    RegisterParams {
                                                        name: slabel.to_string(),
                                                        to: None,
                                                    },
                                                )],
                                                fee_rate: None,
                                                dust: None,
                                                force: false,
                                                confirmed_only: false,
                                                skip_tx_check: false,
                                            },
                                        )
                                        .await
                                        .map(|_| ())
                                        .map_err(RpcError::from);
                                    RpcResponse::RegisterSpace { result }
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
                    RpcResponse::GetSpaceInfo { slabel, result } => {
                        match result {
                            Ok(out) => {
                                self.store.spaces.insert(
                                    slabel,
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
                                let spaces: Vec<_> = spaces
                                    .into_iter()
                                    .map(|out| {
                                        let space = out.space.unwrap();
                                        self.store
                                            .spaces
                                            .insert(space.name.clone(), Some(space.covenant));
                                        space.name
                                    })
                                    .collect();
                                if let Some(wallet) = self.store.get_wallet_with_name(&wallet) {
                                    wallet.spaces = spaces;
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
                    RpcResponse::RegisterSpace { result } => match result {
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
                        if let Ok(slabel) = SLabel::from_str_unprefixed(space_name) {
                            Task::done(Message::RpcRequest(RpcRequest::GetSpaceInfo {
                                slabel: slabel.clone(),
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
                        slabel,
                        amount,
                        open,
                    } => Task::done(Message::RpcRequest(RpcRequest::BidSpace {
                        slabel,
                        amount,
                        open,
                    })),
                    screen::space::Task::RegisterSpace { slabel } => {
                        Task::done(Message::RpcRequest(RpcRequest::RegisterSpace { slabel }))
                    }
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
                vertical_rule(2),
                container(match self.screen {
                    Screen::Home => screen::home::view(
                        self.store.wallet.as_ref().unwrap().balance,
                        self.store.tip_height,
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
                    Screen::Space(ref space_name) => {
                        screen::space::view(
                            &self.screen_space,
                            self.store.tip_height,
                            space_name,
                            match SLabel::from_str_unprefixed(&space_name) {
                                Ok(slabel) => Some((
                                    slabel.clone(),
                                    self.store.spaces.get(&slabel),
                                    self.store.wallet.as_ref().unwrap().spaces.contains(&slabel),
                                )),
                                Err(_) => None,
                            },
                        )
                        .map(Message::ScreenSpace)
                    }
                    Screen::Transactions => screen::transactions::view(
                        &self.store.wallet.as_ref().unwrap().transactions
                    )
                    .map(Message::ScreenTransactions),
                })
            ]
            .into()
        } else {
            center(text("Loading").align_x(Center)).into()
        };
        Column::new()
            .push_maybe(self.rpc_error.as_ref().map(error))
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

fn navbar<'a>(current_screen: &'a Screen) -> Element<'a, Message> {
    let navbar_button = |label, icon: Icon, is_active, screen| {
        let button = button(row![text_icon(icon).size(18), text(label)].spacing(10))
            .style(move |theme, status| {
                let mut style = (if is_active {
                    button::primary
                } else {
                    button::text
                })(theme, status);
                style.border = style.border.rounded(0.0);
                style
            })
            .padding(10)
            .width(Fill);
        button.on_press(Message::SetScreen(screen))
    };

    container(column![
        image(image::Handle::from_bytes(LOGO)),
        navbar_button(
            "Home",
            Icon::Artboard,
            matches!(current_screen, Screen::Home),
            Screen::Home
        ),
        navbar_button(
            "Send",
            Icon::ArrowDownFromArc,
            matches!(current_screen, Screen::Send),
            Screen::Send
        ),
        navbar_button(
            "Receive",
            Icon::ArrowDownToArc,
            matches!(current_screen, Screen::Receive),
            Screen::Receive
        ),
        navbar_button(
            "Space",
            Icon::At,
            matches!(current_screen, Screen::Space(..)),
            Screen::Space(String::new())
        ),
        navbar_button(
            "Transactions",
            Icon::ArrowsExchange,
            matches!(current_screen, Screen::Transactions),
            Screen::Transactions
        ),
    ])
    .width(200)
    .into()
}
