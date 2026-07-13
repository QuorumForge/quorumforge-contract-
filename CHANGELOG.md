# Changelog

All notable changes to `quorumforge-contract` are documented here.
Follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- `description: String` field on `Proposal`.
- `has_signed(proposal_id, addr)` — query whether an address has signed.
- `get_member_count()` — board size without fetching full config.
- `get_pending_proposals()` — shortcut for `get_proposals_by_status(Pending)`.
- `get_admin()` — public admin address query.
- `get_proposal_count()` — total proposals ever created.
- `is_member(addr)` — public membership check.
- `board_updated` event on membership/threshold changes.
- `member_added` and `member_removed` granular events.
- `threshold_updated` event with old and new values.
- `withdrawal_requested` event on `TransferFunds` execution.
- `MIN_TTL_SECS` / `MAX_TTL_SECS` constants enforced on proposal creation.
- `MAX_DESCRIPTION_LEN` (256 chars) enforced on `create_proposal`.
- `MAX_MEMBERS` (20) cap enforced on `initialize` and `AddMember`.
- `cancelled_at: Option<u64>` field populated on proposal cancellation.
- `total_signatures: u64` included in `Stats`.
- `extend_proposal_ttl` and `extend_instance_ttl` storage helpers.

### Fixed
- `AddMember` execution now rejects duplicate members and enforces board capacity.
- `UpdateThreshold` emits both `threshold_updated` and `board_updated` events.

### Changed
- `create_proposal` signature includes `description: String` before `ttl_seconds`.
- `cancel_proposal` now sets `cancelled_at` timestamp on the proposal.

---

## [0.1.0] — 2025-06-01

### Added
- Initial release.
- `initialize(admin, members, threshold)` — one-time board setup.
- `create_proposal`, `sign_proposal`, `execute_proposal`, `cancel_proposal`, `expire_proposal`.
- Proposal types: `ResolveIssue`, `TransferFunds`, `AddMember`, `RemoveMember`, `UpdateThreshold`.
- Auto-execution when signature count reaches threshold.
- `deposit(from, amount, asset)` — fund the treasury.
- Queries: `get_proposal`, `get_board`, `get_proposals_by_status`, `get_proposals_by_member`, `get_stats`.
- Events for all state transitions.
- Full test suite covering all proposal types and edge cases.
