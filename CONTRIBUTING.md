# Contributing to Estamora Contracts

Thank you for contributing to `estamora-contracts`! This repository houses the core Stellar Soroban smart contracts powering Estamora's milestone escrow, timeout protection, dispute arbitration, and delegated spend limit firewalls.

## Development Workflow

### Prerequisites
- **Rust**: 1.80+ (`rustup default stable`)
- **WebAssembly target**: `rustup target add wasm32v1-none`
- **Soroban CLI** (optional for local deployment simulation): `stellar-cli` / `soroban-cli`

### Clone & Test
```bash
git clone https://github.com/Estamora-Soroban-Layers/estamora-contracts.git
cd estamora-contracts

# Run unit tests
cargo test

# Build release WebAssembly
cargo build --target wasm32v1-none --release -p estamora-payments
```

## Pull Request Guidelines

1. **Test Coverage**: All contract functions (`create_escrow`, `release_milestone`, `refund_escrow`, `open_dispute`, `resolve_dispute`, `set_spend_cap`, `execute_delegated_payment`) must maintain 100% unit test coverage.
2. **Deterministic State**: Never rely on nondeterministic timestamps or unconstrained host state.
3. **Event Emission**: Ensure critical contract transitions emit appropriate Soroban topics.
4. **Code Formatting**: Format code using `cargo fmt` before submitting PRs.
