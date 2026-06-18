# Contributing to QuorumForge

Thank you for your interest in contributing! QuorumForge is a community-governed project. All changes — including membership and governance rule changes for this repository itself — follow the processes described here.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Development Setup](#development-setup)
- [Workflow](#workflow)
- [Commit Style](#commit-style)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Security Issues](#security-issues)

---

## Code of Conduct

This project follows the [Contributor Covenant](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). Be respectful and constructive. Harassment of any kind is not tolerated.

---

## Ways to Contribute

- **Bug reports** — Open a GitHub Issue with a minimal reproduction.
- **Feature requests** — Open an Issue describing the use case before writing code.
- **Pull requests** — Bug fixes, new proposal types, documentation improvements, test coverage.
- **Security vulnerabilities** — Do **not** open a public issue. See [SECURITY.md](SECURITY.md).

---

## Development Setup

### Requirements

| Tool | Version | Install |
|---|---|---|
| Rust | stable | `rustup update stable` |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| stellar-cli | latest | `cargo install --locked stellar-cli --features opt` |

### Clone and verify

```bash
git clone https://github.com/your-org/quorumforge-contract
cd quorumforge-contract
make test    # should pass with zero failures
make lint    # should produce zero warnings
make fmt     # should produce no diff
```

---

## Workflow

1. **Fork** the repository and create a branch from `main`:
   ```bash
   git checkout -b fix/double-sign-edge-case
   ```

2. **Make your changes.** Keep each PR focused on a single concern.

3. **Add or update tests.** Every bug fix needs a regression test. Every new feature needs at least one happy-path and one failure-path test.

4. **Run the full check suite locally:**
   ```bash
   make fmt lint test build
   ```

5. **Open a PR** against `main`. Fill in the PR template. Link the relevant issue.

6. **Address review feedback.** Push new commits; do not force-push after a review has started unless asked to.

7. PRs require at least **one approving review** from a maintainer before merging.

---

## Commit Style

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short description>

[optional body]
[optional footer]
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`

Examples:

```
feat(proposals): add UpdateThreshold payload type
fix(sign): reject double-sign atomically
docs(architecture): add storage layout table
test(expire): add ledger timestamp advancement test
```

---

## Testing Requirements

- All tests live in `src/test.rs` using `soroban-sdk`'s `testutils` feature.
- Use `env.mock_all_auths()` for auth mocking in tests; do not manually sign.
- Each PR must maintain or improve code coverage.
- Tests that expect a panic must use `#[should_panic(expected = "...")]` with the exact panic message.

### Running tests

```bash
cargo test                    # all tests
cargo test test_happy_path    # single test
cargo test -- --nocapture     # with stdout
```

---

## Documentation

- Public-facing functions must have Rust doc comments (`///`).
- Changes to contract behavior must be reflected in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
- New proposal types must be documented in [docs/PROPOSAL_TYPES.md](docs/PROPOSAL_TYPES.md).
- The README must be updated if the CLI interface changes.

---

## Security Issues

Please do **not** file public Issues for security vulnerabilities. Follow the process in [SECURITY.md](SECURITY.md).
