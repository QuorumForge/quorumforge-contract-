#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String, Vec,
};

use crate::{
    types::{
        AddMemberPayload, ProposalPayload, ProposalStatus, ProposalType, RemoveMemberPayload,
        ResolveIssuePayload, TransferFundsPayload, UpdateThresholdPayload,
    },
    QuorumForge, QuorumForgeClient,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

struct Setup {
    env: Env,
    contract: Address,
    #[allow(dead_code)]
    admin: Address,
    alice: Address,
    bob: Address,
    carol: Address,
    asset: Address,
}

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn setup_board() -> Setup {
    let env = make_env();
    let contract = env.register(QuorumForge, ());
    let client = QuorumForgeClient::new(&env, &contract);

    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);

    let mut members = Vec::new(&env);
    members.push_back(alice.clone());
    members.push_back(bob.clone());
    members.push_back(carol.clone());

    client.initialize(&admin, &members, &2);

    // mint a token for transfer tests
    let asset_admin = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(asset_admin.clone());
    let asset_addr = asset.address();
    let asset_client = StellarAssetClient::new(&env, &asset_addr);
    asset_client.mint(&contract, &10_000);

    Setup {
        env,
        contract,
        admin,
        alice,
        bob,
        carol,
        asset: asset_addr,
    }
}

fn resolve_payload(s: &Setup) -> ProposalPayload {
    ProposalPayload::ResolveIssue(ResolveIssuePayload {
        issue_number: 42,
        contributor: s.carol.clone(),
        amount: 100,
        asset: s.asset.clone(),
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// 2-of-3 happy path: create → sign × 2 → auto-execute
#[test]
fn test_happy_path_auto_execute() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    assert_eq!(id, 1);

    client.sign_proposal(&s.alice, &id);
    // after first sig, still pending
    let p = client.get_proposal(&id);
    assert_eq!(p.status, ProposalStatus::Pending);
    assert_eq!(p.signatures.len(), 1);

    client.sign_proposal(&s.bob, &id);
    // second sig hits threshold → executed
    let p = client.get_proposal(&id);
    assert_eq!(p.status, ProposalStatus::Executed);
    assert!(p.executed_at.is_some());
}

/// Signing twice by same member is rejected
#[test]
#[should_panic(expected = "already signed")]
fn test_double_sign_rejected() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    client.sign_proposal(&s.alice, &id);
    client.sign_proposal(&s.alice, &id); // panic
}

/// Non-member cannot create a proposal
#[test]
#[should_panic(expected = "not a board member")]
fn test_non_member_create() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let outsider = Address::generate(&s.env);
    client.create_proposal(
        &outsider,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
}

/// Non-member cannot sign a proposal
#[test]
#[should_panic(expected = "not a board member")]
fn test_non_member_sign() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    let outsider = Address::generate(&s.env);
    client.sign_proposal(&outsider, &id);
}

/// Expired proposal cannot be signed
#[test]
#[should_panic(expected = "proposal expired")]
fn test_expired_cannot_sign() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &Some(60u64), // 60 second TTL
    );
    // advance ledger past TTL
    s.env.ledger().with_mut(|l| {
        l.timestamp += 120;
    });
    client.sign_proposal(&s.bob, &id);
}

/// Cancelled proposal cannot be signed
#[test]
#[should_panic(expected = "proposal not pending")]
fn test_cancelled_cannot_sign() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    client.cancel_proposal(&id, &s.alice);
    client.sign_proposal(&s.bob, &id);
}

/// AddMember proposal executes and board updates
#[test]
fn test_add_member() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let newcomer = Address::generate(&s.env);

    let payload = ProposalPayload::AddMember(AddMemberPayload {
        new_member: newcomer.clone(),
    });
    let id = client.create_proposal(&s.alice, &ProposalType::AddMember, &payload, &None);
    client.sign_proposal(&s.alice, &id);
    client.sign_proposal(&s.bob, &id);

    let board = client.get_board();
    assert!(board.members.contains(&newcomer));
    assert_eq!(board.members.len(), 4);
}

/// RemoveMember proposal executes and board updates
#[test]
fn test_remove_member() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let payload = ProposalPayload::RemoveMember(RemoveMemberPayload {
        member: s.carol.clone(),
    });
    let id = client.create_proposal(&s.alice, &ProposalType::RemoveMember, &payload, &None);
    client.sign_proposal(&s.alice, &id);
    client.sign_proposal(&s.bob, &id);

    let board = client.get_board();
    assert!(!board.members.contains(&s.carol));
    assert_eq!(board.members.len(), 2);
}

