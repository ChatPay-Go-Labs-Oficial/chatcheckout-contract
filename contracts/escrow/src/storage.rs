use soroban_sdk::{contracttype, Address, Env};

/// Storage keys for the escrow contract
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow(u64),
    Counter,
    Config,
    Nonce(Address),
}

/// Status of an escrow
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum EscrowStatus {
    Active,
    Released,
    Refunded,
    Disputed,
}

/// Contract configuration
#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub admin: Address,
    pub collect_on_create: bool, // true: cobra no create; false: cobra no release
}

/// Escrow data record
#[derive(Clone)]
#[contracttype]
pub struct EscrowData {
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub asset: Address,
    pub created_at: u64,
    pub release_at: u64,
    pub status: EscrowStatus,
    pub product_id: soroban_sdk::String,
    pub guarantee_days: u32,

    // snapshot da taxa usada na criação (vem como parâmetro do backend)
    pub fee_bps: u32,

    // Dispute tracking fields (0 = not set, 1 = favor buyer, 2 = favor seller)
    pub disputed_by_buyer: bool, // true if buyer initiated dispute
    pub buyer_resolution: u32,  // 0 = no vote, 1 = favor buyer (refund), 2 = favor seller (release)
    pub seller_resolution: u32,  // 0 = no vote, 1 = favor buyer (refund), 2 = favor seller (release)
}

// ============================================================================
// Config Storage Access
// ============================================================================

pub fn read_config(env: &Env) -> Config {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_or_else(|| panic!("Config not initialized"))
}

pub fn write_config(env: &Env, cfg: &Config) {
    env.storage().instance().set(&DataKey::Config, cfg);
}

// ============================================================================
// Escrow Storage Access
// ============================================================================

pub fn read_escrow(env: &Env, escrow_id: u64) -> EscrowData {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(escrow_id))
        .unwrap_or_else(|| panic!("Escrow not found"))
}

pub fn write_escrow(env: &Env, escrow_id: u64, escrow: &EscrowData) {
    env.storage()
        .persistent()
        .set(&DataKey::Escrow(escrow_id), escrow);
}

pub fn has_escrow(env: &Env, escrow_id: u64) -> bool {
    env.storage().persistent().has(&DataKey::Escrow(escrow_id))
}

// ============================================================================
// Counter Storage Access
// ============================================================================

pub fn read_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::Counter)
        .unwrap_or(0u64)
}

pub fn write_counter(env: &Env, value: u64) {
    env.storage().instance().set(&DataKey::Counter, &value);
}

// ============================================================================
// Nonce Storage Access (for meta-transactions)
// ============================================================================

/// Read the current nonce for a user (for replay protection in meta-transactions)
pub fn read_nonce(env: &Env, user: &Address) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::Nonce(user.clone()))
        .unwrap_or(0u64)
}

/// Write the nonce for a user
pub fn write_nonce(env: &Env, user: &Address, nonce: u64) {
    env.storage().instance().set(&DataKey::Nonce(user.clone()), &nonce);
}

/// Increment and return the previous nonce for a user
pub fn increment_nonce(env: &Env, user: &Address) -> u64 {
    let current = read_nonce(env, user);
    let next = current
        .checked_add(1)
        .unwrap_or_else(|| panic!("Nonce overflow"));
    write_nonce(env, user, next);
    current
}
