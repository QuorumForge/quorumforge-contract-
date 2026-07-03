# Events Reference — QuorumForge Contract

QuorumForge emits typed events for every state transition. This document lists each event, its topic keys, and the data tuple it carries. Events can be subscribed to via the Stellar Horizon streaming API or indexed by off-chain services.

---

## Topic Format

All events use a two-symbol topic:

```
(namespace: Symbol, action: Symbol)
```

Symbols are limited to 9 characters each to fit in a single `symbol_short!` value.

---

## Board Events

### `("board", "init")`

Emitted once during `initialize`.

| Field | Type | Description |
|---|---|---|
| `members` | `Vec<Address>` | Initial board member list |
| `threshold` | `u32` | Required signature count |
| `timestamp` | `u64` | Unix ledger timestamp |

---

### `("board", "updated")`

Emitted when board membership or threshold changes as a result of an executed `AddMember`, `RemoveMember`, or `UpdateThreshold` proposal.

| Field | Type | Description |
|---|---|---|
| `members_count` | `u32` | New total member count |
| `threshold` | `u32` | Current threshold (unchanged for member adds/removes) |
| `timestamp` | `u64` | Unix ledger timestamp |

---

## Treasury Events

### `("treasury", "deposit")`

Emitted when tokens are deposited into the contract treasury via `deposit`.

| Field | Type | Description |
|---|---|---|
| `from` | `Address` | Address that initiated the deposit |
| `amount` | `i128` | Amount deposited |
| `asset` | `Address` | Token contract address |
| `timestamp` | `u64` | Unix ledger timestamp |

---

## Proposal Events

### `("proposal", "created")`

Emitted when a new proposal is created.

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | New proposal ID |
| `proposer` | `Address` | Address that created it |
| `proposal_type` | `ProposalType` | Type enum value |
| `expires_at` | `u64` | Unix timestamp of expiry |

---

### `("proposal", "signed")`

Emitted each time a member signs a proposal.

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | Proposal that was signed |
| `signer` | `Address` | Signing member's address |
| `signatures_count` | `u32` | Running total of signatures |
| `threshold` | `u32` | Required count for execution |

---

### `("proposal", "executed")`

Emitted when a proposal reaches quorum and its action is executed.

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | Proposal that was executed |
| `proposal_type` | `ProposalType` | Type of action taken |
| `executor` | `Address` | Contract's own address (self-executing) |
| `timestamp` | `u64` | Unix ledger timestamp |

---

### `("proposal", "cancelled")`

Emitted when a proposer or admin cancels a pending proposal.

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | Cancelled proposal |
| `canceller` | `Address` | Address that cancelled |
| `timestamp` | `u64` | Unix ledger timestamp |

---

### `("proposal", "expired")`

Emitted when `expire_proposal` is called on a proposal past its TTL.

| Field | Type | Description |
|---|---|---|
| `proposal_id` | `u64` | Expired proposal |
| `timestamp` | `u64` | Unix ledger timestamp |

---

## Subscribing via Horizon

```bash
curl "https://horizon-testnet.stellar.org/contracts/<CONTRACT_ID>/events?order=asc&cursor=now"
```

Filter by topic with `?topic[0]=proposal&topic[1]=created` to receive only creation events.
