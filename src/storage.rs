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
