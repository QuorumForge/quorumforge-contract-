# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| `main` branch | ✅ |
| Tagged releases | ✅ (latest only) |
| Older releases | ❌ |

---

## Reporting a Vulnerability

**Do not open a public GitHub Issue for security vulnerabilities.**

Please send a private report to:

```
security@your-org.example
```

Include:

1. A description of the vulnerability and its potential impact.
2. Steps to reproduce (minimal PoC, testnet transaction IDs if applicable).
3. Affected versions.
4. Any suggested mitigations you have identified.

You will receive an acknowledgement within **48 hours** and a status update within **7 days**.

---

## Disclosure Policy

- We follow [responsible disclosure](https://en.wikipedia.org/wiki/Coordinated_vulnerability_disclosure).
- Reporters who follow this policy will be credited in the release notes (unless they prefer anonymity).
- We aim to patch critical issues within **14 days** of confirmed reproduction.
- We coordinate a public disclosure date with the reporter before publishing.

---

## Scope

In scope:

- Logic errors in contract functions (incorrect threshold enforcement, bypass of `require_auth`, double-execution)
- Storage manipulation that could corrupt `BoardConfig` or `Proposal` state
- Reentrancy or ordering issues in token transfer paths
- Denial-of-service vectors that could permanently brick a deployed contract

Out of scope:

- Stellar/Soroban protocol-level vulnerabilities (report these to the Stellar Development Foundation)
- Issues in third-party dependencies (report to the upstream maintainer)
- Issues requiring physical access to a key-holder's machine

---

## Bug Bounty

There is currently no formal bug bounty program. Exceptionally impactful reports may be rewarded at the maintainers' discretion.
