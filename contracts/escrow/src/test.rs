#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger as _, LedgerInfo};
use soroban_sdk::{Address, Env, String};

/// Helper function para avançar o tempo em X dias
fn advance_time(env: &Env, days: u64) {
    let current = env.ledger().timestamp();
    let new_timestamp = current + (days * 86_400); // 86_400 segundos = 1 dia

    env.ledger().set(LedgerInfo {
        timestamp: new_timestamp,
        protocol_version: 22, // Atualizado para versão compatível com o host
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
    // Teste unitário para verificar que calc_fee está funcionando corretamente
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

    // Verificar tempo inicial
    let initial_time = env.ledger().timestamp();
    assert_eq!(initial_time, 0);

    // Avançar 7 dias
    advance_time(&env, 7);

    // Verificar que o tempo avançou
    let new_time = env.ledger().timestamp();
    assert_eq!(new_time, 7 * 86_400); // 7 dias em segundos
}

#[test]
fn test_config_struct_refactored() {
    // Verificar que a struct Config foi refatorada corretamente
    // (apenas admin e collect_on_create, sem fee_bps e flat_fee)
    let env = Env::default();
    let admin = Address::generate(&env);

    let cfg = Config {
        admin: admin.clone(),
        collect_on_create: true,
    };

    // Se compila, está correto
    assert_eq!(cfg.admin, admin);
    assert_eq!(cfg.collect_on_create, true);
}

#[test]
fn test_escrow_data_struct_refactored() {
    // Verificar que a struct EscrowData foi refatorada corretamente
    // (apenas fee_bps, sem flat_fee e fee_collected_on_create)
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
        fee_bps: 400, // Apenas fee_bps, sem flat_fee e fee_collected_on_create
    };

    // Se compila, está correto
    assert_eq!(escrow.fee_bps, 400);
    assert_eq!(escrow.guarantee_days, 7);
}

#[test]
fn test_escrow_status_enum() {
    // Verificar que o enum EscrowStatus funciona corretamente
    let status1 = EscrowStatus::Active;
    let status2 = EscrowStatus::Released;
    let status3 = EscrowStatus::Refunded;
    let status4 = EscrowStatus::Disputed;

    assert_eq!(status1, EscrowStatus::Active);
    assert_eq!(status2, EscrowStatus::Released);
    assert_eq!(status3, EscrowStatus::Refunded);
    assert_eq!(status4, EscrowStatus::Disputed);
}
