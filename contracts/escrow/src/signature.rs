/// Signature verification and nonce management for meta-transactions
use crate::error::EscrowError;
use crate::storage::{increment_nonce, read_nonce};
use soroban_sdk::{Address, Env};

/// Verify nonce and increment for replay protection
///
/// This function checks that the expected nonce matches the current nonce
/// for the user, then increments the nonce. This prevents replay attacks
/// where the same signed transaction could be submitted multiple times.
///
/// # Arguments
/// * `env` - The Soroban environment
/// * `signer` - The address of the user who signed the transaction
/// * `expected_nonce` - The nonce value that was used when creating the signature
///
/// # Returns
/// * `Ok(())` if the nonce is valid and was incremented
/// * `Err(EscrowError::InvalidNonce)` if the nonce doesn't match
pub fn verify_and_increment_nonce(
    env: &Env,
    signer: &Address,
    expected_nonce: u64,
) -> Result<(), EscrowError> {
    let current_nonce = read_nonce(env, signer);
    if expected_nonce != current_nonce {
        return Err(EscrowError::InvalidNonce);
    }
    increment_nonce(env, signer);
    Ok(())
}
