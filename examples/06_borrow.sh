#!/usr/bin/env bash
# 06_borrow.sh — Borrow-side operations: collateralize, take, repay.
#
# These are WRITE operations — they require a configured signer key.
#
# Typical borrow flow:
#   1. Deposit collateral  (collateralize)
#   2. Take a borrow       (take)
#   3. Monitor health       (account health)
#   4. Repay the borrow     (repay)
#   5. Withdraw collateral  (withdraw-collateral)
set -euo pipefail

MARKET_ID="market.v1.tmplr.near"   # Replace with a real market account ID
SIGNER="alice.near"                 # Replace with your NEAR account ID

# --- Step 1: Deposit collateral -----------------------------------------------
# Transfers collateral tokens into the market contract.
templar borrow collateralize "$MARKET_ID" "5000000000000000000000000" --signer "$SIGNER"
#                                          ^ 5 NEAR in yoctoNEAR

# --- Step 2: Take a borrow ---------------------------------------------------
# Borrows the borrow asset (e.g., USDC) against your collateral.
templar borrow take "$MARKET_ID" "1000000" --signer "$SIGNER"

# --- Monitor borrow health ----------------------------------------------------
# Check that your position stays above the Minimum Collateralization Ratio.
templar account health "$MARKET_ID" "$SIGNER"
templar account borrow "$MARKET_ID" "$SIGNER"

# --- View accrued interest ----------------------------------------------------
templar account pending-interest "$MARKET_ID" "$SIGNER"

# --- Step 3: Apply interest ---------------------------------------------------
# Materializes pending interest into the borrow balance. Can be called by
# anyone (permissionless), but typically done by the borrower or a keeper.
templar borrow apply-interest "$MARKET_ID" --signer "$SIGNER"

# Apply interest for a different account (keeper pattern):
# templar borrow apply-interest "$MARKET_ID" --signer "$SIGNER" --account bob.near

# --- Step 4: Repay the borrow ------------------------------------------------
# Repays part or all of the borrow. Amount is in borrow asset base units.
templar borrow repay "$MARKET_ID" "1000000" --signer "$SIGNER"

# --- Step 5: Withdraw collateral ----------------------------------------------
# After repaying, withdraw your collateral.
templar borrow withdraw-collateral "$MARKET_ID" "5000000000000000000000000" --signer "$SIGNER"
