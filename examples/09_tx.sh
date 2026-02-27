#!/usr/bin/env bash
# 09_tx.sh — Inspect transaction status on NEAR.
#
# After submitting a transaction (supply, borrow, vault, etc.), use the
# tx hash to check its execution outcome.
set -euo pipefail

TX_HASH="AbC123..."   # Replace with a real NEAR transaction hash
SIGNER="alice.near"    # The account that signed the transaction

# --- Check transaction status -------------------------------------------------
# Returns the full execution outcome: success/failure, gas used, logs,
# and receipt outcomes.
templar tx status "$TX_HASH" --signer "$SIGNER"

# --- JSON output for parsing --------------------------------------------------
templar tx status "$TX_HASH" --signer "$SIGNER" --output json
