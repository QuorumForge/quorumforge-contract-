# Changelog

All notable changes to `quorumforge-contract` are documented here.
Follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- `description: String` field on `Proposal` — a human-readable summary stored on-chain alongside the typed payload.
- `create_proposal` now accepts a `description` parameter.
- `has_signed(proposal_id, addr)` — query whether a specific address has already signed a proposal.
- `get_member_count()` — returns the current number of board members without fetching the full `BoardConfig`.
- `get_pending_proposals()` — convenience shortcut equivalent to `get_proposals_by_status(Pending)`.
- `get_admin()` — public query to retrieve the admin address set at initialization.
- `get_proposal_count()` — returns the total number of proposals ever created.
- `is_member(addr)` — public query to check membership without fetching the full board.
- `board_updated` event — emitted on `AddMember`, `RemoveMember`, and `UpdateThreshold` executions with the new member count and threshold.
- `MIN_TTL_SECS` (1 hour) and `MAX_TTL_SECS` (30 days) constants. TTL is now validated on proposal creation.

### Changed
- `create_proposal` signature now includes `description: String` before `ttl_seconds`.

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
