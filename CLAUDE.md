# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**templar-cli** is a Rust CLI for interacting with Templar Protocol — a multichain cypher lending protocol on NEAR that lets users borrow against native BTC, ZEC, XRP, ADA, DOGE, XLM, LTC, and more without trusting centralized intermediaries. It supports NEAR, EVM, Solana, Stellar, and Passkey transaction signing via Templar's Universal Account abstraction, with both direct on-chain reads and relayer-mediated write operations.

This is a greenfield project. The implementation plan is in `PLAN.md`. Development follows strict TDD (Red-Green-Refactor) with a 95%+ coverage target.

## Build & Development Commands

```bash
# Build
cargo build
cargo build --release

# Test (uses cargo-nextest)
cargo nextest run                        # Run all tests
cargo nextest run <test_name>            # Run a single test
cargo nextest run --test cli_integration_test  # Run a specific test file
cargo test --doc                         # Run doctests (nextest doesn't support these)

# Coverage
cargo llvm-cov nextest --html            # Generate HTML coverage report (alias: cargo cov)

# Lint & Format
cargo clippy --all-targets -- -D warnings
cargo fmt --check                        # Check formatting
cargo fmt                                # Apply formatting

# Documentation
cargo doc --no-deps --document-private-items --open   # Rust API docs
mdbook build docs/ --open                              # User guide
mdbook test docs/                                      # Test mdbook code examples

# Dependency auditing
cargo deny check
```

## Architecture

### Language & Key Dependencies

- **Language**: Rust (matches the existing Templar ecosystem — contracts, relayer, monitoring)
- **CLI framework**: `clap` (derive mode)
- **Async runtime**: `tokio`
- **NEAR interaction**: `near-jsonrpc-client`, `near-primitives`, `near-crypto` (via `near-cli-rs` as dependency where practical)
- **HTTP**: `reqwest`
- **TUI**: `dialoguer` + `console` + `indicatif`
- **Testing**: `cargo-nextest`, `mockall`, `wiremock`, `insta` (snapshots), `assert_cmd`

### Source Layout (`src/`)

The crate has both `lib.rs` and `main.rs` targets. `#![warn(missing_docs)]` and `#![warn(clippy::pedantic)]` are enabled.

- **`commands/`** — One file per top-level CLI subcommand (markets, account, supply, borrow, vault, bridge, ua, prices, registry, tx, config, batch). Each maps clap args to operations.
- **`near/`** — NEAR blockchain interaction layer:
  - `rpc.rs` — `NearRpcClient` trait with retry/backoff logic (200ms initial, 2x multiplier, 5s cap, 3 retries). View calls retry on 5xx/timeout; transaction sends poll `tx_status` on timeout instead of re-sending.
  - `contract/` — Typed wrappers for each Templar contract (market, vault, registry, token, multi_token, universal_account). Every method from `MarketExternalInterface` and `VaultExternalInterface` is wrapped.
  - `signer.rs` — Loads credentials from `~/.near-credentials/`
  - `tx_builder.rs` — NEAR transaction construction
- **`bridge/`** — Cross-chain bridging via NEAR Intents system:
  - All cross-chain assets are **NEP-245 multi-tokens** within `intents.near`. Token IDs encode underlying OMFT contracts (e.g., `nep141:btc.omft.near`).
  - `BridgeRoute` enum: `IntentsOmft` (most assets, `FtWithdraw` intent) vs `IntentsHotMt` (Stellar assets, `MtWithdraw` intent via `bridge-refuel.hot.tg`)
  - Single bridge API at `bridge.chaindefuser.com/rpc` (JSON-RPC 2.0)
