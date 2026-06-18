# QuorumForge Contract

> **Trustless N-of-M multi-sig governance for open-source maintainer boards on Stellar.**

QuorumForge is a [Soroban](https://soroban.stellar.org) smart contract that lets open-source projects run a self-governing maintainer board entirely on-chain. Board members propose changes, collect signatures, and the contract auto-executes once the required quorum is reached — no trusted coordinator, no off-chain multisig server.

[![CI](https://github.com/your-org/quorumforge-contract/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/quorumforge-contract/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Table of Contents

- [Why QuorumForge?](#why-quorumforge)
- [Architecture](docs/ARCHITECTURE.md)
- [Contract Functions](#contract-functions)
- [N-of-M Configuration Examples](#n-of-m-configuration-examples)
- [Deploy Guide](#deploy-guide)
- [CLI Invocation Reference](#cli-invocation-reference)
- [Development](#development)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [License](#license)

---

## Why QuorumForge?

| Problem | QuorumForge Solution |
|---|---|
| Maintainer turnover breaks informal processes | Board membership is on-chain, not in someone's head |
| Bounty payouts require trust in one treasurer | Contract holds funds; payment only after quorum |
| Adding/removing maintainers causes disputes | `AddMember` / `RemoveMember` proposals require same threshold as any other action |
| Multi-sig wallets are opaque | Every signature and state change is an indexed on-chain event |

---

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design, storage layout, execution flow diagrams, and security model.

---

## Contract Functions

### `initialize(admin, members, threshold)`

One-time setup. Stores the board config and admin address.

```bash
stellar contract invoke --id $CONTRACT_ID --source $ADMIN --network testnet \
  -- initialize \
  --admin GADMIN... \
  --members '["GMEMBER1...","GMEMBER2...","GMEMBER3..."]' \
  --threshold 2
```

### `create_proposal(proposer, proposal_type, payload, ttl_seconds) → u64`

Any board member can open a proposal. Returns the proposal ID.

### `sign_proposal(signer, proposal_id)`

Board member approves a proposal. Auto-executes when `signatures.len() >= threshold`.

### `execute_proposal(proposal_id)`

Public call to trigger execution if threshold is already met. Useful if auto-execution was skipped (e.g., gas limits).

### `cancel_proposal(proposal_id, canceller)`

The original proposer or admin can cancel a pending proposal.

### `expire_proposal(proposal_id)`

Anyone can expire a proposal whose TTL has passed.

### `deposit(from, amount, asset)`

Fund the contract treasury so `TransferFunds` and `ResolveIssue` proposals have tokens to send.

### Queries

| Function | Returns |
|---|---|
| `get_proposal(id)` | `Proposal` |
| `get_board()` | `BoardConfig` |
| `get_proposals_by_status(status)` | `Vec<Proposal>` |
| `get_proposals_by_member(member)` | `Vec<Proposal>` |
| `get_stats()` | `Stats` |

---

## N-of-M Configuration Examples

### 2-of-3 (small project, fast decisions)

```json
{ "members": ["Alice", "Bob", "Carol"], "threshold": 2 }
```
Any two of three maintainers can act. Carol can be outvoted but still participates.

### 3-of-5 (mid-size project, balanced)

```json
{ "members": ["A","B","C","D","E"], "threshold": 3 }
```
A strict majority is always required. Two members cannot collude alone.

### 4-of-5 (high-value treasury, conservative)

```json
{ "members": ["A","B","C","D","E"], "threshold": 4 }
```
Only one member can be absent/disagree. Best for contracts holding significant funds.

### 5-of-7 (DAO-style governance board)

```json
{ "members": ["A","B","C","D","E","F","G"], "threshold": 5 }
```
Super-majority required. Maximally resistant to collusion.

---

## Deploy Guide

### Prerequisites

- Rust toolchain with `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- Stellar CLI:
  ```bash
  cargo install --locked stellar-cli --features opt
  ```
- A funded Stellar account (testnet faucet: `stellar keys generate --global mykey --network testnet`)

### Quick Deploy (Testnet)

```bash
# 1. Clone and build
git clone https://github.com/your-org/quorumforge-contract
cd quorumforge-contract

# 2. Deploy
./scripts/deploy.sh testnet mykey
# → outputs CONTRACT_ID=C...

# 3. Initialize
stellar contract invoke \
  --id $CONTRACT_ID --source mykey --network testnet \
  -- initialize \
  --admin $(stellar keys address mykey) \
  --members '["GMEMBER1","GMEMBER2","GMEMBER3"]' \
  --threshold 2
```

Or with Make:

```bash
make deploy-testnet SOURCE_ACCOUNT=mykey
make invoke-initialize CONTRACT_ID=C... ADMIN=G... \
     MEMBERS='["G1","G2","G3"]' THRESHOLD=2
```

### Mainnet Deploy

```bash
./scripts/deploy.sh mainnet my-mainnet-key
```

The script requires you to type `yes` to confirm before deploying to mainnet.

---

## CLI Invocation Reference

All examples use `stellar contract invoke`. Set these env vars for brevity:

```bash
export CONTRACT_ID=C...
export NETWORK=testnet
export SOURCE=mykey
```

### Create a ResolveIssue proposal

```bash
stellar contract invoke --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- create_proposal \
  --proposer GPROPOSER... \
  --proposal_type '"ResolveIssue"' \
  --payload '{"ResolveIssue":{"issue_number":42,"contributor":"GCONTRIB...","amount":"500","asset":"CASSET..."}}' \
  --ttl_seconds null
```

### Create a TransferFunds proposal

```bash
stellar contract invoke --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- create_proposal \
  --proposer GPROPOSER... \
  --proposal_type '"TransferFunds"' \
  --payload '{"TransferFunds":{"recipient":"GRECIP...","amount":"1000","asset":"CASSET...","memo":"Q3 grant"}}' \
  --ttl_seconds null
```

### Sign a proposal

```bash
stellar contract invoke --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- sign_proposal \
  --signer GSIGNER... \
  --proposal_id 1
```

### Get proposal state

```bash
stellar contract invoke --id $CONTRACT_ID --network $NETWORK \
  -- get_proposal --proposal_id 1
```

### Deposit funds

```bash
stellar contract invoke --id $CONTRACT_ID --source $SOURCE --network $NETWORK \
  -- deposit \
  --from GFROM... \
  --amount 10000 \
  --asset CASSET...
```

---

## Development

```bash
make test        # run all tests
make fmt         # format
make lint        # clippy
make build       # build wasm
make optimize    # build + optimize wasm
```

### Project Layout

```
quorumforge-contract/
├── src/
│   ├── lib.rs       # contract entry points
│   ├── types.rs     # all types, enums, structs
│   ├── storage.rs   # storage read/write helpers
│   ├── events.rs    # event emission helpers
│   └── test.rs      # integration tests
├── docs/
│   ├── ARCHITECTURE.md
│   └── PROPOSAL_TYPES.md
├── scripts/
│   └── deploy.sh
├── .github/workflows/ci.yml
├── Cargo.toml
├── Makefile
├── CONTRIBUTING.md
└── SECURITY.md
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md) for vulnerability disclosure policy.

## License

MIT — see [LICENSE](LICENSE).
