# Templar CLI — Multichain Implementation Plan

## Overview

Build `templar-cli`, a Rust CLI tool for interacting with Templar Protocol contracts and services across multiple blockchains. The CLI will support NEAR, Solana, Stellar, and EVM chains through Templar's Universal Account abstraction, and provide both direct on-chain reads and relayer-mediated write operations.

**Cross-chain asset support**: BTC, XRP, ADA, LTC, ZEC, DOGE, SOL, XLM, ETH, and ERC-20 tokens are bridged via two complementary systems:
- **Hot Bridge** (`v2_1.omni.hot.tg`) — NEP-245 multi-token bridge for assets like XLM, ZEC, and Stellar-based tokens. Uses `bridge-refuel.hot.tg` for gasless withdrawals. Assets are represented as NEP-245 multi-tokens on NEAR.
- **Intents Bridge** (Defuse/1click — `bridge.chaindefuser.com`) — NEP-141 OMFT bridge for BTC, DOGE, LTC, ADA, XRP, ETH, SOL, and ERC-20 tokens. Assets are represented as `*.omft.near` tokens (NEP-141) on NEAR.

Development follows **test-driven development (TDD)** throughout — tests are written before implementation for every module, targeting **95%+ code coverage**. Both **human-readable guide documentation** (mdbook) and **comprehensive Rust API docs** (rustdoc) are produced alongside the code.

## Architecture Decision

**Language**: Rust
**Rationale**: The existing ecosystem is predominantly Rust — contracts, relayer, monitoring, funding-bridge, and the existing `market-config-cli` tool are all Rust. This enables direct reuse of domain types and patterns.

**CLI Framework**: `clap` (derive mode) — consistent with `market-config-cli`
**NEAR Interaction**: `near-cli-rs` — used as the primary dependency for NEAR contract interaction (view calls, function calls, transaction construction, signing, credential management). The CLI wraps `near-cli-rs` capabilities into Templar-specific ergonomic commands.
**HTTP Client**: `reqwest` — consistent with existing services
**Interactive TUI**: `dialoguer` + `console` — consistent with `market-config-cli`
**Async Runtime**: `tokio`
**Testing**: `cargo-nextest` runner, `mockall` for trait mocking, `wiremock` for HTTP mocking, `cargo-llvm-cov` for coverage
**Documentation**: `mdbook` for user guide, `cargo doc` with `#![warn(missing_docs)]` for Rust API docs

### near-cli-rs Integration Strategy

`near-cli-rs` (crate: `near_cli_rs`) exposes a `lib.rs` with public modules including `commands`, `common`, `config`, `network`, and `types`. We use it as a Cargo dependency to leverage:

1. **Config/credentials**: Compatible with `~/.near-credentials/` and NEAR network configuration
2. **RPC layer**: The same `near-jsonrpc-client`, `near-primitives`, `near-crypto` crates it depends on
3. **Transaction construction**: NEAR transaction building, signing, and submission patterns

Where `near-cli-rs` internals are too coupled to its interactive prompt system, we use its underlying NEAR crates directly (`near-jsonrpc-client`, `near-primitives`, `near-crypto`, `near-token`, `near-gas`). All contract view/call functions from the Templar contracts are wrapped as typed, ergonomic CLI commands.

---

## Project Structure