- **`client/`** — HTTP clients for backend gateway, relayer (V0 + V1), and Pyth/Hermes oracle
- **`auth/`** — Multichain authentication: NEAR (ed25519/NEP-413), Solana (Ed25519Raw), EVM (EIP-191), Stellar (Sep53). Key storage uses Argon2id + AES-256-GCM encryption in `~/.templar/keys/`.
- **`signing/`** — Transaction envelope construction, proof-of-work for UA creation, sign-and-relay orchestration, intent signing per auth method
- **`types/`** — Vendored domain types from `templar-common` with matching serde. Includes market, vault, registry, oracle, borrow, supply, number (Decimal, amount newtypes), universal_account, and bridge types.
- **`display/`** — Themed terminal output:
  - `theme.rs` — Templar brand palette (Gold `#D5AA51`, Ivory `#E8E1D3`, Antique Gold `#AE8227`, Warm Grey `#A59B89`, Cipher Purple `#963CDC`). Auto-fallback: truecolor → 256-color → basic ANSI. Respects `NO_COLOR` env var.
  - `banner.rs` — Full 100x57 ASCII art Templar mark (wide terminals) or compact `✠` fallback
  - `spinner.rs` — Binary noise spinner, text-scramble reveal for hashes
  - `frame.rs` — Unicode box-drawing (heavy `┏━┓` for panels, light `┌─┐` for tables)
  - `Voice` enum: `Cypherpunk` ("Sealing transaction...") vs `Standard` ("Submitting transaction...")
- **`config/`** — TOML config at `~/.templar/config.toml` (or `$TEMPLAR_CONFIG`). Named profiles (mainnet, testnet, custom). Config files created with mode `0o600`, directory with `0o700`.
- **`error.rs`** — Unified `CliError` enum (Config, Rpc, Http, Bridge, Signing, InvalidInput, Io, Serialization, Interrupted, Other)
- **`analytics/`** — Opt-in CLI usage telemetry + on-chain contract analytics aggregation

### Testing Architecture

- All external I/O goes through traits (`NearRpcClient`, `BackendClient`, `RelayerClient`, `PythClient`, `BridgeClient`, `SolverClient`) for `mockall`-based mocking
- `wiremock` for HTTP mock servers (backend, relayer, bridge JSON-RPC, solver)
- Test fixtures: JSON files in `tests/fixtures/` with real contract/bridge responses
- `insta` snapshot tests for display output
- `assert_cmd` for CLI E2E tests
- `#[ignore]` tests for real testnet integration (opt-in in CI)

### Relayer Integration

The relayer uses NO API key — authentication is via cryptographic signature verification + gas allowance. Responses follow a three-tier model:
- `Success` (200) — operation completed
- `Rejected` (400) — NEVER retry (invalid signature, insufficient allowance)
- `TransientFailure` (500) — retry with exponential backoff

V0 relay (`POST /relay`) handles Solana/Passkey. V1 relay (`POST /universal_account/relay`) handles Stellar/EVM with `chain_id`.

### Config Defaults

Mainnet profile endpoints:
- NEAR RPC: `https://rpc.mainnet.fastnear.com`
- Backend: `https://api.templarfi.org`
- Relayer V0: `https://relayer.templarfi.org`
- Relayer V1: `https://relayer.templarfi.org:4001`
- Bridge RPC: `https://bridge.chaindefuser.com/rpc`
- Solver: `https://solver-relay.chaindefuser.com/rpc`
- Hermes: `https://hermes.pyth.network`
- Registry contracts: `v1.tmplr.near`
- Intents contract: `intents.near`

## Conventions

- All Rust API docs use `///` with `# Examples`, `# Errors`, and `# Panics` sections
- Cypherpunk-themed UX copy is the default ("Forging your configuration...", "Sealing transaction...") with a `Standard` voice alternative
- The Templar cross `✠` is the primary prompt prefix and section separator
- `--output json` global flag for machine-readable output on all commands
- `--yes` / `-y` skips confirmation; high-risk operations always confirm unless `--yes --force`
- `--quiet` / `-q` suppresses banner and non-essential output
- `--direct` flag bypasses relayer for direct on-chain NEAR transaction submission
