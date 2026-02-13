use crate::error::EscrowError;
use crate::storage::{EscrowData, EscrowStatus};
use soroban_sdk::{Env, String};

// ============================================================================
// Input Validation Functions
// ============================================================================

/// Validate amount is positive
pub fn validate_amount_positive(amount: i128) -> Result<(), EscrowError> {
    if amount <= 0 {
        Err(EscrowError::InvalidAmount)
    } else {
        Ok(())
    }
}

/// Validate fee basis points is within valid range (0-10000)
pub fn validate_fee_bps_range(fee_bps: u32) -> Result<(), EscrowError> {
    if fee_bps > 10_000 {
        Err(EscrowError::InvalidFeeBps)
    } else {
        Ok(())
    }
}

/// Validate guarantee days is within reasonable range (1-36500)
pub fn validate_guarantee_days(days: u32) -> Result<(), EscrowError> {
    if days == 0 || days > 36_500 {
        Err(EscrowError::InvalidGuaranteeDays)
    } else {
        Ok(())
    }
}

/// Validate product ID is not empty
pub fn validate_product_id_not_empty(product_id: &String) -> Result<(), EscrowError> {
    if product_id.is_empty() {
        Err(EscrowError::InvalidProductId)
    } else {
        Ok(())
    }
}

/// Validate all parameters for create_escrow
pub fn validate_create_escrow_params(
    amount: i128,
    fee_bps: u32,
    guarantee_days: u32,
    product_id: &String,
) -> Result<(), EscrowError> {
    validate_amount_positive(amount)?;
    validate_fee_bps_range(fee_bps)?;
    validate_guarantee_days(guarantee_days)?;
    validate_product_id_not_empty(product_id)?;
    Ok(())
}

// ============================================================================
// Business Rule Validation Functions
// ============================================================================

/// Validate escrow is in active status
pub fn validate_escrow_active(status: EscrowStatus) -> Result<(), EscrowError> {
    if let EscrowStatus::Active = status {
        Ok(())
    } else {
        Err(EscrowError::EscrowNotActive)
    }
}

/// Validate refund window has not expired
pub fn validate_refund_window(escrow: &EscrowData, env: &Env) -> Result<(), EscrowError> {
    let now = env.ledger().timestamp();
    if now >= escrow.release_at {
        Err(EscrowError::GuaranteePeriodExpired)
    } else {
        Ok(())
    }
}

/// Validate fee does not exceed amount
pub fn validate_fee_not_exceeds_amount(amount: i128, fee: i128) -> Result<(), EscrowError> {
    if fee >= amount {
        Err(EscrowError::FeeExceedsAmount)
    } else {
        Ok(())
    }
}
