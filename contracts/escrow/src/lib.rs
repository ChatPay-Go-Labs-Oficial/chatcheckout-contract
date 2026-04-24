#![no_std]

// Declare modules
mod error;
mod events;
mod math;
mod signature;
mod storage;
mod validation;

use crate::error::EscrowError;
use crate::events::*;
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

// Re-export commonly used types from storage module
pub use storage::{Config, EscrowData, EscrowStatus};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize contract configuration
    /// Can only be called once on deploy (or when empty).
    pub fn __constructor(env: &Env, admin: Address, collect_on_create: bool) -> Result<(), EscrowError> {
        if env.storage().instance().has(&storage::DataKey::Config) {
            return Err(EscrowError::AlreadyInitialized);
        }
        admin.require_auth();

        let cfg = storage::Config {
            admin: admin.clone(),
            collect_on_create,
        };
        storage::write_config(env, &cfg);
        Ok(())
    }

    /// Update configuration (only current admin)
    pub fn update_config(env: Env, new_admin: Address, collect_on_create: bool) {
        let cfg = storage::read_config(&env);
        cfg.admin.require_auth(); // current admin authorizes

        // Extend instance storage TTL (Config is critical data)
        env.storage().instance().extend_ttl(100, 518_400);

        let new_cfg = storage::Config {
            admin: new_admin,
            collect_on_create,
        };
        storage::write_config(&env, &new_cfg);
    }

    pub fn get_config(env: Env) -> storage::Config {
        storage::read_config(&env)
    }

    /// Withdraw funds from the contract (admin only)
    pub fn admin_withdraw(env: Env, asset: Address, amount: i128) -> Result<(), EscrowError> {
        let cfg = storage::read_config(&env);
        cfg.admin.require_auth();

        validation::validate_amount_positive(amount)?;

        let token = soroban_sdk::token::Client::new(&env, &asset);
        token.transfer(&env.current_contract_address(), &cfg.admin, &amount);

        AdminWithdrawEvent {
            admin: cfg.admin,
            asset,
            amount,
        }
        .publish(&env);

        Ok(())
    }

    // ============================================================================
    // Token Allowlist Management (Admin Only)
    // ============================================================================

    /// Add a token to the allowed list (admin only)
    /// This ensures only trusted tokens can be used in escrows
    pub fn add_allowed_token(env: Env, token: Address) {
        let cfg = storage::read_config(&env);
        cfg.admin.require_auth();

        // Extend instance storage TTL
        env.storage().instance().extend_ttl(100, 518_400);

        storage::add_allowed_token(&env, &token);

        // Emit event: token added to allowlist
        TokenAddedEvent { token }.publish(&env);
    }

    /// Remove a token from the allowed list (admin only)
    /// Note: Existing escrows with this token will still function
    pub fn remove_allowed_token(env: Env, token: Address) {
        let cfg = storage::read_config(&env);
        cfg.admin.require_auth();

        // Extend instance storage TTL
        env.storage().instance().extend_ttl(100, 518_400);

        storage::remove_allowed_token(&env, &token);

        // Emit event: token removed from allowlist
        TokenRemovedEvent { token }.publish(&env);
    }

    /// Get the list of allowed tokens (public)
    pub fn get_allowed_tokens(env: Env) -> Vec<Address> {
        storage::read_allowed_tokens(&env)
    }

    /// Check if a token is allowed (public helper function)
    pub fn is_token_allowed(env: Env, token: Address) -> bool {
        storage::is_token_allowed(&env, &token)
    }

    /// Create escrow with guarantee period
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        asset: Address,
        fee_bps: u32,
        guarantee_days: u32,
        product_id: String,
        allow_early_release: bool,
    ) -> Result<u64, EscrowError> {
        // Require buyer authorization (required for transfers from them)
        buyer.require_auth();

        // Extend instance storage TTL (Counter and Config are critical data)
        env.storage().instance().extend_ttl(100, 518_400);

        // Input validation
        validation::validate_create_escrow_params(amount, fee_bps, guarantee_days, &product_id)?;

        // Validate token is in allowed list
        validation::validate_token_allowed(&env, &asset)?;

        let now = env.ledger().timestamp();
        let release_time = math::calc_release_timestamp(now, guarantee_days);

        // Sequential ID
        let counter = storage::read_counter(&env);
        let escrow_id = counter.checked_add(1).ok_or(EscrowError::CounterOverflow)?;

        // Calculate fee using fee_bps received as parameter (from backend)
        let cfg = storage::read_config(&env);
        let fee = math::calc_fee(amount, fee_bps);

        // 1) Transfer buyer's amount to contract (amount held in custody)
        let token = soroban_sdk::token::Client::new(&env, &asset);
        token.transfer(&buyer, &env.current_contract_address(), &amount);

        // 2) If configured to collect fee on create: transfer from buyer to admin
        if cfg.collect_on_create && fee > 0 {
            // Validate fee doesn't exceed amount before transfer
            validation::validate_fee_not_exceeds_amount(amount, fee)?;
            token.transfer(&buyer, &cfg.admin, &fee);
        }

        // Persist escrow
        let escrow = EscrowData {
            buyer: buyer.clone(),
            seller: seller.clone(),
            amount,
            asset: asset.clone(),
            created_at: now,
            release_at: release_time,
            status: EscrowStatus::Active,
            product_id: product_id.clone(),
            guarantee_days,
            fee_bps,
            allow_early_release,
            buyer_proposal: None,
            seller_proposal: None,
        };

        storage::write_escrow(&env, escrow_id, &escrow);
        storage::write_counter(&env, escrow_id);

        // Extend persistent storage TTL for the new escrow
        env.storage().persistent().extend_ttl(&storage::DataKey::Escrow(escrow_id), 100, 518_400);

        // Emit event: escrow created
        CreateEscrowEvent {
            escrow_id,
            buyer: buyer.clone(),
            seller: seller.clone(),
            amount,
            asset: asset.clone(),
            fee_bps,
            guarantee_days,
            product_id,
            allow_early_release,
        }
        .publish(&env);

        Ok(escrow_id)
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<EscrowData, EscrowError> {
        storage::read_escrow(&env, escrow_id)
    }

    // ============================================================================
    // Nonce Management
    // ============================================================================

    /// Get current nonce for a user
    /// Used to get the current nonce before calling functions that require it
    pub fn get_nonce(env: Env, user: Address) -> u64 {
        storage::read_nonce(&env, &user)
    }

    // ============================================================================
    // Escrow Operations
    // ============================================================================

    /// Release payment to seller (after guarantee period or by seller)
    /// Includes nonce for replay attack protection and traceability
    pub fn release_payment(env: Env, escrow_id: u64, seller: Address, nonce: u64) -> Result<(), EscrowError> {
        // 1. Verify nonce for replay protection
        signature::verify_and_increment_nonce(&env, &seller, nonce)?;

        // Extend persistent storage TTL for this escrow
        env.storage().persistent().extend_ttl(&storage::DataKey::Escrow(escrow_id), 100, 518_400);

        // 2. Verify seller's authorization using Soroban native auth
        // The signature is verified at the transaction level by Soroban
        seller.require_auth();

        // 3. Get and validate escrow
        let mut esc = storage::read_escrow(&env, escrow_id)?;
        validation::validate_escrow_active(esc.status.clone())?;

        // 4. Verify signer is the seller
        if esc.seller != seller {
            return Err(EscrowError::Unauthorized);
        }

        // 5. Check guarantee period has expired OR early release is allowed
        if esc.allow_early_release {
            // Seller can release anytime if buyer agreed to early release
        } else {
            // Seller must wait for guarantee period to expire
            validation::validate_guarantee_period_expired(esc.release_at, &env)?;
        }

        // 6. Process fee and payment (same as original)
        let token = soroban_sdk::token::Client::new(&env, &esc.asset);
        let cfg = storage::read_config(&env);
        let mut to_seller = esc.amount;
        let fee = math::calc_fee(esc.amount, esc.fee_bps);

        if !cfg.collect_on_create {
            if fee > 0 {
                validation::validate_fee_not_exceeds_amount(esc.amount, fee)?;
                token.transfer(&env.current_contract_address(), &cfg.admin, &fee);
                to_seller = esc.amount.checked_sub(fee).ok_or(EscrowError::FeeExceedsAmount)?;
            }
        }

        token.transfer(
            &env.current_contract_address(),
            &esc.seller.clone(),
            &to_seller,
        );

        esc.status = EscrowStatus::Released;
        storage::write_escrow(&env, escrow_id, &esc);

        ReleasePaymentEvent {
            escrow_id,
            seller: esc.seller,
            amount: esc.amount,
            fee,
            to_seller,
        }
        .publish(&env);

        Ok(())
    }

    /// Refund (only buyer and within guarantee period)
    /// Includes nonce for replay attack protection and traceability
    pub fn request_refund(env: Env, escrow_id: u64, buyer: Address, nonce: u64) -> Result<(), EscrowError> {
        // 1. Verify nonce for replay protection
        signature::verify_and_increment_nonce(&env, &buyer, nonce)?;

        // Extend persistent storage TTL for this escrow
        env.storage().persistent().extend_ttl(&storage::DataKey::Escrow(escrow_id), 100, 518_400);

        // 2. Verify buyer's authorization using Soroban native auth
        buyer.require_auth();

        // 3. Get and validate escrow
        let mut esc = storage::read_escrow(&env, escrow_id)?;

        // 4. Verify signer is the buyer
        if esc.buyer != buyer {
            return Err(EscrowError::Unauthorized);
        }

        // 5. Check escrow status
        validation::validate_escrow_active(esc.status.clone())?;

        // 6. Validate refund window
        validation::validate_refund_window(&esc, &env)?;

        // 7. Process refund
        let token = soroban_sdk::token::Client::new(&env, &esc.asset);
        token.transfer(&env.current_contract_address(), &esc.buyer, &esc.amount);

        esc.status = EscrowStatus::Refunded;
        storage::write_escrow(&env, escrow_id, &esc);

        RequestRefundEvent {
            escrow_id,
            buyer: esc.buyer.clone(),
            amount: esc.amount,
            asset: esc.asset.clone(),
        }
        .publish(&env);

        Ok(())
    }

    // ============================================================================
    // Dispute Resolution
    // ============================================================================

    /// Initiate a dispute on an active escrow
    pub fn dispute_escrow(
        env: Env,
        escrow_id: u64,
        caller: Address,
        nonce: u64,
    ) -> Result<(), EscrowError> {
        signature::verify_and_increment_nonce(&env, &caller, nonce)?;
        caller.require_auth();

        env.storage().persistent().extend_ttl(&storage::DataKey::Escrow(escrow_id), 100, 518_400);

        let mut esc = storage::read_escrow(&env, escrow_id)?;

        if esc.buyer != caller && esc.seller != caller {
            return Err(EscrowError::Unauthorized);
        }

        validation::validate_can_dispute(&esc, &env)?;

        esc.status = EscrowStatus::Disputed;
        storage::write_escrow(&env, escrow_id, &esc);

        DisputeEscrowEvent {
            escrow_id,
            initiator: caller,
        }
        .publish(&env);

        Ok(())
    }

    /// Propose a resolution for a disputed escrow
    pub fn propose_resolution(
        env: Env,
        escrow_id: u64,
        caller: Address,
        nonce: u64,
        favor_buyer: bool,
    ) -> Result<(), EscrowError> {
        signature::verify_and_increment_nonce(&env, &caller, nonce)?;
        caller.require_auth();

        env.storage().persistent().extend_ttl(&storage::DataKey::Escrow(escrow_id), 100, 518_400);

        let mut esc = storage::read_escrow(&env, escrow_id)?;

        let is_buyer = esc.buyer == caller;
        let is_seller = esc.seller == caller;

        if !is_buyer && !is_seller {
            return Err(EscrowError::Unauthorized);
        }

        validation::validate_escrow_disputed(esc.status.clone())?;

        if is_buyer {
            validation::validate_not_yet_proposed(esc.buyer_proposal)?;
            esc.buyer_proposal = Some(favor_buyer);
        } else {
            validation::validate_not_yet_proposed(esc.seller_proposal)?;
            esc.seller_proposal = Some(favor_buyer);
        }

        storage::write_escrow(&env, escrow_id, &esc);

        ProposeResolutionEvent {
            escrow_id,
            proposer: caller,
            favor_buyer,
        }
        .publish(&env);

        Ok(())
    }

    /// Resolve a disputed escrow when both parties agree
    pub fn resolve_dispute(
        env: Env,
        escrow_id: u64,
        caller: Address,
        nonce: u64,
    ) -> Result<(), EscrowError> {
        signature::verify_and_increment_nonce(&env, &caller, nonce)?;
        caller.require_auth();

        env.storage().persistent().extend_ttl(&storage::DataKey::Escrow(escrow_id), 100, 518_400);

        let mut esc = storage::read_escrow(&env, escrow_id)?;

        if esc.buyer != caller && esc.seller != caller {
            return Err(EscrowError::Unauthorized);
        }

        validation::validate_escrow_disputed(esc.status.clone())?;

        let favor_buyer = validation::validate_dispute_resolution(
            esc.buyer_proposal,
            esc.seller_proposal,
        )?;

        Self::execute_dispute_resolution(&env, escrow_id, &mut esc, favor_buyer, caller)?;

        Ok(())
    }

    /// Admin resolves dispute when parties can't agree
    pub fn admin_resolve_dispute(
        env: Env,
        escrow_id: u64,
        favor_buyer: bool,
        nonce: u64,
    ) -> Result<(), EscrowError> {
        let cfg = storage::read_config(&env);
        signature::verify_and_increment_nonce(&env, &cfg.admin, nonce)?;
        cfg.admin.require_auth();

        env.storage().persistent().extend_ttl(&storage::DataKey::Escrow(escrow_id), 100, 518_400);

        let mut esc = storage::read_escrow(&env, escrow_id)?;

        validation::validate_escrow_disputed(esc.status.clone())?;

        Self::execute_dispute_resolution(&env, escrow_id, &mut esc, favor_buyer, cfg.admin)?;

        Ok(())
    }

    /// Helper function to execute dispute resolution (refund or release)
    fn execute_dispute_resolution(
        env: &Env,
        escrow_id: u64,
        esc: &mut EscrowData,
        favor_buyer: bool,
        resolved_by: Address,
    ) -> Result<(), EscrowError> {
        let token = soroban_sdk::token::Client::new(env, &esc.asset);
        let cfg = storage::read_config(env);
        let fee = math::calc_fee(esc.amount, esc.fee_bps);

        if favor_buyer {
            let mut to_buyer = esc.amount;

            if !cfg.collect_on_create && fee > 0 {
                validation::validate_fee_not_exceeds_amount(esc.amount, fee)?;
                token.transfer(&env.current_contract_address(), &cfg.admin, &fee);
                to_buyer = esc.amount.checked_sub(fee).ok_or(EscrowError::FeeExceedsAmount)?;
            }

            token.transfer(&env.current_contract_address(), &esc.buyer, &to_buyer);
            esc.status = EscrowStatus::Refunded;

            ResolveDisputeEvent {
                escrow_id,
                favor_buyer: true,
                amount: esc.amount,
                fee,
                recipient: esc.buyer.clone(),
                resolved_by,
            }
            .publish(env);
        } else {
            let mut to_seller = esc.amount;

            if !cfg.collect_on_create && fee > 0 {
                validation::validate_fee_not_exceeds_amount(esc.amount, fee)?;
                token.transfer(&env.current_contract_address(), &cfg.admin, &fee);
                to_seller = esc.amount.checked_sub(fee).ok_or(EscrowError::FeeExceedsAmount)?;
            }

            token.transfer(&env.current_contract_address(), &esc.seller, &to_seller);
            esc.status = EscrowStatus::Released;

            ResolveDisputeEvent {
                escrow_id,
                favor_buyer: false,
                amount: esc.amount,
                fee,
                recipient: esc.seller.clone(),
                resolved_by,
            }
            .publish(env);
        }

        storage::write_escrow(env, escrow_id, esc);
        Ok(())
    }
}

mod test;
