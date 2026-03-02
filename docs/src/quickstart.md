# Quick Start

## 1. Initialize Configuration

```bash
templar config init
```

## 2. Explore Markets

```bash
# List all available markets
templar markets list

# View details for a specific market
templar markets show ibtc-usdc.v1.tmplr.near

# Check current metrics
templar markets metrics ibtc-usdc.v1.tmplr.near
```

## 3. Check Positions

```bash
# View all positions for an account
templar account positions your-account.near

# Check supply position
templar account supply ibtc-usdc.v1.tmplr.near your-account.near

# Check borrow position
templar account borrow ibtc-usdc.v1.tmplr.near your-account.near
```

## 4. Supply to a Market

> **Warning:** This is a state-changing transaction. Use `--profile testnet` or a testnet signer to avoid operating on mainnet accidentally.

```bash
templar supply deposit ibtc-usdc.v1.tmplr.near 1000 --signer your-account.testnet --profile testnet
```

## 5. Check Prices

```bash
templar prices btc eth sol
```

## 6. JSON Output

All commands support `--output json` for scripting:

```bash
templar markets show ibtc-usdc.v1.tmplr.near --output json | jq '.borrow_asset'
```
