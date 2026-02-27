#!/usr/bin/env bash
# 04_account.sh — Inspect account positions, balances, and health.
#
# All commands here are read-only — no signer required.
set -euo pipefail

MARKET_ID="market.v1.tmplr.near"   # Replace with a real market account ID
ACCOUNT_ID="alice.near"             # Replace with the account to inspect
TOKEN_ID="usdc.tether-token.near"   # Replace with a NEP-141 token contract

# --- All positions at once ----------------------------------------------------
# Shows every supply and borrow position for an account across all markets.
templar account positions "$ACCOUNT_ID"

# --- Supply position details --------------------------------------------------
# Shows the supply position in a specific market: shares, value, pending yield.
templar account supply "$MARKET_ID" "$ACCOUNT_ID"

# --- Borrow position details --------------------------------------------------
# Shows the borrow position: principal, accrued interest, collateral.
templar account borrow "$MARKET_ID" "$ACCOUNT_ID"

# --- Health / MCR status ------------------------------------------------------
# Checks whether a borrow position is healthy relative to the Minimum
# Collateralization Ratio.
templar account health "$MARKET_ID" "$ACCOUNT_ID"

# --- Pending yield and interest -----------------------------------------------
# View accrued but unclaimed yield (for suppliers) or interest (for borrowers).
templar account pending-yield "$MARKET_ID" "$ACCOUNT_ID"
templar account pending-interest "$MARKET_ID" "$ACCOUNT_ID"

# --- Withdrawal queue status --------------------------------------------------
# If a withdrawal is pending, shows the queue position and estimated wait.
templar account withdrawal-status "$MARKET_ID" "$ACCOUNT_ID"

# --- Token balances -----------------------------------------------------------
# NEP-141 fungible token balance:
templar account balance "$TOKEN_ID" "$ACCOUNT_ID"

# NEP-245 multi-token balance (e.g., market receipt tokens):
# templar account mt-balance <contract_id> <token_id> "$ACCOUNT_ID"
