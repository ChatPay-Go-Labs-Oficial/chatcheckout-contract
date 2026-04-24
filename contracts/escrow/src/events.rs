use soroban_sdk::{contractevent, Address, String};

// ============================================================================
// Event Types
// ============================================================================

/// Event emitted when a new escrow is created
#[contractevent]
pub struct CreateEscrowEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub asset: Address,
    pub fee_bps: u32,
    pub guarantee_days: u32,
    pub product_id: String,
    pub allow_early_release: bool,
}

/// Event emitted when payment is released to seller
#[contractevent]
pub struct ReleasePaymentEvent {
    pub escrow_id: u64,
    pub seller: Address,
    pub amount: i128,
    pub fee: i128,
    pub to_seller: i128,
}

/// Event emitted when refund is issued to buyer
#[contractevent]
pub struct RequestRefundEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub amount: i128,
    pub asset: Address,
}

// ============================================================================
// Token Allowlist Events
// ============================================================================

/// Event emitted when a token is added to the allowed list
#[contractevent]
pub struct TokenAddedEvent {
    pub token: Address,
}

/// Event emitted when a token is removed from the allowed list
#[contractevent]
pub struct TokenRemovedEvent {
    pub token: Address,
}

// ============================================================================
// Dispute Resolution Events
// ============================================================================

/// Event emitted when a dispute is initiated
#[contractevent]
pub struct DisputeEscrowEvent {
    pub escrow_id: u64,
    pub initiator: Address,
}
/// Event emitted when a party proposes a resolution
#[contractevent]
pub struct ProposeResolutionEvent {
    pub escrow_id: u64,
    pub proposer: Address,
    pub favor_buyer: bool,
}

/// Event emitted when dispute is resolved
#[contractevent]
pub struct ResolveDisputeEvent {
    pub escrow_id: u64,
    pub favor_buyer: bool,
    pub amount: i128,
    pub fee: i128,
    pub recipient: Address,
    pub resolved_by: Address,
}

/// Event emitted when admin withdraws funds
#[contractevent]
pub struct AdminWithdrawEvent {
    pub admin: Address,
    pub asset: Address,
    pub amount: i128,
}
