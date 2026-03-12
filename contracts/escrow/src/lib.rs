#![no_std]

// Declare modules
mod error;
mod events;
mod math;
mod signature;
mod storage;
mod validation;

use crate::events::*;
use soroban_sdk::{contract, contractimpl, vec, Address, Env, String, Vec};

// Re-export commonly used types from storage module
pub use storage::{Config, EscrowData, EscrowStatus};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Inicializa a configuração do contrato
    /// Somente pode ser chamado 1x no deploy (ou quando vazio).
    pub fn __constructor(env: &Env, admin: Address, collect_on_create: bool) {
        if env.storage().instance().has(&storage::DataKey::Config) {
            panic!("Already initialized");
        }
        admin.require_auth();

        let cfg = storage::Config {
            admin: admin.clone(),
            collect_on_create,
        };
        storage::write_config(env, &cfg);
    }

    /// Atualiza a configuração (somente admin atual)
    pub fn update_config(env: Env, new_admin: Address, collect_on_create: bool) {
        let cfg = storage::read_config(&env);
        cfg.admin.require_auth(); // admin atual autoriza

        let new_cfg = storage::Config {
            admin: new_admin,
            collect_on_create,
        };
        storage::write_config(&env, &new_cfg);
    }

    pub fn get_config(env: Env) -> storage::Config {
        storage::read_config(&env)
    }

    /// Cria escrow com período de garantia
    pub fn create_escrow(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        asset: Address,
        fee_bps: u32,
        guarantee_days: u32,
        product_id: String,
    ) -> u64 {
        // Require buyer authorization (necessário para transferências a partir dele)
        buyer.require_auth();

        // Input validation
        validation::validate_create_escrow_params(amount, fee_bps, guarantee_days, &product_id)
            .unwrap();

        let now = env.ledger().timestamp();
        let release_time = math::calc_release_timestamp(now, guarantee_days);

        // ID sequencial
        let counter = storage::read_counter(&env);
        let escrow_id = counter
            .checked_add(1)
            .unwrap_or_else(|| panic!("Counter overflow"));

        // Calcular taxa usando o fee_bps recebido como parâmetro (do backend)
        let cfg = storage::read_config(&env);
        let fee = math::calc_fee(amount, fee_bps);

        // 1) Transferir o valor do comprador para o contrato (valor que ficará em custódia)
        let token = soroban_sdk::token::Client::new(&env, &asset);
        token.transfer(&buyer, &env.current_contract_address(), &amount);

        // 2) Se configurado para cobrar a taxa no create: transferir do comprador para o admin
        if cfg.collect_on_create && fee > 0 {
            token.transfer(&buyer, &cfg.admin, &fee);
        }

        // Persistir escrow
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

            fee_bps, // usa o fee_bps recebido como parâmetro

            // Dispute tracking fields (initialized)
            disputed_by_buyer: false,
            buyer_resolution: 0u32,
            seller_resolution: 0u32,
        };

        storage::write_escrow(&env, escrow_id, &escrow);
        storage::write_counter(&env, escrow_id);

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
        }
        .publish(&env);

        escrow_id
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> EscrowData {
        storage::read_escrow(&env, escrow_id)
    }

    /// Lista todos os escrows de um seller
    pub fn get_seller_escrows(env: Env, seller: Address) -> Vec<u64> {
        let counter = storage::read_counter(&env);
        let mut result = vec![&env];

        for i in 1..=counter {
            if storage::has_escrow(&env, i) {
                let esc = storage::read_escrow(&env, i);
                if esc.seller == seller {
                    result.push_back(i);
                }
            }
        }
        result
    }

    /// Resolve disputa (pode ser chamado por qualquer um após ambas as partes concordarem)
    pub fn res_disp(env: Env, escrow_id: u64) {
        let mut esc = storage::read_escrow(&env, escrow_id);

        // Check escrow status (must be disputed)
        validation::validate_escrow_disputed(esc.status.clone()).unwrap();

        // Check both parties have proposed resolutions
        let buyer_vote = esc.buyer_resolution;
        let seller_vote = esc.seller_resolution;

        // Both must have voted (value != 0)
        if buyer_vote == 0 || seller_vote == 0 {
            panic!("Both parties must propose a resolution before resolving");
        }

        // Check if both agree
        if buyer_vote != seller_vote {
            panic!("Both parties must agree on the resolution");
        }

        // Execute resolution based on agreement
        let token = soroban_sdk::token::Client::new(&env, &esc.asset);

        if buyer_vote == 1 {
            // Both agreed: favor buyer (refund)
            token.transfer(&env.current_contract_address(), &esc.buyer, &esc.amount);

            esc.status = EscrowStatus::Refunded;

            // Emit event: dispute resolved in favor of buyer
            ResolveDisputeEvent {
                escrow_id,
                resolved_in_favor_of: esc.buyer.clone(),
                amount: esc.amount,
                resolution_type: false, // false = refunded to buyer
            }
            .publish(&env);
        } else {
            // Both agreed: favor seller (release payment)
            let cfg = storage::read_config(&env);
            let mut to_seller = esc.amount;

            // Calculate and collect fee if not collected at creation
            if !cfg.collect_on_create {
                let fee = math::calc_fee(esc.amount, esc.fee_bps);
                if fee > 0 {
                    token.transfer(&env.current_contract_address(), &cfg.admin, &fee);
                    to_seller = esc.amount.checked_sub(fee).expect("Fee exceeds amount");
                }
            }

            token.transfer(
                &env.current_contract_address(),
                &esc.seller.clone(),
                &to_seller,
            );

            esc.status = EscrowStatus::Released;

            // Emit event: dispute resolved in favor of seller
            ResolveDisputeEvent {
                escrow_id,
                resolved_in_favor_of: esc.seller.clone(),
                amount: to_seller,
                resolution_type: true, // true = released to seller
            }
            .publish(&env);
        }

        storage::write_escrow(&env, escrow_id, &esc);
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

    /// Libera pagamento para o seller (após garantia ou pelo seller)
    /// Inclui nonce para proteção contra replay attacks e rastreabilidade
    pub fn release_payment(env: Env, escrow_id: u64, seller: Address, nonce: u64) {
        // 1. Verify nonce for replay protection
        signature::verify_and_increment_nonce(&env, &seller, nonce).unwrap();

        // 2. Verify seller's authorization using Soroban native auth
        // The signature is verified at the transaction level by Soroban
        seller.require_auth();

        // 3. Get and validate escrow
        let mut esc = storage::read_escrow(&env, escrow_id);
        validation::validate_escrow_active(esc.status.clone()).unwrap();

        // 4. Verify signer is the seller
        if esc.seller != seller {
            panic!("Signer must be the seller");
        }

        // 5. Check guarantee period (same logic as original)
        let now = env.ledger().timestamp();
        let is_expired = math::is_expired(now, esc.release_at);

        if !is_expired {
            // Before guarantee period, only seller can release (already verified)
        }

        // 6. Process fee and payment (same as original)
        let token = soroban_sdk::token::Client::new(&env, &esc.asset);
        let cfg = storage::read_config(&env);
        let mut to_seller = esc.amount;
        let fee = math::calc_fee(esc.amount, esc.fee_bps);

        if !cfg.collect_on_create {
            if fee > 0 {
                validation::validate_fee_not_exceeds_amount(esc.amount, fee).unwrap();
                token.transfer(&env.current_contract_address(), &cfg.admin, &fee);
                to_seller = esc.amount.checked_sub(fee).expect("Fee exceeds amount");
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
    }

    /// Reembolso (apenas buyer e dentro do período de garantia)
    /// Inclui nonce para proteção contra replay attacks e rastreabilidade
    pub fn request_refund(env: Env, escrow_id: u64, buyer: Address, nonce: u64) {
        // 1. Verify nonce for replay protection
        signature::verify_and_increment_nonce(&env, &buyer, nonce).unwrap();

        // 2. Verify buyer's authorization using Soroban native auth
        buyer.require_auth();

        // 3. Get and validate escrow
        let mut esc = storage::read_escrow(&env, escrow_id);

        // 4. Verify signer is the buyer
        if esc.buyer != buyer {
            panic!("Signer must be the buyer");
        }

        // 5. Check escrow status
        validation::validate_escrow_active(esc.status.clone()).unwrap();

        // 6. Validate refund window
        validation::validate_refund_window(&esc, &env).unwrap();

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
    }

    /// Marca como disputado
    /// Inclui nonce para proteção contra replay attacks e rastreabilidade
    pub fn dispute_escrow(env: Env, escrow_id: u64, as_buyer: bool, user: Address, nonce: u64) {
        // 1. Verify nonce for replay protection
        signature::verify_and_increment_nonce(&env, &user, nonce).unwrap();

        // 2. Verify user's authorization using Soroban native auth
        user.require_auth();

        // 3. Get and validate escrow
        let mut esc = storage::read_escrow(&env, escrow_id);

        // 4. Verify signer is the correct party
        if as_buyer {
            if esc.buyer != user {
                panic!("Signer must be the buyer");
            }
        } else {
            if esc.seller != user {
                panic!("Signer must be the seller");
            }
        }

        // 5. Check escrow status
        validation::validate_escrow_active(esc.status.clone()).unwrap();

        // 6. Track who initiated the dispute
        let disputed_by = if as_buyer {
            esc.buyer.clone()
        } else {
            esc.seller.clone()
        };

        esc.disputed_by_buyer = as_buyer;
        esc.status = EscrowStatus::Disputed;
        storage::write_escrow(&env, escrow_id, &esc);

        DisputeEscrowEvent {
            escrow_id,
            disputed_by,
            as_buyer,
        }
        .publish(&env);
    }

    /// Propõe uma resolução para a disputa (buyer ou seller)
    /// as_buyer: true se está sendo chamado pelo buyer, false se pelo seller
    /// favor_seller: true = favor seller (liberar pagamento), false = favor buyer (reembolso)
    /// Inclui nonce para proteção contra replay attacks e rastreabilidade
    pub fn prop_res(
        env: Env,
        escrow_id: u64,
        as_buyer: bool,
        favor_seller: bool,
        user: Address,
        nonce: u64,
    ) {
        // 1. Verify nonce for replay protection
        signature::verify_and_increment_nonce(&env, &user, nonce).unwrap();

        // 2. Verify user's authorization using Soroban native auth
        user.require_auth();

        // 3. Get and validate escrow
        let mut esc = storage::read_escrow(&env, escrow_id);

        // 4. Verify signer is the correct party
        if as_buyer {
            if esc.buyer != user {
                panic!("Signer must be the buyer");
            }
        } else {
            if esc.seller != user {
                panic!("Signer must be the seller");
            }
        }

        // 5. Check escrow status (must be disputed)
        validation::validate_escrow_disputed(esc.status.clone()).unwrap();

        // 6. Convert favor_seller to resolution value: 1 = favor buyer, 2 = favor seller
        let resolution_value = if favor_seller { 2u32 } else { 1u32 };

        // 7. Record resolution
        if as_buyer {
            // Check if buyer already voted
            if esc.buyer_resolution != 0 {
                panic!("Buyer already proposed a resolution");
            }
            esc.buyer_resolution = resolution_value;
        } else {
            // Check if seller already voted
            if esc.seller_resolution != 0 {
                panic!("Seller already proposed a resolution");
            }
            esc.seller_resolution = resolution_value;
        }

        storage::write_escrow(&env, escrow_id, &esc);

        // 8. Emit event
        let proposed_by = if as_buyer {
            esc.buyer.clone()
        } else {
            esc.seller.clone()
        };

        ProposeResolutionEvent {
            escrow_id,
            proposed_by,
            favor_seller,
        }
        .publish(&env);
    }
}

mod test;
