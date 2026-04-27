#![no_std]

mod error;
mod storage;

#[cfg(test)]
mod test;

use error::MultisigError;
use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contractimpl, contracttype,
    crypto::Hash,
    BytesN, Env, Vec,
};
use storage::{
    read_signers, read_threshold, write_signers, write_threshold, MAX_SIGNERS,
};

/// A single Ed25519 signature from one of the registered signers.
#[contracttype]
#[derive(Clone)]
pub struct AccountSignature {
    pub public_key: BytesN<32>,
    pub signature: BytesN<64>,
}

#[contract]
pub struct MultisigContract;

#[contractimpl]
impl MultisigContract {
    /// Initialize the multisig with an ordered list of Ed25519 public keys and a threshold.
    ///
    /// `signers` must be sorted in strictly ascending order and contain no duplicates.
    /// `threshold` must be in [1, signers.len()].
    pub fn __constructor(
        env: Env,
        signers: Vec<BytesN<32>>,
        threshold: u32,
    ) -> Result<(), MultisigError> {
        if env.storage().instance().has(&storage::DataKey::Signers) {
            return Err(MultisigError::AlreadyInitialized);
        }
        Self::_validate_and_store_signers(&env, &signers, threshold)?;
        Ok(())
    }

    /// Add a new signer public key. The multisig itself must authorize this call.
    pub fn add_signer(env: Env, new_key: BytesN<32>) -> Result<(), MultisigError> {
        env.current_contract_address().require_auth();

        let mut signers = read_signers(&env);

        if signers.len() >= MAX_SIGNERS {
            return Err(MultisigError::TooManySigners);
        }

        // Reject duplicates
        for i in 0..signers.len() {
            if signers.get_unchecked(i) == new_key {
                return Err(MultisigError::SignerAlreadyExists);
            }
        }

        signers.push_back(new_key);
        write_signers(&env, &signers);
        Self::_extend_ttl(&env);
        Ok(())
    }

    /// Remove a signer public key. The multisig itself must authorize this call.
    pub fn remove_signer(env: Env, key: BytesN<32>) -> Result<(), MultisigError> {
        env.current_contract_address().require_auth();

        let signers = read_signers(&env);
        let threshold = read_threshold(&env);

        if signers.len() <= threshold {
            return Err(MultisigError::ThresholdExceedsSigners);
        }

        let mut new_signers: Vec<BytesN<32>> = Vec::new(&env);
        let mut found = false;
        for i in 0..signers.len() {
            let s = signers.get_unchecked(i);
            if s == key {
                found = true;
            } else {
                new_signers.push_back(s);
            }
        }

        if !found {
            return Err(MultisigError::SignerNotFound);
        }

        write_signers(&env, &new_signers);
        Self::_extend_ttl(&env);
        Ok(())
    }

    /// Update the approval threshold. The multisig itself must authorize this call.
    pub fn set_threshold(env: Env, threshold: u32) -> Result<(), MultisigError> {
        env.current_contract_address().require_auth();

        let signers = read_signers(&env);
        if threshold == 0 || threshold > signers.len() {
            return Err(MultisigError::InvalidThreshold);
        }

        write_threshold(&env, threshold);
        Self::_extend_ttl(&env);
        Ok(())
    }

    /// Renew the contract TTL so the account stays live.
    pub fn bump(env: Env) {
        Self::_extend_ttl(&env);
    }

    /// Returns the current list of registered signer public keys.
    pub fn get_signers(env: Env) -> Vec<BytesN<32>> {
        read_signers(&env)
    }

    /// Returns the current threshold.
    pub fn get_threshold(env: Env) -> u32 {
        read_threshold(&env)
    }

    // --- helpers ---

    fn _validate_and_store_signers(
        env: &Env,
        signers: &Vec<BytesN<32>>,
        threshold: u32,
    ) -> Result<(), MultisigError> {
        let n = signers.len();

        if n == 0 || threshold == 0 || threshold > n {
            return Err(MultisigError::InvalidThreshold);
        }
        if n > MAX_SIGNERS {
            return Err(MultisigError::TooManySigners);
        }

        // Enforce strictly ascending order → also rejects duplicates
        for i in 1..n {
            if signers.get_unchecked(i) <= signers.get_unchecked(i - 1) {
                return Err(MultisigError::SignerAlreadyExists);
            }
        }

        write_signers(env, signers);
        write_threshold(env, threshold);
        Self::_extend_ttl(env);
        Ok(())
    }

    fn _extend_ttl(env: &Env) {
        const LEDGERS_30_DAYS: u32 = 30 * 24 * 60 * 60 / 5; // ~518_400
        const LEDGERS_7_DAYS: u32 = 7 * 24 * 60 * 60 / 5;   // ~120_960
        env.storage()
            .instance()
            .extend_ttl(LEDGERS_7_DAYS, LEDGERS_30_DAYS);
    }
}

// ---------------------------------------------------------------------------
// CustomAccountInterface — called by the Soroban host when this contract
// address is used as an authorization source (i.e., when it is the admin).
// ---------------------------------------------------------------------------
impl CustomAccountInterface for MultisigContract {
    type Error = MultisigError;
    /// Expects a `Vec<AccountSignature>` sorted in strictly ascending order
    /// by `public_key`. Every public_key in the list MUST be a registered signer.
    type Signature = Vec<AccountSignature>;

    #[allow(non_snake_case)]
    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signatures: Vec<AccountSignature>,
        _auth_contexts: Vec<Context>,
    ) -> Result<(), MultisigError> {
        let signers = read_signers(&env);
        let threshold = read_threshold(&env);

        // Convert Hash<32> → Bytes for ed25519_verify
        let payload_bytes: soroban_sdk::Bytes =
            Into::<BytesN<32>>::into(signature_payload).into();

        let mut valid_count: u32 = 0;

        // Walk signature list and verify each one.
        // We simultaneously enforce ascending order (→ no duplicate counting)
        // and that every key is a registered signer.
        let mut last_key: Option<BytesN<32>> = None;

        for i in 0..signatures.len() {
            let sig = signatures.get_unchecked(i);

            // Enforce strictly ascending public key order
            if let Some(ref prev) = last_key {
                if sig.public_key <= *prev {
                    return Err(MultisigError::InvalidSignatureOrder);
                }
            }
            last_key = Some(sig.public_key.clone());

            // Reject signatures from unknown signers
            let mut is_known = false;
            for j in 0..signers.len() {
                if signers.get_unchecked(j) == sig.public_key {
                    is_known = true;
                    break;
                }
            }
            if !is_known {
                return Err(MultisigError::UnknownSigner);
            }

            // Verify the Ed25519 signature. Panics (trap) on invalid signature.
            env.crypto()
                .ed25519_verify(&sig.public_key, &payload_bytes, &sig.signature);

            valid_count += 1;
        }

        if valid_count < threshold {
            return Err(MultisigError::InsufficientSignatures);
        }

        Ok(())
    }
}
