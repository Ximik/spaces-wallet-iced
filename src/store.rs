use iced::widget::qr_code::Data as QrCode;
use rustc_hash::FxHashMap;
use spaced::wallets;

pub use protocol::{Covenant, FullSpaceOut};
pub use spaced::config::ExtendedNetwork;
pub use wallet::bitcoin::{Amount, Denomination};
pub use wallets::{AddressKind, Balance, WalletOutput};

#[derive(Debug)]
pub struct Address {
    pub text: String,
    pub qr_code: QrCode,
}

impl Address {
    pub fn new(text: String) -> Self {
        let qr_code = QrCode::new(&text).unwrap();
        Self { text, qr_code }
    }
}

#[derive(Default, Debug)]
pub struct Wallet {
    pub name: String,
    pub coin_address: Option<Address>,
    pub space_address: Option<Address>,
    pub balance: Amount,
    pub space_names: Vec<String>,
}

impl Wallet {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct Store {
    pub tip_height: u32,
    pub wallet: Option<Wallet>,
    pub spaces: FxHashMap<String, Option<Covenant>>,
}

impl Store {
    pub fn get_wallet_name(&self) -> Option<String> {
        self.wallet.as_ref().map(|wallet| wallet.name.clone())
    }

    pub fn get_wallet_with_name(&mut self, name: &str) -> Option<&mut Wallet> {
        self.wallet.as_mut().filter(|wallet| wallet.name == name)
    }

    pub fn get_wallet_spaces(&self) -> Option<impl Iterator<Item = (&String, &Option<Covenant>)>> {
        self.wallet.as_ref().map(|wallet| {
            wallet
                .space_names
                .iter()
                .map(|space_name| (space_name, self.spaces.get(space_name).unwrap_or(&None)))
        })
    }
}
