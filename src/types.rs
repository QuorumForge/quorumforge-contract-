use soroban_sdk::{contracttype, Address, String, Vec};

// ── Storage Keys ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Board,
    Admin,
    Proposal(u64),
    Count,
}

// ── Board ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct BoardConfig {
    pub members: Vec<Address>,
    pub threshold: u32,
    pub created_at: u64,
}

// ── Proposal Types ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalType {
    ResolveIssue,
    TransferFunds,
    AddMember,
    RemoveMember,
    UpdateThreshold,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Executed,
    Expired,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalPayload {
    ResolveIssue(ResolveIssuePayload),
    TransferFunds(TransferFundsPayload),
    AddMember(AddMemberPayload),
    RemoveMember(RemoveMemberPayload),
    UpdateThreshold(UpdateThresholdPayload),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ResolveIssuePayload {
    pub issue_number: u64,
    pub contributor: Address,
    pub amount: i128,
    pub asset: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct TransferFundsPayload {
    pub recipient: Address,
    pub amount: i128,
    pub asset: Address,
    pub memo: String,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AddMemberPayload {
    pub new_member: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct RemoveMemberPayload {
    pub member: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateThresholdPayload {
    pub new_threshold: u32,
}

// ── Proposal ─────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub proposal_id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub payload: ProposalPayload,
    pub signatures: Vec<Address>,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub executed_at: Option<u64>,
    /// Timestamp when the proposal was cancelled, if applicable.
    pub cancelled_at: Option<u64>,
    /// Human-readable description of the proposal's intent (max 256 chars).
    pub description: String,
}

// ── Stats ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Stats {
    pub total_proposals: u64,
    pub executed: u64,
    pub pending: u64,
    pub cancelled: u64,
    pub expired: u64,
}
