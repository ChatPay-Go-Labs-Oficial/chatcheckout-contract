/// Calculate fee based on amount and basis points
/// fee = amount * bps / 10000
pub fn calc_fee(amount: i128, bps: u32) -> i128 {
    let fee = amount * (bps as i128) / 10_000i128;
    if fee < 0 {
        0
    } else {
        fee
    }
}

/// Calculate release timestamp from current time and guarantee days
pub fn calc_release_timestamp(current: u64, days: u32) -> u64 {
    current + (days as u64 * 86_400)
}

/// Check if current time is past release time
pub fn is_expired(current: u64, release_at: u64) -> bool {
    current >= release_at
}

/// Convert days to seconds (for timestamp calculations)
pub const fn days_to_seconds(days: u32) -> u64 {
    days as u64 * 86_400
}
