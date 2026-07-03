use soroban_sdk::{symbol_short, Address, Env, Vec};

use crate::types::ProposalType;

pub fn board_initialized(env: &Env, members: &Vec<Address>, threshold: u32, timestamp: u64) {
    env.events().publish(
        (symbol_short!("board"), symbol_short!("init")),
        (members.clone(), threshold, timestamp),
    );
}

pub fn proposal_created(
    env: &Env,
    proposal_id: u64,
    proposer: &Address,
    proposal_type: &ProposalType,
    expires_at: u64,
) {
    env.events().publish(
        (symbol_short!("proposal"), symbol_short!("created")),
        (
            proposal_id,
            proposer.clone(),
            proposal_type.clone(),
            expires_at,
        ),
    );
}

pub fn proposal_signed(
    env: &Env,
    proposal_id: u64,
    signer: &Address,
    signatures_count: u32,
    threshold: u32,
) {
    env.events().publish(
        (symbol_short!("proposal"), symbol_short!("signed")),
        (proposal_id, signer.clone(), signatures_count, threshold),
    );
}

pub fn proposal_executed(
    env: &Env,
    proposal_id: u64,
    proposal_type: &ProposalType,
    executor: &Address,
    timestamp: u64,
) {
    env.events().publish(
        (symbol_short!("proposal"), symbol_short!("executed")),
        (
            proposal_id,
            proposal_type.clone(),
            executor.clone(),
            timestamp,
        ),
    );
}

pub fn proposal_cancelled(env: &Env, proposal_id: u64, canceller: &Address, timestamp: u64) {
    env.events().publish(
        (symbol_short!("proposal"), symbol_short!("cancelled")),
        (proposal_id, canceller.clone(), timestamp),
    );
}

pub fn proposal_expired(env: &Env, proposal_id: u64, timestamp: u64) {
    env.events().publish(
        (symbol_short!("proposal"), symbol_short!("expired")),
        (proposal_id, timestamp),
    );
}

pub fn board_updated(env: &Env, members_count: u32, threshold: u32, timestamp: u64) {
    env.events().publish(
        (symbol_short!("board"), symbol_short!("updated")),
        (members_count, threshold, timestamp),
    );
}

pub fn deposit_received(env: &Env, from: &Address, amount: i128, asset: &Address, timestamp: u64) {
    env.events().publish(
        (symbol_short!("treasury"), symbol_short!("deposit")),
        (from.clone(), amount, asset.clone(), timestamp),
    );
}
