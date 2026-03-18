#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _, LedgerInfo};
use soroban_sdk::{Address, Env, String};

/// Helper function to advance time by X days
fn advance_time(env: &Env, days: u64) {
    let current = env.ledger().timestamp();
    let new_timestamp = current + (days * 86_400); // 86_400 seconds = 1 day

    env.ledger().set(LedgerInfo {
        timestamp: new_timestamp,
        protocol_version: 22, // Updated for compatible version with host
        sequence_number: 1234,
        network_id: Default::default(),
        base_reserve: 0,
        min_persistent_entry_ttl: 0,
        min_temp_entry_ttl: 0,
        max_entry_ttl: 0,
    });
}

#[test]
fn test_fee_calculation_correctness() {
    // Unit test to verify calc_fee is working correctly
    let fee = math::calc_fee(1000, 400); // 1000 * 400 / 10000 = 40
    assert_eq!(fee, 40);

    let fee2 = math::calc_fee(5000, 200); // 5000 * 200 / 10000 = 100
    assert_eq!(fee2, 100);

    let fee3 = math::calc_fee(100, 500); // 100 * 500 / 10000 = 5
    assert_eq!(fee3, 5);

    let fee4 = math::calc_fee(0, 400); // edge case: 0 amount
    assert_eq!(fee4, 0);
}

#[test]
fn test_time_advance() {
    let env = Env::default();

    // Check initial time
    let initial_time = env.ledger().timestamp();
    assert_eq!(initial_time, 0);

    // Advance 7 days
    advance_time(&env, 7);

    // Check that time has advanced
    let new_time = env.ledger().timestamp();
    assert_eq!(new_time, 7 * 86_400); // 7 days in seconds
}

#[test]
fn test_config_struct_refactored() {
    // Verify Config struct was correctly refactored
    // (only admin and collect_on_create, without fee_bps and flat_fee)
    let env = Env::default();
    let admin = Address::generate(&env);

    let cfg = Config {
        admin: admin.clone(),
        collect_on_create: true,
    };

    // If it compiles, it's correct
    assert_eq!(cfg.admin, admin);
    assert_eq!(cfg.collect_on_create, true);
}

#[test]
fn test_escrow_data_struct_refactored() {
    // Verify EscrowData struct was correctly refactored
    // (only fee_bps, without flat_fee, fee_collected_on_create and dispute fields)
    let env = Env::default();

    let escrow = EscrowData {
        buyer: Address::generate(&env),
        seller: Address::generate(&env),
        amount: 1000,
        asset: Address::generate(&env),
        created_at: 0,
        release_at: 100,
        status: EscrowStatus::Active,
        product_id: String::from_str(&env, "test"),
        guarantee_days: 7,
        fee_bps: 400, // Only fee_bps, without dispute fields
    };

    // If it compiles, it's correct
    assert_eq!(escrow.fee_bps, 400);
    assert_eq!(escrow.guarantee_days, 7);
}

#[test]
fn test_escrow_status_enum() {
    // Verify EscrowStatus enum works correctly (without Disputed)
    let status1 = EscrowStatus::Active;
    let status2 = EscrowStatus::Released;
    let status3 = EscrowStatus::Refunded;

    assert_eq!(status1, EscrowStatus::Active);
    assert_eq!(status2, EscrowStatus::Released);
    assert_eq!(status3, EscrowStatus::Refunded);
}

// ============================================================================
// Helper Functions for Tests
// ============================================================================

/// Helper function to set up test environment with contract and token
fn setup_contract_with_token() -> (Env, Address, soroban_sdk::token::StellarAssetClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Register SAC (Stellar Asset Contract)
    let _sac_contract = env.register_stellar_asset_contract_v2(admin.clone());
    // Token contract address is same as admin for SAC
    let token = soroban_sdk::token::StellarAssetClient::new(&env, &admin);

    // Register escrow contract - __constructor will be called automatically
    let contract_id = env.register(EscrowContract, (&admin, &false));

    (env, contract_id, token)
}

/// Helper to create a test escrow
fn create_test_escrow(
    env: &Env,
    contract_id: &Address,
    token: &soroban_sdk::token::StellarAssetClient,
    buyer: &Address,
    seller: &Address,
    amount: i128,
    fee_bps: u32,
    guarantee_days: u32,
) -> u64 {
    let client = EscrowContractClient::new(env, contract_id);

    // Mint tokens for buyer
    token.mint(buyer, &amount);

    // Create escrow - pass token address as asset
    let token_address = token.address.clone();
    let product_id = soroban_sdk::String::from_str(env, "product-123");
    client.create_escrow(buyer, seller, &amount, &token_address, &fee_bps, &guarantee_days, &product_id)
}

// ============================================================================
// Nonce Tests
// ============================================================================

#[test]
fn test_get_nonce_initial_zero() {
    let (env, contract_id, _token) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Initial nonce should be 0
    let nonce = client.get_nonce(&user);
    assert_eq!(nonce, 0);
}

// ============================================================================
// Zero Guarantee Days Tests
// ============================================================================

#[test]
fn test_validate_guarantee_days_zero_is_allowed() {
    // guarantee_days = 0 should be allowed (immediate release)
    let result = validation::validate_guarantee_days(0);
    assert!(result.is_ok());
}

#[test]
fn test_validate_guarantee_days_one_is_allowed() {
    let result = validation::validate_guarantee_days(1);
    assert!(result.is_ok());
}

#[test]
fn test_validate_guarantee_days_too_large_rejected() {
    let result = validation::validate_guarantee_days(36_501);
    assert!(result.is_err());
}

#[test]
fn test_calc_release_timestamp_with_zero_days() {
    let env = Env::default();
    let now = env.ledger().timestamp();

    // With guarantee_days = 0, release_at should equal current timestamp
    let release_at = math::calc_release_timestamp(now, 0);
    assert_eq!(release_at, now);
}

#[test]
fn test_release_payment_immediate_with_zero_guarantee_days() {
    let (env, contract_id, token) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32; // 4%
    let guarantee_days = 0u32; // ← Zero: immediate release

    // Create escrow with guarantee_days = 0
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Verify escrow was created successfully
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Active);
    assert_eq!(escrow.guarantee_days, 0);

    // Verify release_at equals current timestamp (or very close)
    let now = env.ledger().timestamp();
    assert_eq!(escrow.release_at, now);

    // Mock auths and release immediately
    env.mock_all_auths();
    client.release_payment(&escrow_id, &seller, &0);

    // Verify payment was released
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Released);
}

#[test]
#[should_panic(expected = "GuaranteePeriodExpired")]
fn test_request_refund_blocked_with_zero_guarantee_days() {
    let (env, contract_id, token) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 0u32; // ← Zero: immediate release

    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Trying request_refund should fail because release_at = now (period expired)
    client.request_refund(&escrow_id, &buyer, &0);
}
