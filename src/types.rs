pub use protocol::{slabel::SLabel, Covenant};
pub use wallet::bitcoin::{Amount, Denomination, FeeRate};

pub fn is_slabel_input(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-')
}

pub fn slabel_from_str(s: &str) -> Option<SLabel> {
    SLabel::from_str_unprefixed(s).ok()
}

pub fn is_recipient_input(s: &str) -> bool {
    s.chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-' || c == '@')
}

pub fn recipient_from_str(s: &str) -> Option<String> {
    // TODO: check
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

pub fn is_amount_input(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

pub fn amount_from_str(s: &str) -> Option<Amount> {
    Amount::from_str_in(s, Denomination::Satoshi).ok()
}

pub fn is_fee_rate_input(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

pub fn fee_rate_from_str(s: &str) -> Option<Option<FeeRate>> {
    if s.is_empty() {
        Some(None)
    } else {
        s.parse().ok().map(|n| FeeRate::from_sat_per_vb(n))
    }
}
