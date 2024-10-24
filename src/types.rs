use iced::widget::qr_code::Data as QrCode;
use spaced::wallets;

pub use wallets::AddressKind;

#[derive(Debug)]
pub struct Address {
    pub address: String,
    pub qr_code: QrCode,
}

#[derive(Debug)]
pub struct Wallet {
    name: String,
    coin_address: Option<Address>,
    space_address: Option<Address>,
}

impl Wallet {
    pub fn new(name: String) -> Self {
        Self {
            name,
            coin_address: None,
            space_address: None,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_address(&self, address_kind: AddressKind) -> Option<&Address> {
        match address_kind {
            AddressKind::Coin => self.coin_address.as_ref(),
            AddressKind::Space => self.space_address.as_ref(),
        }
    }

    pub fn set_address(&mut self, address_kind: AddressKind, address: String) {
        let qr_code = QrCode::new(&address).unwrap();
        let address = Some(Address {
            address,
            qr_code,
        });
        match address_kind {
            AddressKind::Coin => self.coin_address = address,
            AddressKind::Space => self.space_address = address,
        }
    }
}