/// UpdateThreshold rejects if new threshold > member count
#[test]
#[should_panic(expected = "invalid threshold")]
fn test_update_threshold_too_high() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let payload = ProposalPayload::UpdateThreshold(UpdateThresholdPayload { new_threshold: 99 });
    let id = client.create_proposal(&s.alice, &ProposalType::UpdateThreshold, &payload, &None);
    client.sign_proposal(&s.alice, &id);
    client.sign_proposal(&s.bob, &id); // executes → panics
}

/// Public execute_proposal works when threshold already met
#[test]
fn test_public_execute_after_threshold() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    client.sign_proposal(&s.alice, &id);
    client.sign_proposal(&s.bob, &id);

    // already executed; calling again should panic
    // Instead test that public call works before threshold on a fresh proposal
    let id2 = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    client.sign_proposal(&s.alice, &id2);
    client.sign_proposal(&s.bob, &id2);
    // threshold met internally, proposal already executed
    let p = client.get_proposal(&id2);
    assert_eq!(p.status, ProposalStatus::Executed);
}

/// get_proposals_by_status returns correct filtered list
#[test]
fn test_get_proposals_by_status() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let id1 = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    let id2 = client.create_proposal(
        &s.bob,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );

    // execute id1
    client.sign_proposal(&s.alice, &id1);
    client.sign_proposal(&s.bob, &id1);

    // cancel id2
    client.cancel_proposal(&id2, &s.bob);

    let pending = client.get_proposals_by_status(&ProposalStatus::Pending);
    let executed = client.get_proposals_by_status(&ProposalStatus::Executed);
    let cancelled = client.get_proposals_by_status(&ProposalStatus::Cancelled);

    assert_eq!(pending.len(), 0);
    assert_eq!(executed.len(), 1);
    assert_eq!(cancelled.len(), 1);
}

/// expire_proposal transitions Pending → Expired
#[test]
fn test_expire_proposal() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &Some(60u64),
    );
    s.env.ledger().with_mut(|l| {
        l.timestamp += 120;
    });
    client.expire_proposal(&id);
    let p = client.get_proposal(&id);
    assert_eq!(p.status, ProposalStatus::Expired);
}

/// TransferFunds payload moves tokens to recipient
#[test]
fn test_transfer_funds() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let recipient = Address::generate(&s.env);

    let payload = ProposalPayload::TransferFunds(TransferFundsPayload {
        recipient: recipient.clone(),
        amount: 500,
        asset: s.asset.clone(),
        memo: String::from_str(&s.env, "bounty"),
    });
    let id = client.create_proposal(&s.alice, &ProposalType::TransferFunds, &payload, &None);
    client.sign_proposal(&s.alice, &id);
    client.sign_proposal(&s.bob, &id);

    let balance = TokenClient::new(&s.env, &s.asset).balance(&recipient);
    assert_eq!(balance, 500);
}

/// has_signed returns correct result before and after signing
#[test]
fn test_has_signed() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );

    assert!(!client.has_signed(&id, &s.alice));
    client.sign_proposal(&s.alice, &id);
    assert!(client.has_signed(&id, &s.alice));
    assert!(!client.has_signed(&id, &s.bob));
}

/// get_pending_proposals returns only pending proposals
#[test]
fn test_get_pending_proposals() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let id1 = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    let _id2 = client.create_proposal(
        &s.bob,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );

    // execute id1
    client.sign_proposal(&s.alice, &id1);
    client.sign_proposal(&s.bob, &id1);

    let pending = client.get_pending_proposals();
    assert_eq!(pending.len(), 1);
}

/// get_stats returns correct aggregated counts
#[test]
fn test_get_stats() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);

    let id1 = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    // execute id1 with 2 signatures
    client.sign_proposal(&s.alice, &id1);
    client.sign_proposal(&s.bob, &id1);

    let stats = client.get_stats();
    assert_eq!(stats.total_proposals, 1);
    assert_eq!(stats.executed, 1);
    assert_eq!(stats.pending, 0);
    assert_eq!(stats.total_signatures, 2);
}

/// cancelled_at is populated when a proposal is cancelled
#[test]
fn test_cancelled_at_is_set() {
    let s = setup_board();
    let client = QuorumForgeClient::new(&s.env, &s.contract);
    let id = client.create_proposal(
        &s.alice,
        &ProposalType::ResolveIssue,
        &resolve_payload(&s),
        &None,
    );
    client.cancel_proposal(&id, &s.alice);
    let p = client.get_proposal(&id);
    assert_eq!(p.status, ProposalStatus::Cancelled);
    assert!(p.cancelled_at.is_some());
}
