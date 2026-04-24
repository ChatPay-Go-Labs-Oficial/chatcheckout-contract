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
    // (includes allow_early_release, buyer_proposal, seller_proposal)
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
        fee_bps: 400,
        allow_early_release: false,
        buyer_proposal: None,
        seller_proposal: None,
    };

    // If it compiles, it's correct
    assert_eq!(escrow.fee_bps, 400);
    assert_eq!(escrow.guarantee_days, 7);
    assert_eq!(escrow.allow_early_release, false);
    assert_eq!(escrow.buyer_proposal, None);
    assert_eq!(escrow.seller_proposal, None);
}

#[test]
fn test_escrow_status_enum() {
    // Verify EscrowStatus enum works correctly (with Disputed)
    let status1 = EscrowStatus::Active;
    let status2 = EscrowStatus::Released;
    let status3 = EscrowStatus::Refunded;
    let status4 = EscrowStatus::Disputed;

    assert_eq!(status1, EscrowStatus::Active);
    assert_eq!(status2, EscrowStatus::Released);
    assert_eq!(status3, EscrowStatus::Refunded);
    assert_eq!(status4, EscrowStatus::Disputed);
}

// ============================================================================
// Helper Functions for Tests
// ============================================================================

/// Helper function to set up test environment with contract and token
fn setup_contract_with_token() -> (Env, Address, soroban_sdk::token::StellarAssetClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Register SAC (Stellar Asset Contract)
    let _sac_contract = env.register_stellar_asset_contract_v2(admin.clone());
    // Token contract address is same as admin for SAC
    let token = soroban_sdk::token::StellarAssetClient::new(&env, &admin);

    // Register escrow contract - __constructor will be called automatically
    let contract_id = env.register(EscrowContract, (&admin, &false));

    (env, contract_id, token, admin)
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
    client.create_escrow(buyer, seller, &amount, &token_address, &fee_bps, &guarantee_days, &product_id, &false)
}

// ============================================================================
// Nonce Tests
// ============================================================================

#[test]
fn test_get_nonce_initial_zero() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    // Initial nonce should be 0
    let nonce = client.get_nonce(&user);
    assert_eq!(nonce, 0);
}

// ============================================================================
// Guarantee Days Validation Tests
// ============================================================================

#[test]
fn test_validate_guarantee_days_zero_is_rejected() {
    // guarantee_days = 0 should be rejected (minimum 1 day required)
    let result = validation::validate_guarantee_days(0);
    assert!(matches!(result, Err(EscrowError::InvalidGuaranteeDays)));
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

// ============================================================================
// Early Release Tests
// ============================================================================

#[test]
fn test_release_payment_with_early_release_allowed() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    // Create escrow with allow_early_release = true
    let token_address = token.address.clone();
    let product_id = soroban_sdk::String::from_str(&env, "product-123");
    token.mint(&buyer, &amount);
    client.create_escrow(&buyer, &seller, &amount, &token_address, &fee_bps, &guarantee_days, &product_id, &true);

    // Get the escrow_id (should be 1)
    let escrow_id = 1u64;

    // Mock auths and try to release immediately (before guarantee period)
    env.mock_all_auths();
    client.release_payment(&escrow_id, &seller, &0);

    // Verify payment was released (should succeed with early release)
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Released);
}

#[test]
#[should_panic(expected = "GuaranteePeriodNotExpired")]
fn test_release_payment_blocked_without_early_release() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    // Create escrow with allow_early_release = false (default)
    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths and try to release immediately (before guarantee period)
    env.mock_all_auths();
    client.release_payment(&escrow_id, &seller, &0);

    // Should fail with GuaranteePeriodNotExpired
}

#[test]
fn test_create_escrow_with_early_release_true() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    // Create escrow with allow_early_release = true
    let token_address = token.address.clone();
    let product_id = soroban_sdk::String::from_str(&env, "product-123");
    token.mint(&buyer, &amount);
    client.create_escrow(&buyer, &seller, &amount, &token_address, &fee_bps, &guarantee_days, &product_id, &true);

    // Get the escrow_id (should be 1)
    let escrow_id = 1u64;

    // Verify escrow stores allow_early_release correctly
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.allow_early_release, true);
}

