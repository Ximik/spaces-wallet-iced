pub fn height_to_est(block_height: u32, tip_height: u32) -> String {
    if block_height <= tip_height {
        return "now".to_string();
    }
    let d = block_height - tip_height;
    if d <= 5 {
        return format!("in {} minutes", d * 10);
    }
    if d <= 144 {
        return format!("in {} hours", (d + 5) / 6);
    }
    format!("in {} days", (d + 143) / 144)
}
