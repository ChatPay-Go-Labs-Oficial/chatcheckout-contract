#![no_std]

use crate::storage::{EscrowData, EscrowStatus};
use soroban_sdk::{Address, Env, String};

// ============================================================================
// Input Validation Functions
// ============================================================================

/// Validate amount is positive
pub fn validate_amount_positive(amount: i128) {
    if amount <= 0 {
        panic!("Invalid amount: must be greater than 0");
    }
}

/// Validate fee basis points is within valid range (0-10000)
pub fn validate_fee_bps_range(fee_bps: u32) {
    if fee_bps > 10_000 {
        panic!("Invalid fee_bps: must be <= 10000 (100%)");
    }
}

/// Validate guarantee days is within reasonable range (1-36500)
pub fn validate_guarantee_days(days: u32) {
    if days == 0 || days > 36_500 {
        panic!("Invalid guarantee_days: must be between 1 and 36500");
    }
}

/// Validate product ID is not empty
pub fn validate_product_id_not_empty(product_id: &String) {
    if product_id.is_empty() {
        panic!("Invalid product_id: cannot be empty");
    }
}

/// Validate all parameters for create_escrow
pub fn validate_create_escrow_params(
    amount: i128,
    fee_bps: u32,
    guarantee_days: u32,
    product_id: &String,
) {
    validate_amount_positive(amount);
    validate_fee_bps_range(fee_bps);
    validate_guarantee_days(guarantee_days);
    validate_product_id_not_empty(product_id);
}

// ============================================================================
// Business Rule Validation Functions
// ============================================================================

/// Validate escrow is in active status
pub fn validate_escrow_active(status: EscrowStatus) {
    if let EscrowStatus::Active = status {
    } else {
        panic!("Escrow not active");
    }
}

/// Validate refund window has not expired
pub fn validate_refund_window(escrow: &EscrowData, env: &Env) {
    let now = env.ledger().timestamp();
    if now >= escrow.release_at {
        panic!("Guarantee period expired");
    }
}

/// Validate fee does not exceed amount
pub fn validate_fee_not_exceeds_amount(amount: i128, fee: i128) {
    if fee >= amount {
        panic!("Fee exceeds or equals amount");
    }
}