#[test]
fn test_create_escrow_with_early_release_false() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    // Create escrow with allow_early_release = false
    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Verify escrow stores allow_early_release correctly
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.allow_early_release, false);
}

// ============================================================================
// Dispute Resolution Tests
// ============================================================================

#[test]
fn test_dispute_escrow_by_buyer() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths and initiate dispute
    env.mock_all_auths();
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Verify status changed to Disputed
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Disputed);
}

#[test]
fn test_dispute_escrow_by_seller() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths and initiate dispute
    env.mock_all_auths();
    client.dispute_escrow(&escrow_id, &seller, &0);

    // Verify status changed to Disputed
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Disputed);
}

#[test]
#[should_panic(expected = "AlreadyDisputed")]
fn test_dispute_escrow_already_disputed() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Initiate dispute first time
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Try to dispute again - should fail
    client.dispute_escrow(&escrow_id, &seller, &1);
}

#[test]
fn test_propose_resolution_buyer_favors_themselves() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Initiate dispute
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Buyer proposes resolution favoring themselves
    client.propose_resolution(&escrow_id, &buyer, &1, &true);

    // Verify proposal was stored
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.buyer_proposal, Some(true));
}

#[test]
fn test_propose_resolution_both_agree_on_refund() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Initiate dispute
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Both parties propose refund (favor_buyer = true)
    client.propose_resolution(&escrow_id, &buyer, &1, &true);
    client.propose_resolution(&escrow_id, &seller, &0, &true);

    // Resolve dispute
    client.resolve_dispute(&escrow_id, &buyer, &2);

    // Verify escrow was refunded
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Refunded);
}

#[test]
fn test_propose_resolution_both_agree_on_release() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Initiate dispute
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Both parties propose release (favor_buyer = false)
    client.propose_resolution(&escrow_id, &buyer, &1, &false);
    client.propose_resolution(&escrow_id, &seller, &0, &false);

    // Resolve dispute
    client.resolve_dispute(&escrow_id, &seller, &1);

    // Verify escrow was released
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Released);
}

#[test]
#[should_panic(expected = "BothPartiesMustAgree")]
fn test_resolve_dispute_parties_disagree() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Initiate dispute
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Parties disagree: buyer wants refund, seller wants release
    client.propose_resolution(&escrow_id, &buyer, &1, &true);
    client.propose_resolution(&escrow_id, &seller, &0, &false);

    // Try to resolve - should fail
    client.resolve_dispute(&escrow_id, &buyer, &2);
}

#[test]
fn test_admin_resolve_dispute_when_parties_disagree() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Initiate dispute
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Parties disagree: buyer wants refund, seller wants release
    client.propose_resolution(&escrow_id, &buyer, &1, &true);
    client.propose_resolution(&escrow_id, &seller, &0, &false);

    // Admin resolves in favor of seller
    client.admin_resolve_dispute(&escrow_id, &false, &0);

    // Verify escrow was released
    let escrow = client.get_escrow(&escrow_id);
    assert_eq!(escrow.status, EscrowStatus::Released);
}

#[test]
#[should_panic(expected = "NoDisputeToResolve")]
fn test_resolve_dispute_before_both_proposed() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Initiate dispute
    client.dispute_escrow(&escrow_id, &buyer, &0);

    // Only buyer proposes
    client.propose_resolution(&escrow_id, &buyer, &1, &true);

    // Try to resolve before both parties propose - should fail
    client.resolve_dispute(&escrow_id, &buyer, &2);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_dispute_by_unauthorized_party() {
    let (env, contract_id, token, token_admin) = setup_contract_with_token();
    let client = EscrowContractClient::new(&env, &contract_id);

    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let third_party = Address::generate(&env);
    let amount = 1000i128;
    let fee_bps = 400u32;
    let guarantee_days = 7u32;

    env.mock_all_auths();
    let escrow_id = create_test_escrow(&env, &contract_id, &token, &buyer, &seller, amount, fee_bps, guarantee_days);

    // Mock auths
    env.mock_all_auths();

    // Third party tries to initiate dispute - should fail
    client.dispute_escrow(&escrow_id, &third_party, &0);
}

#[test]
fn test_escrow_status_enum_includes_disputed() {
    // Verify EscrowStatus enum includes Disputed
    let status = EscrowStatus::Disputed;
    assert_eq!(status, EscrowStatus::Disputed);
}
