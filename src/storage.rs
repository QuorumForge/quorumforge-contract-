use soroban_sdk::{Address, Env};

use crate::types::{BoardConfig, DataKey, Proposal};

pub fn has_board(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Board)
}

pub fn get_board(env: &Env) -> BoardConfig {
    env.storage().instance().get(&DataKey::Board).unwrap()
}

pub fn set_board(env: &Env, board: &BoardConfig) {
    env.storage().instance().set(&DataKey::Board, board);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn has_proposal(env: &Env, id: u64) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Proposal(id))
}

pub fn get_proposal(env: &Env, id: u64) -> Proposal {
    env.storage()
        .persistent()
        .get(&DataKey::Proposal(id))
        .unwrap()
}

pub fn set_proposal(env: &Env, proposal: &Proposal) {
    env.storage()
        .persistent()
        .set(&DataKey::Proposal(proposal.proposal_id), proposal);
}

pub fn get_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::Count)
        .unwrap_or(0u64)
}

pub fn increment_count(env: &Env) -> u64 {
    let next = get_count(env) + 1;
    env.storage().instance().set(&DataKey::Count, &next);
    next
}

/// Returns `true` if the board has been initialized and an admin is set.
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// Extends the TTL of a proposal's persistent storage entry.
/// Call this after reading a proposal to prevent it from being evicted.
pub fn extend_proposal_ttl(env: &Env, id: u64, ledgers_to_extend: u32) {
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Proposal(id), ledgers_to_extend, ledgers_to_extend);
}

/// Extends the TTL of the instance storage (board config, admin, count).
pub fn extend_instance_ttl(env: &Env, ledgers_to_extend: u32) {
    env.storage()
        .instance()
        .extend_ttl(ledgers_to_extend, ledgers_to_extend);
}
