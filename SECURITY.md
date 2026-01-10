# Security Policy

Security is a foundational pillar of the LogLine Protocol.

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | ✅ Active support  |
| 0.2.x   | 🔶 Security fixes  |
| < 0.2   | ❌ End of life     |

## Reporting a Vulnerability

**Please do not open public issues for security vulnerabilities.**

### How to Report

- **Email:** security@logline.foundation
- **GitHub Security Advisory:** Open a private report via GitHub

### What to Include

- Affected crate(s) and version(s)
- Environment (OS, Rust version, features enabled)
- Steps to reproduce
- Expected impact

### Response Timeline

- **Acknowledgment:** Within 5 business days
- **Severity Classification:** Within 10 business days
- **Fix Timeline:** Depends on severity
  - Critical: 7 days
  - High: 14 days
  - Medium: 30 days
  - Low: Next release

## Scope

Security issues include:

- **Integrity violations:** Invalid state transitions, broken invariants
- **Canonicalization bugs:** Different inputs producing same CID, or same input producing different CIDs
- **Signature bypasses:** Invalid signatures being accepted, valid signatures being rejected
- **Domain separation failures:** Cross-domain replay attacks (SIRP vs UBL vs TDLN)
- **Memory safety:** Panics on valid inputs, memory leaks, DoS vectors
- **Cryptographic weaknesses:** Weak randomness, timing attacks, key leakage

## Domain Separation Constants

All signing operations use domain-specific prefixes:

| Domain | Constant |
| ------ | -------- |
| SIRP   | `SIRP:FRAME:v1` |
| UBL    | `UBL:LEDGER:v1` |
| TDLN   | `TDLN:PROOF:v1` |

## Versioning

- Patches are released as `x.y.z` (semver)
- Security advisories are published in release notes
- CVEs are requested for critical vulnerabilities