```
templar-cli/
├── Cargo.toml
├── deny.toml                          # cargo-deny config for dep auditing
├── rustfmt.toml                       # Formatting config
├── clippy.toml                        # Clippy config
├── docs/                              # mdbook user guide
│   ├── book.toml
│   └── src/
│       ├── SUMMARY.md
│       ├── index.md                   # Introduction
│       ├── installation.md            # Install & setup
│       ├── configuration.md           # Config profiles & networks
│       ├── quickstart.md              # Quick start tutorial
│       ├── commands/
│       │   ├── index.md               # Command overview
│       │   ├── config.md              # templar config
│       │   ├── markets.md             # templar markets
│       │   ├── account.md             # templar account
│       │   ├── supply.md              # templar supply
│       │   ├── borrow.md              # templar borrow
│       │   ├── vault.md               # templar vault
│       │   ├── registry.md            # templar registry
│       │   ├── prices.md              # templar prices
│       │   ├── ua.md                  # templar ua
│       │   ├── bridge.md             # templar bridge (deposit/withdraw)
│       │   └── tx.md                  # templar tx
│       ├── multichain/
│       │   ├── index.md               # Multichain overview
│       │   ├── near.md                # NEAR direct signing
│       │   ├── solana.md              # Solana via Universal Account
│       │   ├── evm.md                 # EVM via Universal Account
│       │   └── stellar.md             # Stellar via Universal Account
│       ├── bridging/
│       │   ├── index.md               # Cross-chain bridging overview
│       │   ├── hot-bridge.md          # Hot Bridge (NEP-245) assets
│       │   ├── intents-bridge.md      # Intents/Defuse (NEP-141 OMFT) assets
│       │   └── supported-assets.md    # Full asset table with decimals, contract IDs
│       ├── architecture.md            # Internal architecture for contributors
│       └── glossary.md                # Term definitions (aligned with contracts glossary)
├── src/
│   ├── main.rs                        # Entry point, clap CLI parser
│   ├── lib.rs                         # Crate root: re-exports, #![warn(missing_docs)]
│   ├── error.rs                       # Unified error types (CliError enum)
│   ├── config/
│   │   ├── mod.rs                     # Config loading, saving, defaults
│   │   ├── network.rs                 # Network/chain definitions & NEAR chain IDs
│   │   └── profile.rs                 # Named profiles (mainnet, testnet, custom)
│   ├── types/
│   │   ├── mod.rs                     # Vendored domain types overview
│   │   ├── market.rs                  # MarketConfiguration, DepositMsg, Snapshot, positions
│   │   ├── vault.rs                   # VaultConfiguration, Fees, Restrictions, cap groups
│   │   ├── registry.rs               # Deployment, DeployMode, VersionEntry
│   │   ├── oracle.rs                  # OracleResponse, Pyth types, price types
│   │   ├── borrow.rs                  # BorrowPosition, BorrowStatus, collateral types
│   │   ├── supply.rs                  # SupplyPosition, WithdrawalRequestStatus
│   │   ├── number.rs                  # Decimal, amount types (BorrowAssetAmount, etc.)
│   │   ├── universal_account.rs       # KeyId, KeyParameters, PayloadExecutionParameters
│   │   └── bridge.rs                  # Bridge types: ChainId, TokenInfo, IntentsChain, etc.
│   ├── near/
│   │   ├── mod.rs                     # NEAR interaction layer overview
│   │   ├── rpc.rs                     # RPC client: view_function, view_account, tx_status
│   │   ├── signer.rs                  # NEAR key loading, compatible with ~/.near-credentials/
│   │   ├── tx_builder.rs             # Transaction construction: actions, gas, deposits
│   │   └── contract/
│   │       ├── mod.rs                 # Contract client trait + dispatch
│   │       ├── market.rs              # All MarketExternalInterface view/call wrappers
│   │       ├── vault.rs               # All VaultExternalInterface view/call wrappers
│   │       ├── registry.rs            # Registry contract view/call wrappers
│   │       ├── token.rs               # NEP-141 ft_transfer_call, ft_balance_of
│   │       ├── multi_token.rs         # NEP-245 mt_transfer_call, mt_balance_of
│   │       └── universal_account.rs   # UA contract: get_key, list_keys, execute
│   ├── bridge/
│   │   ├── mod.rs                     # Bridge layer overview, BridgeProvider trait
│   │   ├── chains.rs                  # Supported chains enum + chain metadata
│   │   ├── assets.rs                  # Asset registry: token → bridge route mapping
│   │   ├── hot.rs                     # Hot Bridge client (NEP-245 multi-token deposits/withdrawals)
│   │   ├── intents.rs                 # Intents/Defuse bridge client (OMFT NEP-141 deposits/withdrawals)
│   │   ├── deposit.rs                 # Unified deposit flow: get address → notify → track
│   │   ├── withdraw.rs               # Unified withdrawal flow: create intent → sign → submit
│   │   └── solver.rs                  # Solver relayer client (publish_intents, get_status)
│   ├── client/
│   │   ├── mod.rs                     # Client layer overview
│   │   ├── backend.rs                 # Backend gateway REST API client
│   │   ├── relayer.rs                 # Relayer API client (V0 + V1)
│   │   └── pyth.rs                    # Pyth/Hermes oracle price fetcher
│   ├── auth/
│   │   ├── mod.rs                     # Auth method dispatch (key type → signing method)
│   │   ├── near_wallet.rs             # NEAR ed25519 key-based signing
│   │   ├── solana.rs                  # Ed25519Raw (Solana keypair) signing
│   │   ├── evm.rs                     # EIP-191 (secp256k1) signing
│   │   ├── stellar.rs                 # Sep53 (Stellar ed25519) signing
│   │   └── keystore.rs               # Encrypted local key storage (argon2 + AES-256-GCM)
│   ├── commands/
│   │   ├── mod.rs                     # Top-level Commands enum
│   │   ├── config_cmd.rs             # `config` — init, show, import-key, list-keys
│   │   ├── markets.rs                 # `markets` — list, show, assets
│   │   ├── account.rs                 # `account` — positions, balances, health
│   │   ├── supply.rs                  # `supply` — deposit, withdraw, claim-yield
│   │   ├── borrow.rs                  # `borrow` — take, repay, add/withdraw collateral
│   │   ├── vault.rs                   # `vault` — deposit, withdraw, redeem, info, governance
│   │   ├── universal_account.rs       # `ua` — create, whoami, list-keys, add-key, remove-key
│   │   ├── prices.rs                  # `prices` — query oracle prices
│   │   ├── registry.rs               # `registry` — list, show
│   │   ├── bridge_cmd.rs             # `bridge` — deposit, withdraw, track, supported-assets
│   │   └── tx.rs                      # `tx` — status, history
│   ├── signing/
│   │   ├── mod.rs                     # Signing dispatch (per auth method)
│   │   ├── envelope.rs               # NEAR transaction envelope construction
│   │   ├── pow.rs                     # Proof-of-work for UA account creation
│   │   ├── relay.rs                   # Sign-and-relay flow (V0 + V1)
│   │   └── intents_signer.rs         # Intent signing for cross-chain withdrawals (NEP-413, raw_ed25519, sep53, erc191, webauthn)
│   ├── analytics/
│   │   ├── mod.rs                     # Analytics dispatch + opt-in/out management
│   │   ├── cli_usage.rs              # CLI usage telemetry (commands, errors, timing)
│   │   ├── contract_analytics.rs     # On-chain analytics (TVL, utilization, positions)
│   │   └── reporter.rs               # Background telemetry reporter
│   └── display/
│       ├── mod.rs                     # Output formatting dispatch (json vs table)
│       ├── table.rs                   # Table rendering for terminal
│       └── json.rs                    # JSON output mode
├── tests/
│   ├── common/
│   │   └── mod.rs                     # Shared test fixtures and helpers
│   ├── config_test.rs                 # Config loading, profile switching, defaults
│   ├── near_rpc_test.rs              # RPC client against mocked/testnet endpoints
│   ├── contract_market_test.rs       # Market contract view/call wrappers
│   ├── contract_vault_test.rs        # Vault contract view/call wrappers
│   ├── contract_registry_test.rs     # Registry contract wrappers
│   ├── contract_token_test.rs        # NEP-141 token wrappers
│   ├── contract_multi_token_test.rs  # NEP-245 multi-token wrappers
│   ├── bridge_hot_test.rs            # Hot Bridge client (wiremock)
│   ├── bridge_intents_test.rs        # Intents/Defuse bridge client (wiremock)
│   ├── bridge_deposit_test.rs        # Unified deposit flow tests
│   ├── bridge_withdraw_test.rs       # Unified withdrawal + intent signing tests
│   ├── backend_client_test.rs        # Backend API client (wiremock)
│   ├── relayer_client_test.rs        # Relayer API client (wiremock)
│   ├── auth_test.rs                   # Key import, signing, auth dispatch
│   ├── signing_test.rs               # Transaction signing, envelope construction
│   ├── analytics_test.rs             # Analytics collection and reporting
│   ├── display_test.rs               # Output formatting (table + JSON)
│   ├── types_serde_test.rs           # Serialization round-trips for vendored types
│   └── cli_integration_test.rs       # End-to-end CLI invocations via assert_cmd
└── README.md
```

---

## Testing & Coverage Strategy

### TDD Workflow

Every module follows **Red-Green-Refactor**:
1. **Red**: Write failing tests that define the expected behavior
2. **Green**: Write the minimum implementation to pass
3. **Refactor**: Clean up while keeping tests green

### Test Categories

| Category | Location | Runner | Purpose |
|----------|----------|--------|---------|
| **Unit tests** | `#[cfg(test)] mod tests` in each source file | `cargo nextest` | Test individual functions, parsing, formatting |
| **Integration tests** | `tests/*.rs` | `cargo nextest` | Test module interactions, mocked network calls |
| **Contract mock tests** | `tests/contract_*.rs` | `cargo nextest` + `wiremock` | Test NEAR RPC view/call wrappers against recorded responses |
| **Bridge mock tests** | `tests/bridge_*.rs` | `cargo nextest` + `wiremock` | Test bridge API clients against recorded JSON-RPC responses |
| **CLI E2E tests** | `tests/cli_integration_test.rs` | `assert_cmd` + `predicates` | Test actual CLI binary invocations |
| **Testnet integration** | `#[ignore]` tests in `tests/` | Manual / CI opt-in | Real network calls against NEAR testnet |

