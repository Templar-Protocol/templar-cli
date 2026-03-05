# Templar CLI

**The First Cypher Lending Protocol — Be Your Own Bank.**

The Templar CLI is your sovereign terminal interface to the Templar Protocol. Interact with lending markets, vaults, and cross-chain bridge operations across multiple blockchains.

## Features

- **NEAR Contract Operations** — Query markets, manage supply/borrow positions, interact with vaults and the registry
- **Cross-Chain Support** — Bridge assets from Bitcoin, Ethereum, Solana, Stellar, and more via NEAR Intents
- **Universal Account** — Sign transactions with NEAR, Solana, EVM, or Stellar keys
- **Cypherpunk Terminal** — Themed CLI experience with gold-on-dark styling, binary animations, and cryptographic flair
- **Machine-Friendly** — `--output json` for scripting and automation

## Quick Example

```bash
# View market details
templar markets show ibtc-usdc.v1.tmplr.near

# Check your supply position
templar account supply ibtc-usdc.v1.tmplr.near your-account.near

# Supply to a market
templar supply deposit ibtc-usdc.v1.tmplr.near 1000 --signer your-account.near

# Get oracle prices
templar prices btc eth sol
```

## Architecture

The CLI is built in Rust and interacts directly with NEAR Protocol contracts via JSON-RPC. For cross-chain operations, it uses the NEAR Intents bridge system and the Templar relayer infrastructure.

All cross-chain assets (BTC, ETH, SOL, XRP, XLM, ADA, LTC, ZEC, DOGE) are represented as NEP-245 multi-tokens within the `intents.near` verifier contract on NEAR.
