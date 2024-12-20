pub fn height_to_est(block_height: u32, tip_height: u32) -> String {
    if block_height <= tip_height {
        return "now".to_string();
    }

    let remaining_blocks = block_height - tip_height;

    if remaining_blocks <= 5 {
        return format!("in {} minutes", remaining_blocks * 10);
    }

    if remaining_blocks <= 144 {
        let hours = remaining_blocks / 6;
        let remaining_blocks = remaining_blocks % 6;
        let minutes = remaining_blocks * 10;
        if minutes == 0 {
            return format!("in {} hours", hours);
        }
        return format!("in {} hours {} minutes", hours, minutes);
    }

    let days = remaining_blocks / 144;
    let remaining_blocks = remaining_blocks % 144;
    let hours = remaining_blocks / 6;

    if hours == 0 {
        return format!("in {} days", days);
    }
    return format!("in {} days {} hours", days, hours);
}

pub mod input {
    pub use wallet::bitcoin::{Amount, Denomination, FeeRate};

    pub fn recipient_chars(s: &str) -> bool {
        s.chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || c == '-' || c == '@')
    }

    pub fn recipient_value(s: &str) -> Option<String> {
        // TODO: check
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }

    pub fn amount_chars(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit())
    }

    pub fn amount_value(s: &str) -> Option<Amount> {
        Amount::from_str_in(s, Denomination::Satoshi).ok()
    }

    pub fn fee_rate_chars(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit() || c == '.')
    }

    pub fn fee_rate_value(s: &str) -> Option<Option<FeeRate>> {
        if s.is_empty() {
            Some(None)
        } else {
            s.parse().ok().map(|n| FeeRate::from_sat_per_vb(n))
        }
    }
}