### Coverage Target: 95%+

- **Tool**: `cargo-llvm-cov` with `--html` report generation
- **CI gate**: Coverage check runs on every PR; build fails below 95%
- **Exclusions**: Only `main.rs` entry point and platform-specific code may be excluded via `#[cfg(not(tarpaulin_include))]`
- **Enforced via**: `.cargo/config.toml` alias: `[alias] cov = "llvm-cov nextest --html"`

### Mocking Strategy

- **Trait-based injection**: All external I/O goes through traits (`NearRpcClient`, `BackendClient`, `RelayerClient`, `PythClient`, `BridgeClient`, `SolverClient`)
- **`mockall`**: Auto-generate mock implementations for unit/integration tests
- **`wiremock`**: Mock HTTP servers for backend API, relayer, bridge JSON-RPC, and solver relayer tests
- **Test fixtures**: JSON files in `tests/fixtures/` with real contract/bridge responses captured from testnet/mainnet

### Test Infrastructure (Set up in Phase 1)

```rust
// Cargo.toml [dev-dependencies]
assert_cmd = "2"         // CLI binary testing
predicates = "3"         // Assertion matchers
wiremock = "0.6"         // HTTP mock server
mockall = "0.13"         // Trait mocking
tempfile = "3"           // Temp dirs for config tests
serde_json = "1"         // JSON fixture loading
tokio-test = "0.4"       // Async test utilities
insta = "1"              // Snapshot testing for display output
```

---

## Documentation Strategy

### 1. Rust API Docs (rustdoc)

Generated via `cargo doc --no-deps --document-private-items`.

**Standards** (enforced by `#![warn(missing_docs)]` in `lib.rs`):
- Every public type, trait, function, and module has a `///` doc comment
- Module-level `//!` docs explain purpose and show usage examples
- `# Examples` sections with ````rust` code blocks that compile and run as doctests
- Cross-references via `[`TypeName`]` intra-doc links
- `# Errors` section on all fallible functions
- `# Panics` section where applicable

