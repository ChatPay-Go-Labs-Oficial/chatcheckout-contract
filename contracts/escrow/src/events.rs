use soroban_sdk::{contracttype, symbol_short, Address, Env, String};

// ============================================================================
// Event Types
// ============================================================================

/// Event emitted when a new escrow is created
#[contracttype]
pub struct CreateEscrowEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub asset: Address,
    pub fee_bps: u32,
    pub guarantee_days: u32,
    pub product_id: String,
}

impl CreateEscrowEvent {
    pub fn publish(self, env: &Env) {
        let topics = (symbol_short!("create"),);
        env.events().publish(topics, self);
    }
}

/// Event emitted when payment is released to seller
#[contracttype]
pub struct ReleasePaymentEvent {
    pub escrow_id: u64,
    pub seller: Address,
    pub amount: i128,
    pub fee: i128,
    pub to_seller: i128,
}

impl ReleasePaymentEvent {
    pub fn publish(self, env: &Env) {
        let topics = (symbol_short!("release"),);
        env.events().publish(topics, self);
    }
}

/// Event emitted when refund is issued to buyer
#[contracttype]
pub struct RequestRefundEvent {
    pub escrow_id: u64,
    pub buyer: Address,
    pub amount: i128,
    pub asset: Address,
}

impl RequestRefundEvent {
    pub fn publish(self, env: &Env) {
        let topics = (symbol_short!("refund"),);
        env.events().publish(topics, self);
    }
}

/// Event emitted when escrow is disputed
#[contracttype]
pub struct DisputeEscrowEvent {
    pub escrow_id: u64,
    pub disputed_by: Address,
    pub as_buyer: bool,
}

impl DisputeEscrowEvent {
    pub fn publish(self, env: &Env) {
        let topics = (symbol_short!("dispute"),);
        env.events().publish(topics, self);
    }
}
