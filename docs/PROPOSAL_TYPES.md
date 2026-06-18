# Proposal Types Reference

This document describes every `ProposalPayload` variant, its fields, execution semantics, and recommended use cases.

---

## ResolveIssue

Pay a contributor for resolving a tracked issue.

```json
{
  "ResolveIssue": {
    "issue_number": 42,
    "contributor": "GCONTRIBUTOR...",
    "amount": 500,
    "asset": "CASSET..."
  }
}
```

| Field | Type | Description |
|---|---|---|
| `issue_number` | `u64` | Issue tracker ID (GitHub, GitLab, etc.) |
| `contributor` | `Address` | Stellar address of the contributor |
| `amount` | `i128` | Stroops or token units to pay |
| `asset` | `Address` | Contract address of the token |

**Execution:** `token::transfer(contract → contributor, amount)`.

The contract must hold at least `amount` of `asset` before execution. Use `deposit` to fund the treasury first.

---

## TransferFunds

General-purpose treasury transfer for grants, reimbursements, or operational expenses.

```json
{
  "TransferFunds": {
    "recipient": "GRECIPIENT...",
    "amount": 1000,
    "asset": "CASSET...",
    "memo": "Q3 infrastructure grant"
  }
}
```

| Field | Type | Description |
|---|---|---|
| `recipient` | `Address` | Destination address |
| `amount` | `i128` | Amount to transfer |
| `asset` | `Address` | Token contract address |
| `memo` | `String` | Human-readable note (stored on-chain) |

**Execution:** `token::transfer(contract → recipient, amount)`.

---

## AddMember

Add a new address to the board.

```json
{
  "AddMember": {
    "new_member": "GNEWMEMBER..."
  }
}
```

| Field | Type | Description |
|---|---|---|
| `new_member` | `Address` | Address to add to board |

**Execution:** Appends `new_member` to `board.members`. Idempotent — adding an existing member is a no-op.

After execution, the new member can immediately create and sign proposals.

---

## RemoveMember

Remove an address from the board.

```json
{
  "RemoveMember": {
    "member": "GMEMBER..."
  }
}
```

| Field | Type | Description |
|---|---|---|
| `member` | `Address` | Address to remove |

**Execution:** Filters `member` out of `board.members`. Does not automatically adjust `threshold`.

> ⚠️ If removing a member would leave `members.len() < threshold`, all future proposals will be un-executable. Pair this with an `UpdateThreshold` proposal if necessary.

---

## UpdateThreshold

Change the required signature count.

```json
{
  "UpdateThreshold": {
    "new_threshold": 3
  }
}
```

| Field | Type | Description |
|---|---|---|
| `new_threshold` | `u32` | New minimum signature count |

**Execution:** Sets `board.threshold = new_threshold`. Panics if `new_threshold > members.len()` or `new_threshold == 0`.

This can be used to raise or lower the bar. Raising the threshold increases security. Lowering it speeds up decision-making.

---

## Choosing the Right Payload

| Situation | Payload |
|---|---|
| Pay a bounty hunter | `ResolveIssue` |
| Fund a grant or pay a vendor | `TransferFunds` |
| Onboard a new maintainer | `AddMember` |
| Offboard a departing maintainer | `RemoveMember` |
| Adjust governance rules | `UpdateThreshold` |
