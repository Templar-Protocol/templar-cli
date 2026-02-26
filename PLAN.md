# Templar CLI — Multichain Implementation Plan

## Overview

Build `templar-cli`, a Rust CLI tool for interacting with Templar Protocol contracts and services across multiple blockchains. The CLI will support NEAR, Solana, Stellar, and EVM chains through Templar's Universal Account abstraction, and provide both direct on-chain reads and relayer-mediated write operations.

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
│       │   └── tx.md                  # templar tx
│       ├── multichain/
│       │   ├── index.md               # Multichain overview
│       │   ├── near.md                # NEAR direct signing
│       │   ├── solana.md              # Solana via Universal Account
│       │   ├── evm.md                 # EVM via Universal Account
│       │   └── stellar.md             # Stellar via Universal Account
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
│   │   └── universal_account.rs       # KeyId, KeyParameters, PayloadExecutionParameters
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
│   │       └── universal_account.rs   # UA contract: get_key, list_keys, execute
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
│   │   └── tx.rs                      # `tx` — status, history
│   ├── signing/
│   │   ├── mod.rs                     # Signing dispatch (per auth method)
│   │   ├── envelope.rs               # NEAR transaction envelope construction
│   │   ├── pow.rs                     # Proof-of-work for UA account creation
│   │   └── relay.rs                   # Sign-and-relay flow (V0 + V1)
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
│   ├── backend_client_test.rs        # Backend API client (wiremock)
│   ├── relayer_client_test.rs        # Relayer API client (wiremock)
│   ├── auth_test.rs                   # Key import, signing, auth dispatch
│   ├── signing_test.rs               # Transaction signing, envelope construction
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
| **CLI E2E tests** | `tests/cli_integration_test.rs` | `assert_cmd` + `predicates` | Test actual CLI binary invocations |
| **Testnet integration** | `#[ignore]` tests in `tests/` | Manual / CI opt-in | Real network calls against NEAR testnet |

### Coverage Target: 95%+

- **Tool**: `cargo-llvm-cov` with `--html` report generation
- **CI gate**: Coverage check runs on every PR; build fails below 95%
- **Exclusions**: Only `main.rs` entry point and platform-specific code may be excluded via `#[cfg(not(tarpaulin_include))]`
- **Enforced via**: `.cargo/config.toml` alias: `[alias] cov = "llvm-cov nextest --html"`

### Mocking Strategy

- **Trait-based injection**: All external I/O goes through traits (`NearRpcClient`, `BackendClient`, `RelayerClient`, `PythClient`)
- **`mockall`**: Auto-generate mock implementations for unit/integration tests
- **`wiremock`**: Mock HTTP servers for backend API and relayer client tests
- **Test fixtures**: JSON files in `tests/fixtures/` with real contract responses captured from testnet

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

**Example command page** (matching supply.md pattern from contracts/docs):
```markdown
# Supply

Manage supply positions in Templar markets.

## Deposit

Supply assets to a market to earn yield.

### Using NEAR Direct Signing

    templar supply deposit <market-id> <amount> \
        --signer <account-id>

### Using Universal Account (any chain)

    templar supply deposit <market-id> <amount>

The CLI will use your active key to sign via the Universal Account relayer.

## Withdraw

Since borrowers use supplied assets, withdrawals go through a queue.

### Create Withdrawal Request

    templar supply withdraw <market-id> <amount>

### Check Withdrawal Status

    templar supply withdraw-status <market-id>

### Execute Next Withdrawal

    templar supply execute-withdrawal <market-id>

This is not permissioned — anyone can advance the queue.
```

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
- Define `CliError` enum: `Config`, `Rpc`, `Http`, `Signing`, `InvalidInput`, `Io`, `Serialization`, `Interrupted`, `Other`
- `impl Display` with user-friendly messages
- `impl From<T>` for common error types
- **Tests**: error display formatting, error conversions, all variants round-trip through Display

### 1.3 Configuration System (`src/config/`)
- Config file at `~/.templar/config.toml` (or `$TEMPLAR_CONFIG`)
- `Profile` struct with all network settings (RPC URLs, contract IDs, chain IDs)
- Built-in `mainnet` and `testnet` profiles with sensible defaults
- Load/save/merge logic (file → env → CLI flags)
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
- **Tests** (written first):
  - Deserialize real JSON responses captured from testnet (stored in `tests/fixtures/`)
  - Serialize/deserialize round-trip for every type
  - Edge cases: zero amounts, max values, empty optional fields

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
- `get_configuration(vault_id)` → `VaultConfiguration`
- `get_total_assets(vault_id)` → `U128`
- `get_last_total_assets(vault_id)` → `U128`
- `get_total_supply(vault_id)` → `U128`
- `get_max_deposit(vault_id)` → `U128`
- `convert_to_shares(vault_id, assets)` → `U128`
- `convert_to_assets(vault_id, shares)` → `U128`
- `preview_deposit(vault_id, assets)` → `U128`
- `preview_mint(vault_id, shares)` → `U128`
- `preview_withdraw(vault_id, assets)` → `U128`
- `preview_redeem(vault_id, shares)` → `U128`
- `get_cap_groups(vault_id)` → `Vec<(CapGroupId, CapGroupRecord)>`
- `get_fees(vault_id)` → `Fees`
- `get_restrictions(vault_id)` → `Option<Restrictions>`

