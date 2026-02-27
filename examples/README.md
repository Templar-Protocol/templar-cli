# Templar CLI Examples

Practical examples for the most common Templar CLI workflows. Each script
is a standalone Bash file that can be run directly or read as reference.

## Quick Start

```bash
# 1. Build the CLI
cargo build --release
export PATH="$PWD/target/release:$PATH"

# 2. Initialize config
templar config init

# 3. Check connectivity
templar health
```

## Examples

| File | Description |
|------|-------------|
| [01_setup.sh](01_setup.sh) | First-time configuration and health check |
| [02_markets.sh](02_markets.sh) | Browse markets, view metrics and snapshots |
| [03_prices.sh](03_prices.sh) | Query oracle prices for assets |
| [04_account.sh](04_account.sh) | Inspect account positions, balances, and health |
| [05_supply.sh](05_supply.sh) | Supply-side operations: deposit, withdraw, harvest |
| [06_borrow.sh](06_borrow.sh) | Borrow-side operations: collateralize, take, repay |
| [07_vault.sh](07_vault.sh) | Vault operations: deposit, withdraw, redeem |
| [08_registry.sh](08_registry.sh) | Query the deployment registry |
| [09_tx.sh](09_tx.sh) | Inspect transaction status |
| [10_json_output.sh](10_json_output.sh) | Machine-readable JSON output for scripting |

## Conventions

- All scripts use `set -euo pipefail` for safety.
- Lines starting with `#>` show expected output.
- Replace placeholder values (`<MARKET_ID>`, `<ACCOUNT_ID>`, etc.) with
  real values from your network.
- Write operations require `--signer` and a configured key.
- Add `--output json` to any command for structured output.
- Add `--profile testnet` to run against testnet.
