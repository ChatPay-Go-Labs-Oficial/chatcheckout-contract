use soroban_sdk::{contracttype, BytesN, Env, Vec};

/// Maximum allowed registered signers (prevents unbounded Vec in instance storage)
pub const MAX_SIGNERS: u32 = 20;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Signers,
    Threshold,
}

pub fn read_signers(env: &Env) -> Vec<BytesN<32>> {
    env.storage()
        .instance()
        .get(&DataKey::Signers)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn write_signers(env: &Env, signers: &Vec<BytesN<32>>) {
    env.storage().instance().set(&DataKey::Signers, signers);
}

pub fn read_threshold(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::Threshold)
        .unwrap_or(1u32)
}

pub fn write_threshold(env: &Env, threshold: u32) {
    env.storage()
        .instance()
        .set(&DataKey::Threshold, &threshold);
}
