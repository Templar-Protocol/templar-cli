# Templar CLI — Multichain Implementation Plan

## Overview

Build `templar-cli`, a Rust CLI tool for interacting with Templar Protocol contracts and services across multiple blockchains. The CLI will support NEAR, Solana, Stellar, and EVM chains through Templar's Universal Account abstraction, and provide both direct on-chain reads and relayer-mediated write operations.

## Architecture Decision

**Language**: Rust
**Rationale**: The existing ecosystem is predominantly Rust — contracts, relayer, monitoring, funding-bridge, and the existing `market-config-cli` tool are all Rust. This enables direct reuse of `templar-common` types (domain models, configs, number types) and `templar-universal-account` (key types, signing logic) as workspace-level or git dependencies, avoiding any translation layer.

**CLI Framework**: `clap` (derive mode) — consistent with `market-config-cli`
**HTTP Client**: `reqwest` — consistent with existing services
**NEAR RPC**: `near-jsonrpc-client` — consistent with existing tooling
**Interactive TUI**: `dialoguer` + `console` — consistent with `market-config-cli`
**Async Runtime**: `tokio`

---

## Project Structure

```
templar-cli/
├── Cargo.toml
├── src/
│   ├── main.rs                    # Entry point, clap CLI parser
│   ├── lib.rs                     # Re-exports, shared types
│   ├── config/
│   │   ├── mod.rs                 # Config loading & management
│   │   ├── network.rs             # Network/chain definitions
│   │   └── profile.rs             # Named profiles (mainnet, testnet, custom)
│   ├── client/
│   │   ├── mod.rs
│   │   ├── backend.rs             # Backend API client (gateway REST)
│   │   ├── near_rpc.rs            # Direct NEAR RPC client
│   │   ├── relayer.rs             # Relayer API client (V0 + V1)
│   │   └── pyth.rs                # Pyth oracle price fetcher
│   ├── auth/
│   │   ├── mod.rs                 # Auth method dispatch
│   │   ├── near_wallet.rs         # NEAR key-based signing
│   │   ├── solana.rs              # Ed25519Raw (Solana keypair)
│   │   ├── evm.rs                 # EIP-191 (EVM wallet/private key)
│   │   ├── stellar.rs             # Sep53 (Stellar keypair)
│   │   └── keystore.rs            # Encrypted local key storage
│   ├── commands/
│   │   ├── mod.rs                 # Command enum registration
│   │   ├── config_cmd.rs          # `config` — manage profiles, networks, keys
│   │   ├── markets.rs             # `markets` — list, inspect markets
│   │   ├── account.rs             # `account` — positions, balances, health
│   │   ├── supply.rs              # `supply` — deposit/withdraw supply
│   │   ├── borrow.rs              # `borrow` — borrow/repay operations
│   │   ├── vault.rs               # `vault` — deposit, withdraw, redeem, info
│   │   ├── universal_account.rs   # `ua` — create, list-keys, add-key
│   │   ├── prices.rs              # `prices` — query oracle prices
│   │   ├── registry.rs            # `registry` — list deployments, versions
│   │   └── tx.rs                  # `tx` — query transaction status
│   ├── signing/
│   │   ├── mod.rs                 # Signing dispatch (per auth method)
│   │   ├── envelope.rs            # NEAR transaction envelope construction
│   │   ├── pow.rs                 # Proof-of-work for account creation
│   │   └── relay.rs               # Sign-and-relay flow
│   ├── display/
│   │   ├── mod.rs                 # Output formatting utilities
│   │   ├── table.rs               # Table rendering for terminal
│   │   └── json.rs                # JSON output mode
│   └── error.rs                   # Unified error types
├── tests/
│   ├── integration/               # Integration tests against testnet/mocks
│   └── unit/                      # Unit tests
└── README.md
```

---

## Phase 1: Foundation (Read-Only CLI)

**Goal**: Project scaffolding + read-only queries via backend API and direct NEAR RPC.

### 1.1 Project Setup
- Initialize Cargo project with binary target
- Set up dependencies: `clap`, `tokio`, `reqwest`, `serde`, `serde_json`, `near-jsonrpc-client`, `near-primitives`, `near-sdk` (non-contract-usage), `dialoguer`, `console`, `base64`, `bs58`, `hex`, `thiserror`, `tracing`, `tracing-subscriber`
- Add git dependency on `templar-common` (from contracts repo) for shared domain types — OR vendor the minimal types needed (contract IDs, configuration structs, number types). **Recommendation**: vendor minimal types to keep the CLI repo self-contained and avoid coupling to the contracts monorepo build.
- Set up `tracing` with env-filter for debug logging

