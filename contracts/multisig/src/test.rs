#![cfg(test)]

extern crate std;

use super::*;
use ed25519_dalek::{Signer as DalekSigner, SigningKey};
use rand::rngs::OsRng;
use soroban_sdk::{Bytes, BytesN, Env, Vec};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct Keypair {
    signing_key: SigningKey,
    pub_key: [u8; 32],
}

impl Keypair {
    fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let pub_key = signing_key.verifying_key().to_bytes();
        Keypair { signing_key, pub_key }
    }

    fn pub_key_bytes(&self, env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &self.pub_key)
    }

    fn sign(&self, payload: &[u8; 32]) -> [u8; 64] {
        self.signing_key.sign(payload).to_bytes()
    }
}

fn make_account_sig(env: &Env, kp: &Keypair, payload: &[u8; 32]) -> AccountSignature {
    AccountSignature {
        public_key: kp.pub_key_bytes(env),
        signature: BytesN::from_array(env, &kp.sign(payload)),
    }
}

fn sort_keypairs(kps: &mut [&Keypair]) {
    kps.sort_by(|a, b| a.pub_key.cmp(&b.pub_key));
}

fn register_contract(
    env: &Env,
    signers: Vec<BytesN<32>>,
    threshold: u32,
) -> MultisigContractClient {
    let addr = env.register(MultisigContract, (signers, threshold));
    MultisigContractClient::new(env, &addr)
}

/// Generate a random Hash<32> using the soroban crypto SHA256 API.
/// Returns (Hash<32>, raw [u8;32]) so callers can sign the raw bytes.
fn random_hash_payload(env: &Env) -> (soroban_sdk::crypto::Hash<32>, [u8; 32]) {
    use rand::RngCore;
    let mut random_input = [0u8; 64];
    OsRng.fill_bytes(&mut random_input);
    let input_bytes = Bytes::from_slice(env, &random_input);
    let hash = env.crypto().sha256(&input_bytes);
    let raw: [u8; 32] = hash.clone().into();
    (hash, raw)
}

fn call_check_auth(
    env: &Env,
    contract_addr: &soroban_sdk::Address,
    hash: soroban_sdk::crypto::Hash<32>,
    sigs: Vec<AccountSignature>,
) -> Result<(), MultisigError> {
    use soroban_sdk::auth::CustomAccountInterface;
    env.as_contract(contract_addr, || {
        MultisigContract::__check_auth(env.clone(), hash, sigs, Vec::new(env))
    })
}

// ---------------------------------------------------------------------------
// Constructor tests
// ---------------------------------------------------------------------------

