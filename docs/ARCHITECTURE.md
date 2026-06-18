# Architecture — QuorumForge Contract

This document describes the on-chain design of `quorumforge-contract`: its storage layout, module structure, execution flow, event model, and security considerations.

---

## Table of Contents

- [Overview](#overview)
- [Module Structure](#module-structure)
- [Storage Layout](#storage-layout)
- [Type System](#type-system)
- [Execution Flow](#execution-flow)
  - [Initialize](#initialize)
  - [Create Proposal](#create-proposal)
  - [Sign → Auto-Execute](#sign--auto-execute)
  - [Public Execute](#public-execute)
  - [Cancel / Expire](#cancel--expire)
- [Event Model](#event-model)
- [Proposal Payload Execution](#proposal-payload-execution)
- [Security Model](#security-model)
- [Upgrade Path](#upgrade-path)

---

## Overview

QuorumForge implements an N-of-M signature scheme directly in a Soroban smart contract. There is no off-chain coordination layer — every state transition is an on-chain transaction, every approval is an authenticated signature, and every execution is deterministic.

```
┌─────────────────────────────────────────────────────────────┐
│                     QuorumForge Contract                    │
│                                                             │
│  Board Members  ──►  Proposals  ──►  Signatures  ──►  Exec │
│                                                             │
│  BoardConfig (members[], threshold)                        │
│  Proposal (id, type, payload, sigs[], status, TTL)         │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Structure

```
src/
├── lib.rs       — #[contract] + #[contractimpl]: all public entry points
├── types.rs     — #[contracttype] enums and structs
├── storage.rs   — typed wrappers around env.storage()
├── events.rs    — typed event emission helpers
└── test.rs      — #[cfg(test)] integration tests
```

Each module has a single responsibility. `lib.rs` contains only business logic; all raw storage calls go through `storage.rs`, all `env.events().publish` calls go through `events.rs`.

---

## Storage Layout

QuorumForge uses two storage tiers:

| Key | Tier | Value | Notes |
|---|---|---|---|
| `DataKey::Board` | Instance | `BoardConfig` | Shared contract lifecycle |
| `DataKey::Admin` | Instance | `Address` | Shared contract lifecycle |
| `DataKey::Count` | Instance | `u64` | Proposal counter |
| `DataKey::Proposal(id)` | Persistent | `Proposal` | Per-proposal, long-lived |

**Instance storage** is shared across the contract's entire lifetime and is archived with the contract instance entry. It is appropriate for small, always-needed state like the board config.

**Persistent storage** is used for proposals because each proposal is an independent, long-lived record. Proposal entries survive ledger expiration policies for as long as their rent is paid.

---

## Type System

### BoardConfig

```rust
struct BoardConfig {
    members: Vec<Address>,   // current board members
    threshold: u32,          // minimum signatures required
    created_at: u64,         // unix timestamp of initialization
}
```

### Proposal

```rust
struct Proposal {
    proposal_id:   u64,
    proposer:      Address,
    proposal_type: ProposalType,
    payload:       ProposalPayload,
    signatures:    Vec<Address>,    // addresses that have signed
    status:        ProposalStatus,
    created_at:    u64,
    expires_at:    u64,
    executed_at:   Option<u64>,
}
```

### ProposalStatus Transitions

```
                   ┌──────────┐
         create    │          │  sign (threshold met)
       ──────────► │ Pending  │ ─────────────────────► Executed
                   │          │
                   └──────────┘
                        │  │
             cancel ────┘  └──── expire
                        │  │
                   Cancelled  Expired
```

Status is terminal once it leaves `Pending`. No re-activation is possible.

### ProposalPayload (tagged union)

```
ProposalPayload
  ├── ResolveIssue  { issue_number, contributor, amount, asset }
  ├── TransferFunds { recipient, amount, asset, memo }
  ├── AddMember     { new_member }
  ├── RemoveMember  { member }
  └── UpdateThreshold { new_threshold }
```

See [PROPOSAL_TYPES.md](PROPOSAL_TYPES.md) for payload-level documentation.

---

## Execution Flow

### Initialize

```
Admin ──► initialize(admin, members[], threshold)
            │
            ├── assert: not already initialized
            ├── assert: members not empty
            ├── assert: 0 < threshold ≤ members.len()
            ├── store BoardConfig (instance)
            ├── store admin (instance)
            ├── set count = 0 (instance)
            └── emit BoardInitializedEvent
```

`initialize` is callable exactly once. Subsequent calls panic with `"already initialized"`.

### Create Proposal

```
Member ──► create_proposal(proposer, type, payload, ttl?)
             │
             ├── proposer.require_auth()
             ├── assert: proposer ∈ board.members
             ├── id = ++count
             ├── expires_at = now + ttl (default: 7 days)
             ├── store Proposal { status: Pending, signatures: [] }
             └── emit ProposalCreatedEvent
```

### Sign → Auto-Execute

```
Member ──► sign_proposal(signer, proposal_id)
             │
             ├── signer.require_auth()
             ├── assert: signer ∈ board.members
             ├── load proposal
             ├── assert: status == Pending
             ├── assert: now ≤ expires_at
             ├── assert: signer ∉ signatures
             ├── signatures.push(signer)
             ├── emit ProposalSignedEvent
             │
             └── if signatures.len() >= threshold
                   └── _execute(proposal_id)
                         │
                         ├── match payload → execute action
                         ├── status = Executed
                         ├── executed_at = now
                         └── emit ProposalExecutedEvent
```

Auto-execution happens atomically within the same transaction as the final signature. There is no separate execution window where state can change between threshold being reached and the action occurring.

### Public Execute

The `execute_proposal(proposal_id)` entry point is callable by anyone. It validates that `status == Pending` and `signatures.len() >= threshold`, then calls `_execute`. This is a safety valve in case auto-execution is skipped due to resource limits on a high-traffic ledger.

### Cancel / Expire

```
cancel_proposal(proposal_id, canceller)
  ├── canceller.require_auth()
  ├── assert: canceller == proposer OR admin
  ├── assert: status == Pending
  ├── status = Cancelled
  └── emit ProposalCancelledEvent

expire_proposal(proposal_id)          [anyone can call]
  ├── assert: status == Pending
  ├── assert: now > expires_at
  ├── status = Expired
  └── emit ProposalExpiredEvent
```

---

## Event Model

All events are published under a two-symbol topic `(namespace, action)`:

| Topic | Data fields |
|---|---|
| `("board", "init")` | `(members, threshold, timestamp)` |
| `("proposal", "created")` | `(id, proposer, type, expires_at)` |
| `("proposal", "signed")` | `(id, signer, sig_count, threshold)` |
| `("proposal", "executed")` | `(id, type, executor, timestamp)` |
| `("proposal", "cancelled")` | `(id, canceller, timestamp)` |
| `("proposal", "expired")` | `(id, timestamp)` |

Events are indexed by the Stellar Horizon API and can be subscribed to via streaming.

---

## Proposal Payload Execution

| Payload | Contract Action |
|---|---|
| `ResolveIssue` | `token::Client::transfer(contract → contributor, amount)` |
| `TransferFunds` | `token::Client::transfer(contract → recipient, amount)` |
| `AddMember` | Appends `new_member` to `board.members` (idempotent) |
| `RemoveMember` | Filters `member` out of `board.members` |
| `UpdateThreshold` | Sets `board.threshold` (validates ≤ member count) |

Token transfers use the Soroban token interface (`soroban_sdk::token::Client`). The contract must hold sufficient token balance before a transfer proposal is executed. Use the `deposit` function to fund the treasury.

---

## Security Model

### Authentication

Every state-mutating call requires `Address::require_auth()` for the acting party. Soroban's auth framework enforces this at the VM level — no spoofing is possible.

### Threshold Invariant

- `threshold` must satisfy `1 ≤ threshold ≤ members.len()` at all times.
- `UpdateThreshold` proposals are rejected at execution time if the new value would violate this invariant.
- `RemoveMember` does not automatically lower the threshold. Projects should pair a `RemoveMember` with an `UpdateThreshold` proposal if the removal would leave fewer members than the current threshold.

### Re-entrancy

Soroban's execution model does not allow re-entrant calls to the same contract within the same transaction. Cross-contract calls (token transfers) complete before control returns, so the proposal status is updated after the transfer in a single atomic operation.

### Replay Protection

Each proposal has a monotonically incrementing `proposal_id`. A proposal can only be executed once: execution sets `status = Executed`, and all subsequent calls to `sign_proposal` or `execute_proposal` will panic on the `status == Pending` assertion.

### No Admin Escape Hatch

The admin address can cancel proposals but cannot bypass the threshold to execute them. The only privileged admin actions are cancelling proposals and initializing the contract.

---

## Upgrade Path

This contract does not implement an upgrade mechanism in v0.1. To upgrade:

1. Deploy a new contract instance.
2. Pass a `TransferFunds` or `ResolveIssue` proposal to drain any treasury balance to the new contract.
3. Update client integrations to point to the new contract ID.

A governance-controlled upgrade mechanism (e.g., using `UpdateThreshold` pattern for contract migration) is planned for a future version.