### 1.2 Configuration System
- Config file at `~/.templar/config.toml` (or `$TEMPLAR_CONFIG`)
- Named profiles: `mainnet` (default), `testnet`, custom
- Per-profile settings:
  ```toml
  [profiles.mainnet]
  near_rpc_url = "https://rpc.mainnet.fastnear.com"
  backend_url = "https://api.templarfi.org"
  relayer_v0_url = "https://relayer.templarfi.org"
  relayer_v1_url = "https://relayer.templarfi.org:4001"
  near_network_id = "mainnet"
  near_chain_id = 397
  registry_contract_ids = ["v1.tmplr.near"]
  hermes_url = "https://hermes.pyth.network"

  [profiles.testnet]
  near_rpc_url = "https://rpc.testnet.fastnear.com"
  backend_url = "http://localhost:3200"
  relayer_v0_url = "http://localhost:3000"
  relayer_v1_url = "http://localhost:3000"
  near_network_id = "testnet"
  near_chain_id = 398
  registry_contract_ids = ["templar-alpha.near"]
  hermes_url = "https://hermes-beta.pyth.network"
  ```
- CLI-level `--profile` and `--network` flags override defaults
- `templar config init` — interactive setup
- `templar config show` — display active configuration

### 1.3 Backend API Client
Wrap the gateway REST API:
- `GET /v1/health` → `templar health`
- `GET /v1/markets` → `templar markets list`
- `GET /v1/markets/{id}` → `templar markets show <id>`
- `GET /v1/assets` → `templar markets assets`
- `GET /v1/prices?assetIds=...` → `templar prices <asset-ids...>`
- `GET /v1/accounts/{account}/positions` → `templar account positions <account>`
- `GET /v1/accounts/{account}/balances` → `templar account balances <account>`

### 1.4 Direct NEAR RPC Client
For direct on-chain queries (no backend dependency):
- `view_function(contract_id, method, args)` — generic view call
- `view_account(account_id)` — account balance/state
- Wrappers for key contract methods:
  - `registry.list_deployments()`
  - `market.get_configuration()`
  - `market.get_current_snapshot()`
  - `market.get_supply_position(account_id)`
  - `market.get_borrow_position(account_id)`
  - `market.get_borrow_status(account_id, oracle_response)`
  - `vault.get_configuration()`
  - `vault.get_total_assets()`
  - `universal_account.get_key(key_id)`
  - `universal_account.list_keys()`

### 1.5 Output Formatting
- `--output json` flag for machine-readable output (all commands)
- Default: human-friendly table/formatted output
- Color support via `console` crate (respects `NO_COLOR` env var)

### 1.6 Commands (Phase 1)
```
templar health                          # Check backend health
templar config init                     # Interactive config setup
templar config show                     # Show active config

templar markets list [--domain <d>]     # List markets (optionally by domain: btc, zec, etc.)
templar markets show <market-id>        # Market details: config, APYs, utilization, snapshot
templar markets assets                  # List known assets

templar account positions <account-id>  # All supply/borrow positions with USD values
templar account balances <account-id>   # Wallet token balances
templar account health <account-id> <market-id>  # Borrow health factor

templar prices <asset-id> [<asset-id>...] # Oracle prices

templar registry list                   # List deployed market contracts
templar registry show <account-id>      # Deployment details

templar ua list-keys <account-id>       # List keys on a universal account
```

---

## Phase 2: Authentication & Key Management

**Goal**: Support multichain wallet authentication for signing transactions.

### 2.1 Key Storage
- Encrypted keystore at `~/.templar/keys/`
- Support importing keys for each chain type:
  - NEAR: ed25519 keypair (compatible with `~/.near-credentials/`)
  - Solana: ed25519 keypair (compatible with `~/.config/solana/id.json`)
  - EVM: secp256k1 private key (hex or keystore JSON)
  - Stellar: ed25519 secret key (S... format)
- `templar config import-key --type <near|solana|evm|stellar> --source <path-or-value>`
- `templar config list-keys`
- `templar config set-active-key <key-alias>`
- Password-based encryption (argon2 + AES-256-GCM) for local key files

### 2.2 Auth Method Dispatch
- Map key type → signing method:
  - NEAR key → standard NEAR transaction signing
  - Solana key → Ed25519Raw (V0 relayer)
  - EVM key → Eip191 (V1 relayer)
  - Stellar key → Sep53 (V1 relayer)
- `--auth-method` flag to explicitly select method
- Auto-detect from active key type if not specified

### 2.3 Universal Account Lookup
- Given a key + type, query the relayer for the associated UA account ID:
  `GET /universal_account/account_id?type=<KeyType>&key=<pubkey>`
- Cache account ID locally to avoid repeated lookups
- `templar ua whoami` — show current account ID based on active key

---

## Phase 3: Transaction Signing & Relay (Write Operations)

**Goal**: Sign and relay transactions through the Universal Account relayer.

### 3.1 Transaction Building
- Build NEAR action payloads (function calls with gas + deposit)
- Multi-action transaction support (storage deposit → transfer → operation)
- Amount formatting with token decimal handling

