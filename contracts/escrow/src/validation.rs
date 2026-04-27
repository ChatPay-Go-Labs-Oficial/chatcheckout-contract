use crate::error::EscrowError;
use crate::storage::{EscrowData, EscrowStatus};
use soroban_sdk::{Address, Env, String};

// ============================================================================
// Input Validation Functions
// ============================================================================

/// Validate buyer and seller are different addresses
pub fn validate_buyer_not_seller(buyer: &Address, seller: &Address) -> Result<(), EscrowError> {
    if buyer == seller {
        Err(EscrowError::InvalidSeller)
    } else {
        Ok(())
    }
}

/// Validate fee_bps does not exceed the on-chain maximum configured by admin
pub fn validate_fee_bps_within_limit(fee_bps: u32, max_fee_bps: u32) -> Result<(), EscrowError> {
    if fee_bps > max_fee_bps {
        Err(EscrowError::InvalidFeeBps)
    } else {
        Ok(())
    }
}

/// Validate that the seller is allowed to dispute (only after the guarantee period).
/// During the guarantee period the buyer has the self-service right to request a refund;
/// allowing the seller to dispute during that window would block it.
pub fn validate_seller_can_dispute(escrow: &EscrowData, env: &Env) -> Result<(), EscrowError> {
    let now = env.ledger().timestamp();
    if now < escrow.release_at {
        Err(EscrowError::DisputeNotAllowed)
    } else {
        Ok(())
    }
}

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
/// days must be at least 1 to ensure buyer protection
/// Maximum 100 years (36500 days)
pub fn validate_guarantee_days(days: u32) -> Result<(), EscrowError> {
    if days < 1 || days > 36_500 {
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

/// Validate guarantee period has expired (for release_payment)
pub fn validate_guarantee_period_expired(release_at: u64, env: &Env) -> Result<(), EscrowError> {
    let now = env.ledger().timestamp();
    if now < release_at {
        Err(EscrowError::GuaranteePeriodNotExpired)
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

// ============================================================================
// Token Validation Functions
// ============================================================================

/// Validate token is in the allowed list
pub fn validate_token_allowed(env: &Env, asset: &Address) -> Result<(), EscrowError> {
    if crate::storage::is_token_allowed(env, asset) {
        Ok(())
    } else {
        Err(EscrowError::TokenNotAllowed)
    }
}

// ============================================================================
// Dispute Validation Functions
// ============================================================================

/// Validate escrow is in disputed status
pub fn validate_escrow_disputed(status: EscrowStatus) -> Result<(), EscrowError> {
    if let EscrowStatus::Disputed = status {
        Ok(())
    } else {
        Err(EscrowError::NotDisputed)
    }
}

/// Validate if an escrow can be disputed
/// Only Active escrows can be disputed; Released/Refunded are terminal states for payouts.
pub fn validate_can_dispute(escrow: &EscrowData) -> Result<(), EscrowError> {
    match escrow.status {
        EscrowStatus::Disputed => Err(EscrowError::AlreadyDisputed),
        EscrowStatus::Active => Ok(()),
        _ => Err(EscrowError::EscrowNotActive),
    }
}

/// Validate both parties have proposed and agree on resolution
pub fn validate_dispute_resolution(
    buyer_proposal: Option<bool>,
    seller_proposal: Option<bool>,
) -> Result<bool, EscrowError> {
    match (buyer_proposal, seller_proposal) {
        (Some(buyer_choice), Some(seller_choice)) => {
            if buyer_choice == seller_choice {
                Ok(buyer_choice)
            } else {
                Err(EscrowError::BothPartiesMustAgree)
            }
        }
        _ => Err(EscrowError::NoDisputeToResolve),
    }
}

/// Validate party hasn't already proposed
pub fn validate_not_yet_proposed(proposal: Option<bool>) -> Result<(), EscrowError> {
    if proposal.is_some() {
        Err(EscrowError::AlreadyProposed)
    } else {
        Ok(())
    }
}
