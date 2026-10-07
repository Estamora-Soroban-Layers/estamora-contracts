# Security Policy

## Reporting a Vulnerability

Security is paramount for on-chain smart contracts managing customer funds and autonomous agent allowances.

If you discover a security vulnerability in `estamora-contracts`:
1. **Do not open a public GitHub issue.**
2. Report the vulnerability privately via GitHub Security Advisories on this repository or contact the core maintainers.
3. Provide detailed steps to reproduce the issue, contract version, and a proof of concept if possible.

## Smart Contract Security Principles

- **Authorization**: All state modifications require strict cryptographic authorization (`payer.require_auth()`, `mediator.require_auth()`, etc.).
- **Escrow Invariants**: Escrowed balances can never be released without either:
  1. Explicit payer/authority milestone release,
  2. Timestamp expiry beyond timeout block for refund, or
  3. Formally arbitrated dispute resolution by the designated mediator.
- **Agent Limits**: Delegated spend allowances are strictly constrained by 24-hour rolling windows and maximum spend caps.
