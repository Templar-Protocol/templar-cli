#!/usr/bin/env bash
# 05_supply.sh — Supply-side operations: deposit, withdraw, harvest.
#
# These are WRITE operations — they require a configured signer key.
# The --signer flag specifies which NEAR account signs the transaction.
set -euo pipefail

MARKET_ID="market.v1.tmplr.near"   # Replace with a real market account ID
SIGNER="alice.near"                 # Replace with your NEAR account ID

# --- Deposit into a market ----------------------------------------------------
# Deposits the borrow asset (e.g., USDC) into the market as a supplier.
# The amount is in base token units.
templar supply deposit "$MARKET_ID" "1000000" --signer "$SIGNER"

# --- Check your supply position -----------------------------------------------
templar account supply "$MARKET_ID" "$SIGNER"

# --- Harvest accumulated yield ------------------------------------------------
# Claims yield that has accrued since your last harvest.
templar supply harvest-yield "$MARKET_ID" --signer "$SIGNER"

# --- Claim static yield -------------------------------------------------------
templar supply claim-static-yield "$MARKET_ID" --signer "$SIGNER"

# --- Request a withdrawal -----------------------------------------------------
# Enters the withdrawal queue. Execution depends on available liquidity.
templar supply withdraw "$MARKET_ID" "500000" --signer "$SIGNER"

# --- Check withdrawal queue status --------------------------------------------
templar account withdrawal-status "$MARKET_ID" "$SIGNER"

# --- Execute a pending withdrawal ---------------------------------------------
# Once your withdrawal reaches the front of the queue and liquidity is
# available, execute it to receive tokens.
templar supply execute-withdraw "$MARKET_ID" --signer "$SIGNER"

# --- Cancel a pending withdrawal ----------------------------------------------
# If you change your mind, cancel before execution.
# templar supply cancel-withdraw "$MARKET_ID" --signer "$SIGNER"
