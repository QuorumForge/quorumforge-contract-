#![no_std]

mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};

use crate::{
    storage::{
        self, get_admin, get_board, get_count, get_proposal, has_board, increment_count, set_admin,
        set_board, set_proposal,
    },
    types::{BoardConfig, Proposal, ProposalPayload, ProposalStatus, ProposalType, Stats},
};

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
            (members.len() as u32) <= crate::types::MAX_MEMBERS,
            "too many members"
        );
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
        description: String,
        ttl_seconds: Option<u64>,
    ) -> u64 {
        proposer.require_auth();

        let board = get_board(&env);
        assert!(is_member(&board.members, &proposer), "not a board member");

        let ts = now(&env);
        let ttl = ttl_seconds.unwrap_or(crate::types::DEFAULT_TTL_SECS);
        assert!(ttl >= MIN_TTL_SECS, "ttl too short");
        assert!(ttl <= MAX_TTL_SECS, "ttl too long");

        // Enforce description length limit
        assert!(
            description.len() <= crate::types::MAX_DESCRIPTION_LEN,
            "description too long"
        );

        let expires_at = ts + ttl;
        let proposal_id = increment_count(&env);

        let proposal = Proposal {
            proposal_id,
            proposer: proposer.clone(),
            proposal_type: proposal_type.clone(),
            payload,
            description,
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
            events::quorum_reached(&env, proposal_id, sig_count, board.threshold, now(&env));
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

        let ts = now(&env);
        proposal.status = ProposalStatus::Cancelled;
        proposal.cancelled_at = Some(ts);
        set_proposal(&env, &proposal);

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
        let ts = now(&env);
        events::deposit_received(&env, &from, amount, &asset, ts);
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        get_proposal(&env, proposal_id)
    }

    pub fn get_board(env: Env) -> BoardConfig {
        get_board(&env)
    }

    /// Returns the admin address that was set during initialization.
    pub fn get_admin(env: Env) -> Address {
        get_admin(&env)
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

    /// Returns `true` if `addr` has already signed the given proposal.
    pub fn has_signed(env: Env, proposal_id: u64, addr: Address) -> bool {
        let proposal = get_proposal(&env, proposal_id);
        proposal.signatures.contains(&addr)
    }

    /// Returns the total number of current board members.
    pub fn get_member_count(env: Env) -> u32 {
        get_board(&env).members.len()
    }

    /// Returns `true` if the board has been initialized and an admin is set.
    pub fn is_initialized(env: Env) -> bool {
        storage::has_board(&env) && storage::has_admin(&env)
    }

    /// Returns the current signing threshold without fetching the full board config.
    pub fn get_threshold(env: Env) -> u32 {
        get_board(&env).threshold
    }

    /// Returns `true` if a proposal with the given ID exists in storage.
    pub fn has_proposal(env: Env, proposal_id: u64) -> bool {
        storage::has_proposal(&env, proposal_id)
    }

    /// Convenience shortcut — returns all proposals with `Pending` status.
    pub fn get_pending_proposals(env: Env) -> Vec<Proposal> {
        Self::get_proposals_by_status(env, ProposalStatus::Pending)
    }

    pub fn get_stats(env: Env) -> Stats {
        let total = get_count(&env);
        let mut executed = 0u64;
        let mut pending = 0u64;
        let mut cancelled = 0u64;
        let mut expired = 0u64;
        let mut total_signatures = 0u64;
        for id in 1..=total {
            let p = get_proposal(&env, id);
            total_signatures += p.signatures.len() as u64;
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
            total_signatures,
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
                events::withdrawal_requested(env, proposal_id, &p.recipient, p.amount, ts);
            }
            ProposalPayload::AddMember(p) => {
                let mut board = get_board(env);
                assert!(
                    !is_member(&board.members, &p.new_member),
                    "member already exists"
                );
                assert!(
                    (board.members.len() as u32) < crate::types::MAX_MEMBERS,
                    "board is at maximum capacity"
                );
                board.members.push_back(p.new_member.clone());
                let count = board.members.len();
                let threshold = board.threshold;
                set_board(env, &board);
                events::member_added(env, &p.new_member, count, ts);
                events::board_updated(env, count, threshold, ts);
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
                let count = board.members.len();
                let threshold = board.threshold;
                set_board(env, &board);
                events::member_removed(env, &p.member, count, ts);
                events::board_updated(env, count, threshold, ts);
            }
            ProposalPayload::UpdateThreshold(p) => {
                let mut board = get_board(env);
                assert!(
                    p.new_threshold as usize <= board.members.len() as usize && p.new_threshold > 0,
                    "invalid threshold"
                );
                let old_threshold = board.threshold;
                board.threshold = p.new_threshold;
                let count = board.members.len();
                set_board(env, &board);
                events::threshold_updated(env, old_threshold, p.new_threshold, ts);
                events::board_updated(env, count, p.new_threshold, ts);
            }
        }

        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = Some(ts);
        set_proposal(env, &proposal);

        let executor = env.current_contract_address();
        events::proposal_executed(env, proposal_id, &proposal.proposal_type, &executor, ts);
    }
}
