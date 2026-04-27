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
