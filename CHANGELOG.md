# Changelog

All notable changes to `estamora-contracts` are recorded here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-07

### Added
- **Core Escrow Engine (`contracts/estamora-payments`)**:
  - `create_escrow`: Multi-party milestone escrow with configurable timeout and mediator.
  - `release_milestone`: Progressive milestone payouts triggered by payer or designated authority.
  - `refund_escrow`: Automatic timeout refunds protecting payers against unfulfilled merchant agreements.
  - `open_dispute` & `resolve_dispute`: Built-in arbitration and dispute resolution with custom payout splits.
- **Agent Policy Firewall & Delegated Spend Caps**:
  - `set_spend_cap`: Rolling 24-hour spending limits with strict caller authorization.
  - `execute_delegated_payment`: Policy-enforced autonomous execution verifying accumulated windows and spend limits.
- **Events and Telemetry**:
  - Structured Soroban topics for `escrow_created`, `milestone_released`, `refunded`, `disputed`, `resolved`, `spend_cap_set`, and `delegated_paid`.
- **Test Suite**:
  - End-to-end Rust Soroban tests covering normal lifecycle, dispute arbitration, timeout refunds, and spend cap enforcement.
- **Deployments**:
  - Release WebAssembly artifact built for `wasm32v1-none` (26.2 KB).
  - Testnet deployment configuration recorded under `deployments/testnet.json`.