### 3.2 Signing Flow
- Construct message payload per auth method:
  - V0 (Solana): `\x19UAccount Signed Message:\n` + JSON payload
  - V1 (Stellar Sep53, EVM Eip191): versioned payload with chain_id + salt
- Fetch key parameters (nonce, block_height, index) from UA contract
- Sign with local key
- Submit to relayer: `POST /universal_account/relay`

### 3.3 Proof-of-Work (Account Creation)
- Implement double-SHA256 PoW for UA creation
- `POST /universal_account/create` with PoW proof
- `templar ua create` — create new universal account

### 3.4 Write Commands
```
templar supply deposit <market-id> <amount>     # Supply assets to market
templar supply withdraw <market-id> <amount>    # Withdraw supply

templar borrow take <market-id> <amount>        # Borrow from market
templar borrow repay <market-id> <amount>       # Repay borrowed amount
templar borrow add-collateral <market-id> <amount>    # Add collateral
templar borrow withdraw-collateral <market-id> <amount> # Withdraw collateral

templar vault deposit <vault-id> <amount>       # Deposit into vault
templar vault withdraw <vault-id> <amount>      # Withdraw from vault
templar vault redeem <vault-id> <shares>        # Redeem vault shares

templar ua create                               # Create universal account
templar ua add-key <key-type> <pubkey>          # Add signing key
templar ua remove-key <key-type> <pubkey>       # Remove signing key
```

### 3.5 Transaction Confirmation
- Show transaction summary before signing (amounts, gas, method)
- `--yes` flag to skip confirmation prompt
- Display transaction hash on success
- `templar tx status <tx-hash>` — query transaction result

---

## Phase 4: Advanced Features

**Goal**: Power-user features, governance operations, and operational tooling.

### 4.1 Vault Governance Commands
```
templar vault info <vault-id>                           # Full vault config + state
templar vault set-curator <vault-id> <account>          # Curator role assignment
templar vault submit-cap <vault-id> <market> <cap>      # Submit market cap change
templar vault accept-cap <vault-id> <market>            # Accept pending cap
templar vault reallocate <vault-id> <delta-json>        # Rebalance capital
```

### 4.2 Interest & Yield Operations
```
templar borrow apply-interest <market-id> <account>     # Apply pending interest
templar supply claim-yield <market-id>                  # Claim accrued yield
```

### 4.3 Registry Operations (Admin)
```
templar registry add-version <version-key> <wasm-path>  # Register new version
```

### 4.4 Cross-Chain Deposit Tracking
- Integration with Intents bridge API
- `templar deposit track <chain> <tx-hash>` — track cross-chain deposit status
- Display deposit address generation for supported chains (BTC, SOL, XLM)

### 4.5 Monitoring Integration
- `templar monitor balance <account-id>` — check funder account balance
- `templar monitor screen <account-id> <chain>` — TRM screening check (if API access available)

### 4.6 Batch Operations
- `templar batch <file.json>` — execute multiple operations from a JSON file
- Useful for automated workflows and scripting

---

## Implementation Notes

### Dependency Strategy
Rather than importing `templar-common` as a git dependency (which would pull the entire contracts workspace), we will:
1. **Vendor minimal type definitions** for contract method args/return types (MarketConfiguration, Deployment, position types, etc.)
2. Define them in a `src/types/` module with matching serde serialization
3. This keeps the CLI repo self-contained and avoids cargo workspace coupling

### Error Handling
- Use `thiserror` for typed errors
- Unified `CliError` enum covering: config errors, network errors, RPC errors, signing errors, user cancellation
- User-facing error messages via `tracing` + formatted output
- Debug-level logging for troubleshooting

### Testing Strategy
- **Unit tests**: Config parsing, amount formatting, key serialization
- **Integration tests**: NEAR RPC queries against testnet (behind `#[ignore]` by default)
- **Mock tests**: Backend API client against recorded responses

### Security Considerations
- Private keys never logged or displayed
- Encrypted keystore with password protection
- Transaction previews before signing
- Gas limits enforced on all transactions
- No plaintext key storage

---

## Phase Summary & Dependencies

| Phase | Deliverables | Dependencies |
|-------|-------------|-------------|
| 1 | Read-only CLI: config, markets, positions, balances, prices | Backend API, NEAR RPC |
| 2 | Auth & key management: import, encrypt, lookup UA | Phase 1 + local keystore |
| 3 | Write ops: supply, borrow, vault, UA management | Phase 2 + relayer API |
| 4 | Governance, monitoring, batch, cross-chain tracking | Phase 3 + bridge API |

---

## Estimated Scope

- **Phase 1**: ~15 source files, ~2500 lines — full read-only CLI
- **Phase 2**: ~8 source files, ~1500 lines — auth + key management
- **Phase 3**: ~8 source files, ~2000 lines — transaction building + relay
- **Phase 4**: ~6 source files, ~1500 lines — advanced features

Total: ~37 source files, ~7500 lines of Rust