#[test]
fn test_constructor_success() {
    let env = Env::default();
    let mut kps: std::vec::Vec<Keypair> = (0..3).map(|_| Keypair::generate()).collect();
    kps.sort_by(|a, b| a.pub_key.cmp(&b.pub_key));

    let mut signers = Vec::new(&env);
    for kp in &kps {
        signers.push_back(kp.pub_key_bytes(&env));
    }

    let client = register_contract(&env, signers.clone(), 2);
    assert_eq!(client.get_threshold(), 2);
    assert_eq!(client.get_signers(), signers);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_constructor_threshold_zero() {
    let env = Env::default();
    let kp = Keypair::generate();
    let mut signers = Vec::new(&env);
    signers.push_back(kp.pub_key_bytes(&env));
    register_contract(&env, signers, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_constructor_threshold_exceeds_signers() {
    let env = Env::default();
    let kp = Keypair::generate();
    let mut signers = Vec::new(&env);
    signers.push_back(kp.pub_key_bytes(&env));
    register_contract(&env, signers, 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_constructor_duplicate_signers() {
    let env = Env::default();
    let kp = Keypair::generate();
    let pk = kp.pub_key_bytes(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(pk.clone());
    signers.push_back(pk);
    register_contract(&env, signers, 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_constructor_too_many_signers() {
    let env = Env::default();
    let mut kps: std::vec::Vec<Keypair> = (0..21).map(|_| Keypair::generate()).collect();
    kps.sort_by(|a, b| a.pub_key.cmp(&b.pub_key));
    let mut signers = Vec::new(&env);
    for kp in &kps {
        signers.push_back(kp.pub_key_bytes(&env));
    }
    register_contract(&env, signers, 1);
}

// ---------------------------------------------------------------------------
// Management tests
// ---------------------------------------------------------------------------

#[test]
fn test_add_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let kp1 = Keypair::generate();
    let kp2 = Keypair::generate();
    let mut signers = Vec::new(&env);
    signers.push_back(kp1.pub_key_bytes(&env));

    let client = register_contract(&env, signers, 1);
    client.add_signer(&kp2.pub_key_bytes(&env));
    assert_eq!(client.get_signers().len(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_add_signer_duplicate() {
    let env = Env::default();
    env.mock_all_auths();

    let kp = Keypair::generate();
    let mut signers = Vec::new(&env);
    signers.push_back(kp.pub_key_bytes(&env));

    let client = register_contract(&env, signers, 1);
    client.add_signer(&kp.pub_key_bytes(&env));
}

#[test]
fn test_remove_signer() {
    let env = Env::default();
    env.mock_all_auths();

    let kp1 = Keypair::generate();
    let kp2 = Keypair::generate();
    let mut kp_refs = [&kp1, &kp2];
    sort_keypairs(&mut kp_refs);
    let mut signers = Vec::new(&env);
    for kp in &kp_refs {
        signers.push_back(kp.pub_key_bytes(&env));
    }

    let client = register_contract(&env, signers, 1);
    client.remove_signer(&kp1.pub_key_bytes(&env));
    assert_eq!(client.get_signers().len(), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_remove_signer_would_breach_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let kp = Keypair::generate();
    let mut signers = Vec::new(&env);
    signers.push_back(kp.pub_key_bytes(&env));

    let client = register_contract(&env, signers, 1);
    client.remove_signer(&kp.pub_key_bytes(&env));
}

#[test]
fn test_set_threshold() {
    let env = Env::default();
    env.mock_all_auths();

    let kp1 = Keypair::generate();
    let kp2 = Keypair::generate();
    let mut kp_refs = [&kp1, &kp2];
    sort_keypairs(&mut kp_refs);
    let mut signers = Vec::new(&env);
    for kp in &kp_refs {
        signers.push_back(kp.pub_key_bytes(&env));
    }

    let client = register_contract(&env, signers, 1);
    client.set_threshold(&2);
    assert_eq!(client.get_threshold(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_set_threshold_too_high() {
    let env = Env::default();
    env.mock_all_auths();

    let kp = Keypair::generate();
    let mut signers = Vec::new(&env);
    signers.push_back(kp.pub_key_bytes(&env));

    let client = register_contract(&env, signers, 1);
    client.set_threshold(&5);
}

// ---------------------------------------------------------------------------
// __check_auth — real Ed25519 signature tests
// ---------------------------------------------------------------------------

fn setup_multisig(
    env: &Env,
    n: usize,
    threshold: u32,
) -> (MultisigContractClient, std::vec::Vec<Keypair>) {
    let mut kps: std::vec::Vec<Keypair> = (0..n).map(|_| Keypair::generate()).collect();
    kps.sort_by(|a, b| a.pub_key.cmp(&b.pub_key));

    let mut signers = Vec::new(env);
    for kp in &kps {
        signers.push_back(kp.pub_key_bytes(env));
    }

    let client = register_contract(env, signers, threshold);
    (client, kps)
}

#[test]
fn test_check_auth_1_of_1() {
    let env = Env::default();
    let (client, kps) = setup_multisig(&env, 1, 1);

    let (hash, raw) = random_hash_payload(&env);
    let mut sigs = Vec::new(&env);
    sigs.push_back(make_account_sig(&env, &kps[0], &raw));

    call_check_auth(&env, &client.address, hash, sigs).unwrap();
}

#[test]
fn test_check_auth_2_of_3() {
    let env = Env::default();
    let (client, kps) = setup_multisig(&env, 3, 2);

    let (hash, raw) = random_hash_payload(&env);
    let mut sigs = Vec::new(&env);
    sigs.push_back(make_account_sig(&env, &kps[0], &raw));
    sigs.push_back(make_account_sig(&env, &kps[1], &raw));

    call_check_auth(&env, &client.address, hash, sigs).unwrap();
}

#[test]
fn test_check_auth_insufficient_signatures() {
    let env = Env::default();
    let (client, kps) = setup_multisig(&env, 3, 2);

    let (hash, raw) = random_hash_payload(&env);
    let mut sigs = Vec::new(&env);
    sigs.push_back(make_account_sig(&env, &kps[0], &raw));

    let result = call_check_auth(&env, &client.address, hash, sigs);
    assert_eq!(result, Err(MultisigError::InsufficientSignatures));
}

#[test]
#[should_panic]
fn test_check_auth_wrong_signature() {
    let env = Env::default();
    let (client, kps) = setup_multisig(&env, 1, 1);

    let (hash, _raw) = random_hash_payload(&env);
    let (_hash2, raw2) = random_hash_payload(&env);

    // Sign a different payload — ed25519_verify panics (traps) on bad sig
    let mut sigs = Vec::new(&env);
    sigs.push_back(make_account_sig(&env, &kps[0], &raw2));

    call_check_auth(&env, &client.address, hash, sigs).unwrap();
}

#[test]
fn test_check_auth_unknown_signer() {
    let env = Env::default();
    let (client, _kps) = setup_multisig(&env, 1, 1);

    let (hash, raw) = random_hash_payload(&env);
    let stranger = Keypair::generate();
    let mut sigs = Vec::new(&env);
    sigs.push_back(make_account_sig(&env, &stranger, &raw));

    let result = call_check_auth(&env, &client.address, hash, sigs);
    assert_eq!(result, Err(MultisigError::UnknownSigner));
}

#[test]
fn test_check_auth_out_of_order() {
    let env = Env::default();
    let (client, kps) = setup_multisig(&env, 2, 2);

    let (hash, raw) = random_hash_payload(&env);
    let mut sigs = Vec::new(&env);
    // Reverse order
    sigs.push_back(make_account_sig(&env, &kps[1], &raw));
    sigs.push_back(make_account_sig(&env, &kps[0], &raw));

    let result = call_check_auth(&env, &client.address, hash, sigs);
    assert_eq!(result, Err(MultisigError::InvalidSignatureOrder));
}

#[test]
fn test_check_auth_duplicate_sig() {
    let env = Env::default();
    let (client, kps) = setup_multisig(&env, 2, 2);

    let (hash, raw) = random_hash_payload(&env);
    let mut sigs = Vec::new(&env);
    // Same key twice — not strictly ascending
    sigs.push_back(make_account_sig(&env, &kps[0], &raw));
    sigs.push_back(make_account_sig(&env, &kps[0], &raw));

    let result = call_check_auth(&env, &client.address, hash, sigs);
    assert_eq!(result, Err(MultisigError::InvalidSignatureOrder));
}
