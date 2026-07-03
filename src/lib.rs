#![no_std]

mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};

use crate::{
    storage::{
        get_admin, get_board, get_count, get_proposal, has_board, increment_count, set_admin,
        set_board, set_proposal,
    },
    types::{BoardConfig, Proposal, ProposalPayload, ProposalStatus, ProposalType, Stats},
};

const SEVEN_DAYS_SECS: u64 = 7 * 24 * 60 * 60;
const MIN_TTL_SECS: u64 = 60 * 60; // 1 hour minimum
const MAX_TTL_SECS: u64 = 30 * 24 * 60 * 60; // 30 days maximum

fn is_member(members: &Vec<Address>, addr: &Address) -> bool {
    members.contains(addr)
}

fn now(env: &Env) -> u64 {
    env.ledger().timestamp()
}

#[contract]
pub struct QuorumForge;

#[contractimpl]
impl QuorumForge {
    // ── Initialize ────────────────────────────────────────────────────────────

    pub fn initialize(env: Env, admin: Address, members: Vec<Address>, threshold: u32) {
        assert!(!has_board(&env), "already initialized");
        assert!(!members.is_empty(), "members cannot be empty");
        assert!(
            threshold as usize <= members.len() as usize && threshold > 0,
            "invalid threshold"
        );

        admin.require_auth();

        let ts = now(&env);
        let board = BoardConfig {
            members: members.clone(),
            threshold,
            created_at: ts,
        };
        set_board(&env, &board);
        set_admin(&env, &admin);
        env.storage()
            .instance()
            .set(&crate::types::DataKey::Count, &0u64);

        events::board_initialized(&env, &members, threshold, ts);
    }

