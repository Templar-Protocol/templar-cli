# Templar CLI

[![CI](https://github.com/Templar-Protocol/templar-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/Templar-Protocol/templar-cli/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Templar-Protocol/templar-cli/graph/badge.svg)](https://codecov.io/gh/Templar-Protocol/templar-cli)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

Command-line interface for [Templar Protocol](https://templarfi.org) — the first cypher lending protocol on NEAR. Borrow against native BTC, ZEC, XRP, ADA, DOGE, XLM, LTC, and more without trusting centralized intermediaries.

Your keys. Your protocol. Your bank.

## Features

- **Lending markets** — browse markets, view metrics, check utilization and rates
- **Supply & borrow** — deposit, withdraw, collateralize, repay, harvest yield
- **Vaults** — deposit into and manage vault positions
- **Oracle prices** — real-time Pyth price feeds for all supported assets
- **Account positions** — inspect balances, health factors, and liquidation risk
- **Cross-chain bridging** — deposit and withdraw assets via NEAR Intents and Hot Bridge
- **Universal Accounts** — manage Templar's cross-chain account abstraction
- **Transaction inspection** — look up status and history of on-chain transactions
- **Themed output** — gold/ivory cypherpunk terminal aesthetic with ASCII art, spinners, and box-drawing frames
- **JSON output** — machine-readable `--output json` on every command for scripting and pipelines

## Requirements

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- A NEAR account with signing keys for write operations

## Getting Started

The setup script handles everything — building the binary, initializing config, installing [near-cli-rs](https://github.com/near/near-cli-rs), and importing your NEAR account credentials:

```bash
git clone https://github.com/Templar-Protocol/templar-cli.git
cd templar-cli
./setup.sh
```

The script is interactive and will walk you through each step. Once it finishes:

```bash
templar markets list
templar prices btc eth sol
templar account positions your-account.near
```

### Manual Setup

If you prefer to set things up yourself:

```bash
# Build
cargo build --release
export PATH="$PWD/target/release:$PATH"

# Initialize config
templar config init

# Install near-cli-rs and import your NEAR account
cargo install near-cli-rs
near account import-account using-web-wallet network-config testnet

# Verify
templar health
```

Write operations (supply, borrow, repay, etc.) require a NEAR signing key. The `near account import-account` command above stores your key at `~/.near-credentials/{network}/{account_id}.json` — the standard format used by near-cli-rs. See [`examples/01_setup.sh`](examples/01_setup.sh) for more credential setup options.

## Configuration

Templar CLI stores its configuration at `~/.templar/config.toml` (override with `$TEMPLAR_CONFIG`). The config supports multiple profiles for different networks:

```toml
active_profile = "mainnet"

[profiles.mainnet]
near_rpc_url = "https://rpc.mainnet.fastnear.com"
backend_url = "https://api.templarfi.org"
near_network_id = "mainnet"
# ... other endpoints

[profiles.testnet]
near_rpc_url = "https://rpc.testnet.near.org"
backend_url = "https://api-testnet.templarfi.org"
near_network_id = "testnet"

[theme]
banner = true
color = "auto"         # auto | always | never
animations = true
unicode = true
voice = "cypherpunk"   # cypherpunk | standard
```

Switch profiles per-command with `--profile testnet`, or set the default:

```bash
templar config set active_profile testnet
```

## Usage

### Global Flags

| Flag | Description |
|------|-------------|
| `--profile <NAME>` | Configuration profile to use |
| `--network <NETWORK>` | Override NEAR network ID |
| `--rpc-url <URL>` | Override NEAR RPC endpoint |
| `--output <FORMAT>` | Output format: `table` (default) or `json` |
| `--color <MODE>` | Color mode: `auto`, `always`, or `never` |
| `--quiet` / `-q` | Suppress banner and non-essential output |
| `--no-banner` | Suppress startup banner only |
| `--no-animation` | Disable spinners and text effects |

### Commands

| Command | Description |
|---------|-------------|
| `templar config` | Initialize, show, or modify configuration |
| `templar markets` | List markets, view details and metrics |
| `templar account` | Inspect account positions, balances, and health |
| `templar supply` | Supply-side operations: deposit, withdraw, harvest |
| `templar borrow` | Borrow-side operations: collateralize, take, repay |
| `templar vault` | Vault operations: deposit, withdraw, redeem |
| `templar registry` | Query the deployment registry |
| `templar prices` | Query oracle prices for assets |
| `templar ua` | Universal Account operations |
| `templar bridge` | Cross-chain bridge operations |
| `templar tx` | Transaction status and history |
| `templar health` | Check backend connectivity |

Run `templar <command> --help` for subcommands and details.

## Documentation

- **User guide** (mdbook): build with `mdbook build docs/` then open `docs/book/index.html`, or browse `docs/src/` directly
- **API docs** (rustdoc): generate with `cargo doc --open`
- **Examples**: see the [`examples/`](examples/) directory for annotated workflow scripts covering setup, markets, supply, borrow, vaults, and more

## Development

### Prerequisites

```bash
cargo install cargo-nextest cargo-llvm-cov mdbook just
```

### Common Tasks

```bash
just test          # Run tests with nextest
just lint          # Run clippy lints
just fmt           # Format code
just cov           # Test coverage report (HTML)
just doc           # Generate rustdoc
just book          # Build mdbook user guide
just ci            # Full CI check: fmt + lint + test + doctest + doc + book
```

### Project Structure

```
src/
  commands/       # CLI command definitions (clap derive)
  config/         # Configuration profiles and file management
  client/         # Backend API and HTTP clients
  display/        # Themed terminal output (palette, frames, tables, spinners)
  near/           # NEAR RPC client, transaction builder, signer
  types/          # Vendored domain types (market, vault, oracle, bridge)
  error.rs        # Unified error type with exit codes
  lib.rs          # Library root
  main.rs         # Binary entrypoint
docs/             # mdbook user guide source
examples/         # Annotated workflow scripts
```

## Supported Assets

Templar supports lending against native assets from multiple chains, bridged via NEAR Intents:

BTC, ETH, SOL, XRP, ADA, LTC, ZEC, DOGE, XLM, and ERC-20 tokens.

All cross-chain assets are represented as NEP-245 multi-tokens within the `intents.near` verifier contract on NEAR.
