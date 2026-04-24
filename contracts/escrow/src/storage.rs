use crate::error::EscrowError;
use soroban_sdk::{contracttype, Address, Env, Vec};

/// Storage keys for the escrow contract
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Escrow(u64),
    Counter,
    Config,
    Nonce(Address),
    AllowedTokens,
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
    pub collect_on_create: bool, // true: collect on create; false: collect on release
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
    /// Fee snapshot used at creation (comes as parameter from backend)
    pub fee_bps: u32,
    /// Whether seller can release payment before guarantee period expires
    pub allow_early_release: bool,
    /// Buyer's proposal for dispute resolution (Some(true) = favor buyer, Some(false) = favor seller)
    pub buyer_proposal: Option<bool>,
    /// Seller's proposal for dispute resolution
    pub seller_proposal: Option<bool>,
}

// ============================================================================
// Config Storage Access
// ============================================================================

pub fn read_config(env: &Env) -> Config {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_or_else(|| Err(EscrowError::ConfigNotInitialized).unwrap())
}

pub fn write_config(env: &Env, cfg: &Config) {
    env.storage().instance().set(&DataKey::Config, cfg);
}

// ============================================================================
// Escrow Storage Access
// ============================================================================

pub fn read_escrow(env: &Env, escrow_id: u64) -> Result<EscrowData, EscrowError> {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(escrow_id))
        .ok_or(EscrowError::EscrowNotFound)
}

pub fn write_escrow(env: &Env, escrow_id: u64, escrow: &EscrowData) {
    env.storage()
        .persistent()
        .set(&DataKey::Escrow(escrow_id), escrow);
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
        .ok_or(EscrowError::InvalidNonce).unwrap(); // Reusing InvalidNonce for overflow (unlikely in practice)
    write_nonce(env, user, next);
    current
}

// ============================================================================
// Allowed Tokens Storage Access
// ============================================================================

/// Read the list of allowed tokens
pub fn read_allowed_tokens(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::AllowedTokens)
        .unwrap_or_else(|| Vec::new(env))
}

/// Write the list of allowed tokens
pub fn write_allowed_tokens(env: &Env, tokens: &Vec<Address>) {
    env.storage().instance().set(&DataKey::AllowedTokens, tokens);
}

/// Check if a token is in the allowed list
pub fn is_token_allowed(env: &Env, token: &Address) -> bool {
    let allowed = read_allowed_tokens(env);
    allowed.iter().any(|t| t == *token)
}

/// Add a token to the allowed list
pub fn add_allowed_token(env: &Env, token: &Address) {
    let mut allowed = read_allowed_tokens(env);
    if !allowed.iter().any(|t| t == *token) {
        allowed.push_back(token.clone());
    }
    write_allowed_tokens(env, &allowed);
}

/// Remove a token from the allowed list
pub fn remove_allowed_token(env: &Env, token: &Address) {
    let allowed = read_allowed_tokens(env);
    let mut new_allowed = Vec::new(env);
    for t in allowed.iter() {
        if t != *token {
            new_allowed.push_back(t.clone());
        }
    }
    write_allowed_tokens(env, &new_allowed);
}