    // ── Create Proposal ───────────────────────────────────────────────────────

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        payload: ProposalPayload,
        ttl_seconds: Option<u64>,
    ) -> u64 {
        proposer.require_auth();

        let board = get_board(&env);
        assert!(is_member(&board.members, &proposer), "not a board member");

        let ts = now(&env);
        let ttl = ttl_seconds.unwrap_or(SEVEN_DAYS_SECS);
        assert!(ttl >= MIN_TTL_SECS, "ttl too short");
        assert!(ttl <= MAX_TTL_SECS, "ttl too long");
        let expires_at = ts + ttl;
        let proposal_id = increment_count(&env);

        let proposal = Proposal {
            proposal_id,
            proposer: proposer.clone(),
            proposal_type: proposal_type.clone(),
            payload,
            signatures: Vec::new(&env),
            status: ProposalStatus::Pending,
            created_at: ts,
            expires_at,
            executed_at: None,
        };
        set_proposal(&env, &proposal);

        events::proposal_created(&env, proposal_id, &proposer, &proposal_type, expires_at);
        proposal_id
    }

    // ── Sign Proposal ─────────────────────────────────────────────────────────

    pub fn sign_proposal(env: Env, signer: Address, proposal_id: u64) {
        signer.require_auth();

        let board = get_board(&env);
        assert!(is_member(&board.members, &signer), "not a board member");

        let mut proposal = get_proposal(&env, proposal_id);
        assert!(
            proposal.status == ProposalStatus::Pending,
            "proposal not pending"
        );
        assert!(now(&env) <= proposal.expires_at, "proposal expired");
        assert!(!proposal.signatures.contains(&signer), "already signed");

        proposal.signatures.push_back(signer.clone());
        let sig_count = proposal.signatures.len();
        set_proposal(&env, &proposal);

        events::proposal_signed(&env, proposal_id, &signer, sig_count, board.threshold);

        if sig_count >= board.threshold {
            Self::_execute(&env, proposal_id);
        }
    }

    // ── Execute Proposal (public) ─────────────────────────────────────────────

    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let board = get_board(&env);
        let proposal = get_proposal(&env, proposal_id);
        assert!(
            proposal.status == ProposalStatus::Pending,
            "proposal not pending"
        );
        assert!(
            proposal.signatures.len() >= board.threshold,
            "threshold not met"
        );
        Self::_execute(&env, proposal_id);
    }

    // ── Cancel Proposal ───────────────────────────────────────────────────────

    pub fn cancel_proposal(env: Env, proposal_id: u64, canceller: Address) {
        canceller.require_auth();

        let admin = get_admin(&env);
        let mut proposal = get_proposal(&env, proposal_id);
        assert!(
            canceller == proposal.proposer || canceller == admin,
            "unauthorized"
        );
        assert!(
            proposal.status == ProposalStatus::Pending,
            "proposal not pending"
        );

        proposal.status = ProposalStatus::Cancelled;
        set_proposal(&env, &proposal);

        let ts = now(&env);
        events::proposal_cancelled(&env, proposal_id, &canceller, ts);
    }

    // ── Expire Proposal ───────────────────────────────────────────────────────

    pub fn expire_proposal(env: Env, proposal_id: u64) {
        let mut proposal = get_proposal(&env, proposal_id);
        let ts = now(&env);
        assert!(
            proposal.status == ProposalStatus::Pending,
            "proposal not pending"
        );
        assert!(ts > proposal.expires_at, "not yet expired");

        proposal.status = ProposalStatus::Expired;
        set_proposal(&env, &proposal);

        events::proposal_expired(&env, proposal_id, ts);
    }

    // ── Deposit ───────────────────────────────────────────────────────────────

    pub fn deposit(env: Env, from: Address, amount: i128, asset: Address) {
        from.require_auth();
        let client = token::Client::new(&env, &asset);
        client.transfer(&from, &env.current_contract_address(), &amount);
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        get_proposal(&env, proposal_id)
    }

    pub fn get_board(env: Env) -> BoardConfig {
        get_board(&env)
    }

    pub fn get_proposals_by_status(env: Env, status: ProposalStatus) -> Vec<Proposal> {
        let count = get_count(&env);
        let mut results = Vec::new(&env);
        for id in 1..=count {
            let p = get_proposal(&env, id);
            if p.status == status {
                results.push_back(p);
            }
        }
        results
    }

    pub fn get_proposals_by_member(env: Env, member: Address) -> Vec<Proposal> {
        let count = get_count(&env);
        let mut results = Vec::new(&env);
        for id in 1..=count {
            let p = get_proposal(&env, id);
            if p.proposer == member || p.signatures.contains(&member) {
                results.push_back(p);
            }
        }
        results
    }

    pub fn get_proposal_count(env: Env) -> u64 {
        get_count(&env)
    }

    pub fn is_member(env: Env, addr: Address) -> bool {
        let board = get_board(&env);
        is_member(&board.members, &addr)
    }

    pub fn get_stats(env: Env) -> Stats {
        let total = get_count(&env);
        let mut executed = 0u64;
        let mut pending = 0u64;
        let mut cancelled = 0u64;
        let mut expired = 0u64;
        for id in 1..=total {
            let p = get_proposal(&env, id);
            match p.status {
                ProposalStatus::Executed => executed += 1,
                ProposalStatus::Pending => pending += 1,
                ProposalStatus::Cancelled => cancelled += 1,
                ProposalStatus::Expired => expired += 1,
            }
        }
        Stats {
            total_proposals: total,
            executed,
            pending,
            cancelled,
            expired,
        }
    }

    // ── Internal Execute ──────────────────────────────────────────────────────

    fn _execute(env: &Env, proposal_id: u64) {
        let mut proposal = get_proposal(env, proposal_id);
        let ts = now(env);

        match proposal.payload.clone() {
            ProposalPayload::ResolveIssue(p) => {
                let client = token::Client::new(env, &p.asset);
                client.transfer(&env.current_contract_address(), &p.contributor, &p.amount);
            }
            ProposalPayload::TransferFunds(p) => {
                let client = token::Client::new(env, &p.asset);
                client.transfer(&env.current_contract_address(), &p.recipient, &p.amount);
            }
            ProposalPayload::AddMember(p) => {
                let mut board = get_board(env);
                if !is_member(&board.members, &p.new_member) {
                    board.members.push_back(p.new_member);
                    set_board(env, &board);
                }
            }
            ProposalPayload::RemoveMember(p) => {
                let mut board = get_board(env);
                let mut new_members: Vec<Address> = Vec::new(env);
                for m in board.members.iter() {
                    if m != p.member {
                        new_members.push_back(m);
                    }
                }
                board.members = new_members;
                set_board(env, &board);
            }
            ProposalPayload::UpdateThreshold(p) => {
                let mut board = get_board(env);
                assert!(
                    p.new_threshold as usize <= board.members.len() as usize && p.new_threshold > 0,
                    "invalid threshold"
                );
                board.threshold = p.new_threshold;
                set_board(env, &board);
            }
        }

        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(ts);
        set_proposal(env, &proposal);

        let executor = env.current_contract_address();
        events::proposal_executed(env, proposal_id, &proposal.proposal_type, &executor, ts);
    }
}
