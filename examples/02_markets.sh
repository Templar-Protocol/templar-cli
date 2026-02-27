#!/usr/bin/env bash
# 02_markets.sh — Browse lending markets and view their state.
#
# All commands here are read-only — no signer required.
set -euo pipefail

MARKET_ID="market.v1.tmplr.near"  # Replace with a real market account ID

# --- List all markets ---------------------------------------------------------
# Returns every market registered in the protocol.
templar markets list

# --- Show market details ------------------------------------------------------
# Displays the full configuration for a single market: assets, oracles,
# parameters, and current state.
templar markets show "$MARKET_ID"

# --- View current snapshot ----------------------------------------------------
# A snapshot captures the market's real-time state: total supply, total
# borrows, utilization, and rates.
templar markets snapshot "$MARKET_ID"

# --- View historical snapshots ------------------------------------------------
# Returns the last N snapshots (default 10). Useful for tracking rate changes.
templar markets snapshots "$MARKET_ID" --count 5

# --- View market metrics ------------------------------------------------------
# Focused view of deposited and borrowed amounts for the borrow asset.
templar markets metrics "$MARKET_ID"

# --- View current yield rate --------------------------------------------------
# Shows the annualized yield rate for suppliers.
templar markets yield-rate "$MARKET_ID"