**Function calls:**
- `withdraw(vault_id, amount, receiver)` — withdraw assets
- `redeem(vault_id, shares, receiver)` — redeem shares
- `deposit(token_id, vault_id, amount)` → `ft_transfer_call` with deposit msg
- Governance: `set_curator`, `submit_cap`, `accept_cap`, `reallocate`, `set_supply_queue`, etc.

**Tests**: Same pattern as market — mock RPC, verify deserialization, verify tx construction

### 2.6 Registry Contract Client (`src/near/contract/registry.rs`)

- `list_versions(offset, count)` → `Vec<String>`
- `get_version_code_hash(version_key)` → `Option<Base58CryptoHash>`
- `list_deployments(offset, count)` → `Vec<AccountId>`
- `get_deployment(account_id)` → `Option<Deployment>`

**Tests**: Mock RPC responses, verify deserialization

### 2.7 NEP-141 Token Client (`src/near/contract/token.rs`)

- `ft_balance_of(token_id, account_id)` → `U128`
- `ft_metadata(token_id)` → `FungibleTokenMetadata`
- `ft_transfer_call(token_id, receiver_id, amount, msg)` → `Promise`
- `storage_deposit(token_id, account_id)` — ensure storage registered

**Tests**: Standard NEP-141 responses, storage deposit flow

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
templar account balance <token-id> <account-id>         # Token balance

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
- Map key type → signing method → relayer version
- Auto-detect from active key type, or explicit `--auth-method` flag
- **Tests**: dispatch logic for each key type

### 3.3 Chain-Specific Signing (`src/auth/{near_wallet,solana,evm,stellar}.rs`)
- NEAR: Standard ed25519 signing (direct transaction)
- Solana: Ed25519Raw signature → V0 relayer payload
- EVM: EIP-191 personal_sign → V1 relayer payload
- Stellar: Sep53 ed25519 signature → V1 relayer payload
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

## Phase 4: Transaction Signing & Relay (Universal Account Write Operations)

**Goal**: Sign and relay transactions through the Universal Account relayer, enabling all write operations from any supported chain.

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

### 4.5 CLI Commands (Phase 4)

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
```

### 4.6 Transaction Confirmation
- Show transaction summary before signing (method, contract, amounts, gas, auth method)
- `--yes` / `-y` flag to skip confirmation
- Display tx hash on success
- `templar tx status <tx-hash>` — query result

### 4.7 Documentation (Phase 4)
- mdbook `commands/ua.md` and `commands/tx.md`
- Update all write command pages with UA usage examples
- Rustdoc for all `signing/` modules

---

## Phase 5: Advanced Features

**Goal**: Power-user features, governance, cross-chain deposit tracking, and operational tooling.

### 5.1 Vault Governance Commands
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

### 5.2 Cross-Chain Deposit Tracking
- Integration with Intents bridge API
- `templar deposit track <chain> <tx-hash>` — track cross-chain deposit status
- `templar deposit address <chain>` — get deposit address for supported chain

### 5.3 Monitoring Integration
- `templar monitor balance <account-id>` — check funder account balance
- `templar monitor screen <account-id> <chain>` — TRM screening (if API available)

### 5.4 Batch Operations
- `templar batch <file.json>` — execute multiple operations from JSON file
- Dry-run mode: `templar batch --dry-run <file.json>`
- Useful for automated workflows and CI/CD

### 5.5 Shell Completions
- `templar completions <bash|zsh|fish|powershell>` — generate shell completions
- Auto-complete contract IDs, account IDs from config

### 5.6 Documentation (Phase 5)
- mdbook pages for governance, monitoring, batch, completions
- Architecture page for contributors
- Complete glossary

---

## Phase Summary & Dependencies

| Phase | Deliverables | Dependencies | Test Focus |
|-------|-------------|-------------|------------|
| **1: Foundation** | Project scaffold, config, display, vendored types, test infra | None | Config parsing, type serde, display formatting |
| **2: NEAR Contracts** | All Templar contract wrappers, NEAR signing, backend/Pyth clients | Phase 1 + NEAR crates | Mock RPC, contract call construction, CLI E2E |
| **3: Multichain Auth** | Key import/encrypt for 4 chains, auth dispatch, UA lookup | Phase 2 + crypto crates | Key round-trips, signing verification, dispatch |
| **4: UA Relay** | Sign-and-relay flow, PoW, all write ops via UA | Phase 3 + relayer API | Full relay flow, envelope construction, PoW |
| **5: Advanced** | Governance, monitoring, batch, completions, cross-chain | Phase 4 + bridge API | Governance flows, batch parsing, E2E |

---

## Estimated Scope

| Phase | Source Files | Lines (approx) | Test Files | Test Lines (approx) |
|-------|-------------|----------------|------------|---------------------|
| 1: Foundation | ~12 | ~1,800 | ~5 | ~1,200 |
| 2: NEAR Contracts | ~15 | ~3,500 | ~8 | ~3,000 |
| 3: Multichain Auth | ~8 | ~1,500 | ~3 | ~1,200 |
| 4: UA Relay | ~6 | ~1,800 | ~3 | ~1,500 |
| 5: Advanced | ~6 | ~1,400 | ~3 | ~1,000 |

**Total**: ~47 source files, ~10,000 lines of implementation + ~22 test files, ~7,900 lines of tests + mdbook guide (~20 pages) + comprehensive rustdoc

**Coverage target**: 95%+ enforced in CI via `cargo-llvm-cov`