**Style** (matching `templar-common` patterns):
```rust
/// Retrieve the current market configuration from a deployed contract.
///
/// Performs a NEAR RPC `view_function` call to the market contract's
/// `get_configuration` method.
///
/// # Arguments
///
/// * `market_id` - The NEAR account ID of the market contract
///
/// # Errors
///
/// Returns [`CliError::Rpc`] if the RPC call fails or the response
/// cannot be deserialized into [`MarketConfiguration`].
///
/// # Examples
///
/// ```no_run
/// # use templar_cli::near::contract::market::MarketClient;
/// # async fn example() -> Result<(), templar_cli::error::CliError> {
/// let client = MarketClient::new("https://rpc.mainnet.near.org")?;
/// let config = client.get_configuration("market.v1.tmplr.near".parse()?).await?;
/// println!("Borrow asset: {}", config.borrow_asset_id);
/// # Ok(())
/// # }
/// ```
pub async fn get_configuration(&self, market_id: AccountId) -> Result<MarketConfiguration, CliError> {
```

**CI enforcement**: `cargo doc --no-deps 2>&1 | grep -c "warning" | xargs test 0 -eq` (zero doc warnings)

### 2. User Guide (mdbook)

Located in `docs/`, built via `mdbook build docs/`, deployed alongside rustdoc.

**Style** (matching the existing Templar Protocol Guide at `contracts/docs/`):
- Clear section headers with practical examples
- Every command documented with: synopsis, description, arguments/flags, examples, related commands
- Cross-links to rustdoc for type details: `[MarketConfiguration](/doc/templar_cli/types/market/struct.MarketConfiguration.html)`
- Shell examples showing both interactive and non-interactive (scripting) usage
- Glossary aligned with `contracts/docs/src/glossary.md`

### 3. Documentation Build Commands

```bash
# Rust API docs
cargo doc --no-deps --document-private-items --open

# User guide
mdbook build docs/ --open

# Combined (CI)
cargo doc --no-deps && mdbook build docs/ && mdbook test docs/
```

---

## Phase 1: Foundation & Scaffolding

**Goal**: Project setup, config system, output formatting, test infrastructure — the skeleton everything else builds on. All code written TDD-first.

### 1.1 Project Initialization
- Initialize Cargo project with `[lib]` + `[[bin]]` targets
- Set up all dependencies (see Cargo.toml structure above)
- Configure `#![warn(missing_docs)]`, `#![warn(clippy::pedantic)]`
- Set up `rustfmt.toml`, `clippy.toml`, `deny.toml`
- Initialize mdbook in `docs/`
- Set up `cargo-llvm-cov` and coverage alias
- Create CI-ready `Makefile` or `justfile` with targets: `test`, `cov`, `doc`, `lint`, `fmt`
- **Tests**: verify project compiles, `--help` produces output, `--version` works

### 1.2 Error Types (`src/error.rs`)
- Define `CliError` enum: `Config`, `Rpc`, `Http`, `Bridge`, `Signing`, `InvalidInput`, `Io`, `Serialization`, `Interrupted`, `Other`
- `impl Display` with user-friendly messages
- `impl From<T>` for common error types
- **Tests**: error display formatting, error conversions, all variants round-trip through Display

### 1.3 Configuration System (`src/config/`)
- Config file at `~/.templar/config.toml` (or `$TEMPLAR_CONFIG`)
- `Profile` struct with all network settings (RPC URLs, contract IDs, chain IDs, bridge endpoints)
- Built-in `mainnet` and `testnet` profiles with sensible defaults:
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
  bridge_rpc_url = "https://bridge.chaindefuser.com/rpc"
  solver_relayer_url = "https://solver-relay.chaindefuser.com/rpc"
  hot_bridge_contract = "v2_1.omni.hot.tg"
  intents_contract = "intents.near"
  analytics_enabled = true
  analytics_endpoint = "https://analytics.templarfi.org/cli"
  ```
- CLI-level `--profile` and `--network` flags override defaults
- `templar config init` — interactive setup
- `templar config show` — display active configuration
- **Tests** (written first):
  - Default config creates valid mainnet profile
  - Config file round-trips through serialize/deserialize
  - CLI flag overrides config file values
  - `$TEMPLAR_CONFIG` env var changes config path
  - Missing config file creates defaults gracefully
  - Profile switching works
  - Invalid TOML produces clear error

### 1.4 Output Formatting (`src/display/`)
- `OutputFormat` enum: `Table`, `Json`
- `--output json` global flag
- Table renderer using formatted terminal output
- JSON renderer using `serde_json::to_string_pretty`
- `NO_COLOR` env var support
- **Tests** (written first):
  - JSON mode produces valid JSON for all output types
  - Table mode respects terminal width
  - `NO_COLOR` disables ANSI codes
  - Snapshot tests (via `insta`) for formatted output of each data type

### 1.5 Vendored Types (`src/types/`)
- Mirror key types from `templar-common` with matching serde serialization:
  - `MarketConfiguration`, `Snapshot`, `BorrowAssetMetrics`
  - `SupplyPosition`, `BorrowPosition`, `BorrowStatus`
  - `VaultConfiguration`, `Fees`, `Restrictions`
  - `Deployment`, `DeployMode`
  - `OracleResponse`, price types
  - `Decimal`, amount newtypes
  - `KeyId`, `KeyParameters`
- Bridge types (from `funding-bridge`):
  - `ChainId` (e.g., `"eth:1"`, `"btc:mainnet"`, `"stellar:mainnet"`)
  - `IntentsChain` enum: `Near`, `Eth`, `Btc`, `Sol`, `Xlm`, `Zec`, `Doge`, `Ada`, `Ltc`, `Xrp`
  - `TokenInfo`, `DepositAddressResult`, `DepositInfo`, `DepositStatus`
  - `Intent` enum: `FtWithdraw`, `MtWithdraw`, `Transfer`, `TokenDiff`
  - `SignedPayload`, `PayloadWrapper` (NEP-413)
- **Tests** (written first):
  - Deserialize real JSON responses captured from testnet (stored in `tests/fixtures/`)
  - Serialize/deserialize round-trip for every type
  - Edge cases: zero amounts, max values, empty optional fields
  - Bridge type parsing: ChainId parsing, IntentsChain → chain string mapping

### 1.6 Commands Skeleton (`src/commands/`)
- Top-level `Commands` enum with all subcommands registered
- Each command module stubbed with argument parsing + `todo!()` handler
- `templar config init` and `templar config show` fully implemented
- `templar health` implemented (simple HTTP GET)
- **Tests**:
  - CLI parses all subcommands correctly (`clap` arg validation)
  - `config init` creates valid config file
  - `config show` outputs current profile
  - `health` returns appropriate output for 200/500/unreachable

### 1.7 Documentation (Phase 1)
- mdbook skeleton with SUMMARY.md, index.md, installation.md, configuration.md, quickstart.md
- Rustdoc for all Phase 1 modules (error, config, display, types)
- README.md with install instructions, quick start, and link to docs

---

## Phase 2: NEAR Contract Operations

**Goal**: Wrap all Templar contract functionality as CLI commands using `near-cli-rs` / NEAR crates for direct on-chain interaction. This covers all read AND write operations that use standard NEAR signing (not multichain Universal Account).

### 2.1 NEAR RPC Client (`src/near/rpc.rs`)
- Trait `NearRpcClient` for testability:
  ```rust
  #[async_trait]
  pub trait NearRpcClient: Send + Sync {
      async fn view_function(&self, contract_id: &AccountId, method: &str, args: &[u8]) -> Result<Vec<u8>, CliError>;
      async fn view_account(&self, account_id: &AccountId) -> Result<AccountView, CliError>;
      async fn send_transaction(&self, signed_tx: SignedTransaction) -> Result<FinalExecutionOutcomeView, CliError>;
      async fn tx_status(&self, tx_hash: CryptoHash, sender_id: &AccountId) -> Result<FinalExecutionOutcomeView, CliError>;
      async fn access_key(&self, account_id: &AccountId, public_key: &PublicKey) -> Result<AccessKeyView, CliError>;
  }
  ```
- Implementation backed by `near-jsonrpc-client`
- Configurable RPC URL from profile
- Retry logic with exponential backoff for transient failures
- **Tests**:
  - Mock RPC responses for each method
  - Error handling: timeout, invalid response, contract not found
  - Retry logic triggers on 5xx, skips on 4xx

### 2.2 NEAR Transaction Builder (`src/near/tx_builder.rs`)
- Build `Transaction` with actions: `FunctionCall`, `Transfer`
- Gas estimation with configurable defaults (100 TGas for most calls)
- Deposit amount formatting (yoctoNEAR)
- Multi-action transactions (e.g., storage_deposit + ft_transfer_call)
- **Tests**:
  - Transaction with single FunctionCall action
  - Multi-action transaction ordering
  - Gas and deposit amounts serialize correctly
  - Block hash and nonce fetching

### 2.3 NEAR Signer (`src/near/signer.rs`)
- Load credentials from `~/.near-credentials/` (compatible with near-cli-rs)
- Support ed25519 key files (JSON format)
- Sign transactions and produce `SignedTransaction`
- **Tests**:
  - Load key from standard NEAR credentials path
  - Sign and verify transaction signature
  - Handle missing/invalid credential files gracefully

### 2.4 Market Contract Client (`src/near/contract/market.rs`)

Wraps every method from `MarketExternalInterface`:

**View calls (read-only):**
- `get_configuration(market_id)` → `MarketConfiguration`
- `get_current_snapshot(market_id)` → `Snapshot`
- `get_finalized_snapshots_len(market_id)` → `u32`
- `list_finalized_snapshots(market_id, offset, count)` → `Vec<Snapshot>`
- `get_borrow_asset_metrics(market_id)` → `BorrowAssetMetrics`
- `get_borrow_position(market_id, account_id)` → `Option<BorrowPosition>`
- `list_borrow_positions(market_id, offset, count)` → `HashMap<AccountId, BorrowPosition>`
- `get_borrow_position_pending_interest(market_id, account_id, snapshot_limit)` → `Option<BorrowAssetAmount>`
- `get_borrow_status(market_id, account_id, oracle_response)` → `Option<BorrowStatus>`
- `get_supply_position(market_id, account_id)` → `Option<SupplyPosition>`
- `list_supply_positions(market_id, offset, count)` → `HashMap<AccountId, SupplyPosition>`
- `get_supply_position_pending_yield(market_id, account_id, snapshot_limit)` → `Option<BorrowAssetAmount>`
- `get_supply_withdrawal_request_status(market_id, account_id)` → `Option<WithdrawalRequestStatus>`
- `get_supply_withdrawal_queue_status(market_id)` → `WithdrawalQueueStatus`
- `get_last_yield_rate(market_id)` → `Decimal`
- `get_static_yield(market_id, account_id)` → `Option<Accumulator>`

**Function calls (write, require NEAR signer):**
- `borrow(market_id, amount)` — direct function call
- `withdraw_collateral(market_id, amount)` — direct function call
- `create_supply_withdrawal_request(market_id, amount)` — direct function call
- `cancel_supply_withdrawal_request(market_id)` — direct function call
- `execute_next_supply_withdrawal_request(market_id, batch_limit)` — direct function call
- `harvest_yield(market_id, account_id, mode)` — direct function call
- `apply_interest(market_id, account_id, snapshot_limit)` — direct function call
- `accumulate_static_yield(market_id, account_id, snapshot_limit)` — direct function call
- `withdraw_static_yield(market_id, amount)` — direct function call

**Token transfer calls (require ft_transfer_call):**
- `supply(token_id, market_id, amount)` → `ft_transfer_call` with `msg: "\"Supply\""`
- `collateralize(token_id, market_id, amount)` → `ft_transfer_call` with `msg: "\"Collateralize\""`
- `repay(token_id, market_id, amount)` → `ft_transfer_call` with `msg: "\"Repay\""`
- `liquidate(token_id, market_id, amount, account_id)` → `ft_transfer_call` with `msg: "{\"Liquidate\":{\"account_id\":\"...\"}}"`

**Tests** (written first for each method):
- View call returns correct deserialized type from mock RPC
- Function call constructs correct transaction with proper gas/deposit
- ft_transfer_call constructs correct msg payload for each variant
- Error handling: contract not found, method not found, insufficient balance

### 2.5 Vault Contract Client (`src/near/contract/vault.rs`)

Wraps every method from `VaultExternalInterface`:

**View calls:**
- `get_configuration`, `get_total_assets`, `get_last_total_assets`, `get_total_supply`
- `get_max_deposit`, `convert_to_shares`, `convert_to_assets`
- `preview_deposit`, `preview_mint`, `preview_withdraw`, `preview_redeem`
- `get_cap_groups`, `get_fees`, `get_restrictions`

**Function calls:**
- `withdraw`, `redeem`, `deposit` (via ft_transfer_call)
- Governance: `set_curator`, `submit_cap`, `accept_cap`, `reallocate`, `set_supply_queue`, etc.

**Tests**: Same pattern as market — mock RPC, verify deserialization, verify tx construction

### 2.6 Registry Contract Client (`src/near/contract/registry.rs`)

- `list_versions(offset, count)` → `Vec<String>`
- `get_version_code_hash(version_key)` → `Option<Base58CryptoHash>`
- `list_deployments(offset, count)` → `Vec<AccountId>`
- `get_deployment(account_id)` → `Option<Deployment>`

**Tests**: Mock RPC responses, verify deserialization

### 2.7 Token Clients (`src/near/contract/token.rs`, `multi_token.rs`)

**NEP-141 (Fungible Token):**
- `ft_balance_of(token_id, account_id)` → `U128`
- `ft_metadata(token_id)` → `FungibleTokenMetadata`
- `ft_transfer_call(token_id, receiver_id, amount, msg)` → `Promise`
- `storage_deposit(token_id, account_id)` — ensure storage registered

**NEP-245 (Multi-Token — for Hot Bridge assets):**
- `mt_balance_of(contract_id, account_id, token_id)` → `U128`
- `mt_metadata(contract_id, token_ids)` → metadata
- `mt_transfer_call(contract_id, receiver_id, token_id, amount, msg)` → `Promise`

**Tests**: Standard NEP-141/245 responses, storage deposit flow

### 2.8 CLI Commands (Phase 2)

All contract operations exposed as CLI commands:

```
# Market read operations
templar markets list                                    # List markets from registry
templar markets show <market-id>                        # Full market config + metrics
templar markets snapshot <market-id>                    # Current snapshot
templar markets snapshots <market-id> [--count N]       # Historical snapshots
templar markets metrics <market-id>                     # Borrow asset metrics
templar markets yield-rate <market-id>                  # Current yield rate

# Account read operations
templar account positions <account-id>                  # All supply + borrow positions
templar account supply <market-id> <account-id>         # Supply position details
templar account borrow <market-id> <account-id>         # Borrow position details
templar account health <market-id> <account-id>         # Borrow health / MCR status
templar account pending-interest <market-id> <account-id> # Pending interest
templar account pending-yield <market-id> <account-id>  # Pending yield
templar account withdrawal-status <market-id> <account-id> # Withdrawal queue position
templar account balance <token-id> <account-id>         # NEP-141 token balance
templar account mt-balance <contract-id> <token-id> <account-id> # NEP-245 multi-token balance

# Supply write operations (NEAR direct signing)
templar supply deposit <market-id> <amount> --signer <id>
templar supply withdraw <market-id> <amount> --signer <id>
templar supply cancel-withdraw <market-id> --signer <id>
templar supply execute-withdraw <market-id> --signer <id>
templar supply harvest-yield <market-id> --signer <id>
templar supply claim-static-yield <market-id> --signer <id>

# Borrow write operations (NEAR direct signing)
templar borrow collateralize <market-id> <amount> --signer <id>
templar borrow take <market-id> <amount> --signer <id>
templar borrow repay <market-id> <amount> --signer <id>
templar borrow withdraw-collateral <market-id> <amount> --signer <id>
templar borrow apply-interest <market-id> [--account <id>] --signer <id>

# Vault operations (NEAR direct signing)
templar vault info <vault-id>
templar vault deposit <vault-id> <amount> --signer <id>
templar vault withdraw <vault-id> <amount> --signer <id>
templar vault redeem <vault-id> <shares> --signer <id>
templar vault preview-deposit <vault-id> <amount>
templar vault preview-redeem <vault-id> <shares>

# Registry operations
templar registry list [--count N] [--offset N]
templar registry show <account-id>
templar registry versions [--count N]

# Prices
templar prices <asset-id> [<asset-id>...]

# Transaction inspection
templar tx status <tx-hash> --signer <account-id>
```

### 2.9 Backend API Client (`src/client/backend.rs`)
- Alternative path for read operations (avoids direct RPC when backend is available)
- `GET /v1/health`, `GET /v1/markets`, `GET /v1/markets/{id}`, `GET /v1/assets`
- `GET /v1/prices?assetIds=...`
- `GET /v1/accounts/{account}/positions`, `GET /v1/accounts/{account}/balances`
- **Tests**: wiremock-based tests for each endpoint

### 2.10 Pyth/Hermes Client (`src/client/pyth.rs`)
- Fetch latest price updates from Hermes API
- Format oracle responses for contract calls
- **Tests**: wiremock-based tests with real Hermes response fixtures

### 2.11 Documentation (Phase 2)
- mdbook pages for every command group: markets, account, supply, borrow, vault, registry, prices, tx
- Each page mirrors the Templar Protocol Guide style with bash examples
- Rustdoc for all `near/` modules, `client/` modules, and `commands/` modules
- `docs/src/quickstart.md` with end-to-end walkthrough

---

## Phase 3: Multichain Authentication & Key Management

**Goal**: Support multichain wallet authentication for signing transactions via Universal Account.

### 3.1 Key Storage (`src/auth/keystore.rs`)
- Encrypted keystore at `~/.templar/keys/`
- Support importing keys for each chain type:
  - NEAR: ed25519 keypair (compatible with `~/.near-credentials/`)
  - Solana: ed25519 keypair (compatible with `~/.config/solana/id.json`)
  - EVM: secp256k1 private key (hex or keystore JSON)
  - Stellar: ed25519 secret key (S... format)
- Password-based encryption (argon2 + AES-256-GCM)
- **Tests** (written first):
  - Import and retrieve each key type
  - Encryption round-trip with correct password
  - Wrong password fails with clear error
  - File permissions are restrictive (0600)
  - Key listing shows types and aliases without exposing secrets

### 3.2 Auth Method Dispatch (`src/auth/mod.rs`)
- `AuthMethod` enum: `Near`, `Solana`, `Evm`, `Stellar`
- Map key type → signing method → relayer version:
  - NEAR key → standard NEAR transaction signing OR NEP-413 for intents
  - Solana key → `raw_ed25519` standard for solver relayer, `Ed25519Raw` for UA relayer V0
  - EVM key → `erc191` standard for solver relayer, `Eip191` for UA relayer V1
  - Stellar key → `sep53` standard for solver relayer, `Sep53` for UA relayer V1
- Auto-detect from active key type, or explicit `--auth-method` flag
- **Tests**: dispatch logic for each key type, correct standard selected

### 3.3 Chain-Specific Signing (`src/auth/{near_wallet,solana,evm,stellar}.rs`)
- NEAR: Standard ed25519 signing (direct transaction) + NEP-413 off-chain signing
- Solana: Ed25519Raw signature → raw payload signing
- EVM: EIP-191 personal_sign → with v-byte normalization (v >= 27 → v - 27)
- Stellar: Sep53 ed25519 signature → with address encoding (G... → XDR ScVal → base58)
- **Tests**: Sign and verify with test keys for each chain

### 3.4 Universal Account Lookup
- Query relayer: `GET /universal_account/account_id?type=<KeyType>&key=<pubkey>`
- Cache account ID locally
- `templar ua whoami` — show current account ID

### 3.5 CLI Commands (Phase 3)
```
templar config import-key --type <near|solana|evm|stellar> --source <path>
templar config list-keys
templar config set-active-key <alias>
templar config remove-key <alias>
templar ua whoami
```

### 3.6 Documentation (Phase 3)
- mdbook `multichain/` section: near.md, solana.md, evm.md, stellar.md
- Each page: key generation, import, verification, example flows
- Rustdoc for all `auth/` modules

---

## Phase 4: Transaction Signing & Relay + Cross-Chain Bridging

**Goal**: Sign and relay transactions through the Universal Account relayer AND enable cross-chain deposits/withdrawals via Hot Bridge + Intents Bridge.

### 4.1 Relayer Client (`src/client/relayer.rs`)
- V0 relay: `POST /relay` (Solana, Passkey)
- V1 relay: `POST /universal_account/relay` (Stellar, EVM)
- UA creation: `POST /universal_account/create`
- **Tests**: wiremock for relay endpoint, verify payload structure per version

### 4.2 Signing Envelope (`src/signing/envelope.rs`)
- V0: `\x19UAccount Signed Message:\n` + JSON payload (Ed25519Raw)
- V1: Versioned payload with `chain_id`, `salt`, `nonce` (Eip191, Sep53)
- Fetch key parameters (nonce, block_height, index) from UA contract
- **Tests**: envelope construction matches expected bytes for each version

### 4.3 Proof-of-Work (`src/signing/pow.rs`)
- Double-SHA256 PoW for UA creation
- Configurable difficulty prefix (default: "0000")
- Progress display during computation
- **Tests**: PoW produces valid proof, difficulty check works

### 4.4 Sign-and-Relay Flow (`src/signing/relay.rs`)
- Orchestrates: fetch nonce → build actions → construct envelope → sign → relay → poll status
- Transaction confirmation prompt (unless `--yes`)
- Display transaction hash and result
- **Tests**: full flow against mocked relayer + RPC

### 4.5 Intent Signing (`src/signing/intents_signer.rs`)

Sign intents for cross-chain withdrawals, supporting all auth methods:

| Auth Method | Standard | Payload Format | Signature Format |
|------------|----------|---------------|-----------------|
| NEAR wallet | `nep413` | Borsh-serialized `(OFFCHAIN_PREFIX_TAG, PayloadWrapper)` → SHA-256 → sign | `ed25519:<base58>` |
| Solana | `raw_ed25519` | Raw JSON string → sign bytes directly | `ed25519:<base58>` |
| Stellar | `sep53` | Pretty-printed JSON payload → sign | `ed25519:<base58>` |
| EVM | `erc191` | Pretty-printed JSON payload → personal_sign | `secp256k1:<base58>` (v normalized) |

- **Tests**: verify signed payload matches expected format for each auth method, round-trip deserialization

### 4.6 Cross-Chain Bridge Module (`src/bridge/`)

#### 4.6.1 Supported Chains & Assets (`src/bridge/chains.rs`, `assets.rs`)

Complete asset registry mapping each asset to its bridge route:

| Asset | Chain ID | Bridge | NEAR Token | Decimals | Type |
|-------|----------|--------|------------|----------|------|
| BTC | `btc:mainnet` | Intents | `btc.omft.near` | 8 | NEP-141 OMFT |
| XRP | `xrp:mainnet` | Intents | `xrp.omft.near` | 6 | NEP-141 OMFT |
| ADA | `cardano:mainnet` | Intents | `cardano.omft.near` | 6 | NEP-141 OMFT |
| LTC | `ltc:mainnet` | Intents | `ltc.omft.near` | 8 | NEP-141 OMFT |
| ZEC | `zec:mainnet` | Intents | `zec.omft.near` | 8 | NEP-141 OMFT |
| DOGE | `doge:mainnet` | Intents | `doge.omft.near` | 8 | NEP-141 OMFT |
| ETH | `eth:1` | Intents | `eth.omft.near` | 18 | NEP-141 OMFT |
| SOL | `sol:mainnet` | Intents | Various OMFT | varies | NEP-141 OMFT |
| XLM | `stellar:mainnet` | Hot Bridge | `v2_1.omni.hot.tg` | 7 | NEP-245 MT |
| USDC (Stellar) | `stellar:mainnet` | Hot Bridge | `v2_1.omni.hot.tg` | 7 | NEP-245 MT |
| USDC (ETH) | `eth:1` | Intents | `eth-0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.omft.near` | 6 | NEP-141 OMFT |
| USDT (ETH) | `eth:1` | Intents | `eth-0xdac17f958d2ee523a2206206994597c13d831ec7.omft.near` | 6 | NEP-141 OMFT |
| WBTC (ETH) | `eth:1` | Intents | `eth-0x2260fac5e5542a773aa44fbcfedf7c193bc2c599.omft.near` | 8 | NEP-141 OMFT |

- `AssetRegistry` struct with lookup by symbol, chain, and NEAR token ID
- `BridgeRoute` enum: `IntentsBridge` (NEP-141 OMFT) | `HotBridge` (NEP-245 MT)
- **Tests**: all asset lookups resolve correctly, NEAR token IDs match real contracts

#### 4.6.2 Intents Bridge Client (`src/bridge/intents.rs`)

Wraps the Defuse/1click bridge API at `bridge.chaindefuser.com/rpc`:

- `deposit_address(account_id, chain)` → `DepositAddressResult` (address + optional memo)
  - Stellar uses `deposit_mode: "MEMO"` for shared address + memo deposits
- `recent_deposits(account_id, chain, limit)` → `Vec<DepositInfo>`
- `notify_deposit(tx_hash, chain)` → acknowledgment
- `supported_tokens(chains)` → `Vec<TokenInfo>`
- `withdrawal_estimate(chain, token, address)` → fee + min amounts
- `withdrawal_status(withdrawal_hash)` → status tracking

**Withdrawal intent construction** (for NEP-141 OMFT assets):
```rust
Intent::FtWithdraw {
    token: "btc.omft.near",           // NEAR OMFT contract
    receiver_id: "btc.omft.near",     // Same as token for withdrawals
    amount: "100000000",              // In smallest units (satoshis)
    memo: "WITHDRAW_TO:<btc-address>" // Destination address
}
```

**Tests**: wiremock for all JSON-RPC endpoints, fixture-based response verification

#### 4.6.3 Hot Bridge Client (`src/bridge/hot.rs`)

Wraps the Hot Bridge for NEP-245 multi-token assets (XLM, Stellar-based tokens):

- Deposit: Same `deposit_address` API (bridge.chaindefuser.com) — assets arrive as NEP-245 tokens on `v2_1.omni.hot.tg`
- Withdrawal: Uses `mt_withdraw` intent via `bridge-refuel.hot.tg` for gasless bridging

**Withdrawal intent construction** (for NEP-245 MT assets):
```rust
Intent::MtWithdraw {
    token: "v2_1.omni.hot.tg",                   // Hot Bridge MT contract
    receiver_id: "bridge-refuel.hot.tg",          // Gasless bridge refuel
    token_ids: vec!["1100_111bzQBB5v7Ah..."],     // Stellar-specific token ID
    amounts: vec!["10000000"],                     // 1 XLM (7 decimals)
    memo: None,
    msg: Some(json!({                             // Gasless withdrawal payload
        "receiver_id": "<base58-encoded-stellar-address>",
        "amount_native": "0",
        "block_number": 0
    }).to_string()),
}
```

- Stellar address encoding: G... → XDR ScVal → base58 (matching `encode_receiver` from funding-bridge)
- **Tests**: verify mt_withdraw intent construction, stellar address encoding, token ID resolution

#### 4.6.4 Solver Relayer Client (`src/bridge/solver.rs`)

Submits signed intents to the solver relayer:

- `publish_intents(signed_data)` → intent hash
- `get_status(intent_hash)` → `PENDING | COMPLETED | FAILED`

The `signed_data` varies by auth method (see Intent Signing table in 4.5).

- **Tests**: wiremock for publish/status endpoints, auth method payloads

#### 4.6.5 Unified Deposit/Withdraw Flows (`src/bridge/deposit.rs`, `withdraw.rs`)

**Deposit flow**:
1. Determine bridge route from asset symbol → `IntentsBridge` or `HotBridge`
2. Call `deposit_address(account_id, chain)` to get deposit address (+ memo for Stellar)
3. Display QR code (via `qr2term`) + address + memo
4. Poll `recent_deposits` until deposit is confirmed
5. Display confirmation + NEAR tx hash

**Withdraw flow**:
1. Determine bridge route from asset symbol
2. Construct appropriate intent (`FtWithdraw` for Intents, `MtWithdraw` for Hot)
3. Sign using active auth method (NEP-413, raw_ed25519, sep53, or erc191)
4. Submit to solver relayer
5. Poll `get_status` until resolved
6. Display result

### 4.7 CLI Commands (Phase 4)

All Phase 2 write commands now work WITHOUT `--signer` using Universal Account:
```
# These now auto-detect auth method from active key
templar supply deposit <market-id> <amount>
templar borrow take <market-id> <amount>
templar vault deposit <vault-id> <amount>
# etc.

# UA management
templar ua create [--key-type <type>]
templar ua add-key <key-type> <pubkey>
templar ua remove-key <key-type> <pubkey>
templar ua list-keys <account-id>

# Cross-chain bridge operations
templar bridge supported-assets                          # List all supported bridgeable assets
templar bridge deposit-address <asset> [--chain <chain>] # Get deposit address (+ memo for XLM)
templar bridge deposit <asset> [--chain <chain>]         # Interactive deposit flow with QR + polling
templar bridge withdraw <asset> <amount> <destination-address> [--chain <chain>]  # Cross-chain withdrawal
templar bridge track <intent-hash>                       # Track withdrawal/deposit status
templar bridge recent-deposits [--chain <chain>]         # List recent deposits
templar bridge estimate-fee <asset> <destination-address> [--chain <chain>]  # Withdrawal fee estimate
```

### 4.8 Transaction Confirmation
- Show transaction summary before signing (method, contract, amounts, gas, auth method)
- For bridge operations: show bridge route, fees, estimated time
- `--yes` / `-y` flag to skip confirmation
- Display tx hash on success
- `templar tx status <tx-hash>` — query result

### 4.9 Documentation (Phase 4)
- mdbook `commands/bridge.md` — full bridge command reference
- mdbook `bridging/` section:
  - `index.md` — bridging overview, how assets flow cross-chain
  - `hot-bridge.md` — Hot Bridge mechanics, NEP-245 token format, gasless withdrawals
  - `intents-bridge.md` — Intents/Defuse mechanics, OMFT format, solver relayer
  - `supported-assets.md` — comprehensive table of all supported assets with decimals, contract IDs, and deposit/withdrawal instructions
- mdbook `commands/ua.md` and `commands/tx.md`
- Update all write command pages with UA usage examples
- Rustdoc for all `signing/` and `bridge/` modules

---

## Phase 5: Advanced Features, Analytics & Governance

**Goal**: Power-user features, governance operations, CLI usage analytics, on-chain contract analytics, and operational tooling.

### 5.1 CLI Usage Analytics (`src/analytics/cli_usage.rs`)

Opt-in telemetry for understanding how the CLI is used.

**What is tracked**:
- **Downloads/installations**: Version, platform, install method (cargo, binary, brew)
- **Command usage**: Which commands are run, how often, with which flags (no sensitive values)
- **Error rates**: Which commands fail, error categories (not error details)
- **Timing**: Command execution duration (bucketed: <1s, 1-5s, 5-30s, 30s+)
- **Session info**: Session duration, number of commands per session
- **Environment**: OS, architecture, terminal type, shell

**What is NOT tracked**:
- No account IDs, private keys, wallet addresses, or transaction data
- No contract IDs, amounts, or any user-specific financial information
- No IP addresses (if self-hosted, no network data at all)

**Implementation**:
```rust
/// Analytics event sent to the telemetry endpoint
pub struct CliEvent {
    pub event_type: EventType,   // "command_run", "command_error", "session_start", "session_end"
    pub command: String,         // e.g., "markets.list", "supply.deposit"
    pub flags: Vec<String>,      // sanitized flag names only (e.g., "--output", "--yes")
    pub duration_ms: u64,        // execution time
    pub success: bool,           // did it succeed?
    pub error_category: Option<String>, // e.g., "rpc_timeout", "invalid_input"
    pub cli_version: String,
    pub os: String,
    pub arch: String,
}
```

**Opt-in/out mechanism**:
- First run: prompt user "Help improve Templar CLI by sharing anonymous usage data? [y/N]"
- `templar config set analytics.enabled true|false`
- `TEMPLAR_ANALYTICS=0` env var to disable
- Config file: `analytics_enabled = true|false`
- Default: **disabled** (must explicitly opt in)

**Reporter** (`src/analytics/reporter.rs`):
- Background async task batches events, sends every 60s or on CLI exit
- Fire-and-forget: analytics failures never affect CLI functionality
- Endpoint configurable in profile: `analytics_endpoint`
- Local fallback: events written to `~/.templar/analytics.jsonl` if endpoint unreachable

**Tests**:
- Event construction produces valid JSON
- Opt-out prevents any data collection
- Reporter handles endpoint failures gracefully
- Sensitive data scrubbing (no account IDs leak through)
- Batch sending and flush-on-exit logic

### 5.2 Contract-Level Analytics (`src/analytics/contract_analytics.rs`)

On-chain analytics queries aggregating data across Templar contracts.

**Market Analytics**:
```
templar analytics markets                               # Overview of all markets
templar analytics market <market-id>                    # Detailed market analytics
```

Per-market metrics (computed from on-chain data):
- **TVL**: Total supply + total collateral (in USD via oracle prices)
- **Utilization rate**: borrowed / total_supply
- **APY**: Current supply yield rate, borrow interest rate
- **Borrow asset metrics**: total deposited, available, utilized
- **Position counts**: number of suppliers, number of borrowers
- **Withdrawal queue**: queue depth, pending amount
- **Historical**: snapshot-over-snapshot trends (from `list_finalized_snapshots`)

**Vault Analytics**:
```
templar analytics vault <vault-id>                      # Vault performance analytics
templar analytics vaults                                # All vaults overview
```

Per-vault metrics:
- **TVL**: `get_total_assets` in USD
- **Share price**: `convert_to_assets(1e18)` / 1e18 — share price trend
- **Deposit capacity**: `get_max_deposit`
- **Fee summary**: `get_fees` breakdown
- **Market allocation**: cap groups and current allocation vs caps

**Protocol-Wide Analytics**:
```
templar analytics protocol                              # Protocol-wide summary
```

Aggregated metrics:
- Total protocol TVL (sum of all markets + vaults)
- Total number of positions across all markets
- Total borrowed vs total supplied
- Registry deployment count

**Output formats**:
- `--output json` for machine-readable data (integration with dashboards, Grafana, etc.)
- `--output table` for human-readable terminal output
- `--output csv` for spreadsheet/analysis tools

**Tests**:
- Mock RPC responses for multi-market aggregation
- Correct USD conversion with oracle prices
- Utilization rate calculation edge cases (0 supply, full utilization)
- CSV and JSON output format validation
- Snapshot trend computation

### 5.3 Vault Governance Commands
```
templar vault set-curator <vault-id> <account>
templar vault submit-cap <vault-id> <market> <cap>
templar vault accept-cap <vault-id> <market>
templar vault revoke-cap <vault-id> <market>
templar vault set-supply-queue <vault-id> <market1,market2,...>
templar vault submit-guardian <vault-id> <account>
templar vault accept-guardian <vault-id>
templar vault set-fees <vault-id> <fees-json>
templar vault reallocate <vault-id> <delta-json>
templar vault skim <vault-id> <token-id>
```

### 5.4 Batch Operations
- `templar batch <file.json>` — execute multiple operations from JSON file
- Dry-run mode: `templar batch --dry-run <file.json>`
- Useful for automated workflows and CI/CD

### 5.5 Shell Completions
- `templar completions <bash|zsh|fish|powershell>` — generate shell completions
- Auto-complete contract IDs, account IDs from config

### 5.6 Documentation (Phase 5)
- mdbook pages for analytics, governance, batch, completions
- mdbook `analytics.md` — what's tracked, how to opt in/out, data privacy
- Architecture page for contributors
- Complete glossary

---

## Phase Summary & Dependencies

| Phase | Deliverables | Dependencies | Test Focus |
|-------|-------------|-------------|------------|
| **1: Foundation** | Project scaffold, config, display, vendored types, test infra | None | Config parsing, type serde, display formatting |
| **2: NEAR Contracts** | All Templar contract wrappers, NEAR signing, backend/Pyth clients | Phase 1 + NEAR crates | Mock RPC, contract call construction, CLI E2E |
| **3: Multichain Auth** | Key import/encrypt for 4 chains, auth dispatch, UA lookup | Phase 2 + crypto crates | Key round-trips, signing verification, dispatch |
| **4: UA Relay + Bridging** | Sign-and-relay, PoW, Hot Bridge + Intents Bridge, all cross-chain ops | Phase 3 + relayer + bridge APIs | Full relay flow, intent signing, bridge mocks |
| **5: Analytics + Advanced** | CLI telemetry, contract analytics, governance, batch, completions | Phase 4 + analytics endpoint | Analytics collection, aggregation, opt-in/out |

---

## Estimated Scope

| Phase | Source Files | Lines (approx) | Test Files | Test Lines (approx) |
|-------|-------------|----------------|------------|---------------------|
| 1: Foundation | ~12 | ~1,800 | ~5 | ~1,200 |
| 2: NEAR Contracts | ~15 | ~3,500 | ~8 | ~3,000 |
| 3: Multichain Auth | ~8 | ~1,500 | ~3 | ~1,200 |
| 4: UA Relay + Bridging | ~12 | ~3,500 | ~7 | ~3,000 |
| 5: Analytics + Advanced | ~8 | ~2,000 | ~4 | ~1,500 |

**Total**: ~55 source files, ~12,300 lines of implementation + ~27 test files, ~9,900 lines of tests + mdbook guide (~25 pages) + comprehensive rustdoc

**Coverage target**: 95%+ enforced in CI via `cargo-llvm-cov`
