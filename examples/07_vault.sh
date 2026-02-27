#!/usr/bin/env bash
# 07_vault.sh — Vault operations (ERC-4626-style).
#
# Vaults pool assets and deploy them into lending markets automatically.
# Depositors receive shares representing their proportional stake.
set -euo pipefail

VAULT_ID="vault.v1.tmplr.near"   # Replace with a real vault account ID
SIGNER="alice.near"               # Replace with your NEAR account ID

# --- View vault info ----------------------------------------------------------
# Shows vault configuration: underlying asset, total assets, total shares,
# and share price.
templar vault info "$VAULT_ID"

# --- Preview operations -------------------------------------------------------
# Estimate how many shares you would receive for a deposit, or how many
# assets you would receive for a redemption — without executing.
templar vault preview-deposit "$VAULT_ID" "1000000"
templar vault preview-redeem "$VAULT_ID" "500"

# --- Deposit into vault -------------------------------------------------------
# Deposits assets and receives vault shares in return.
templar vault deposit "$VAULT_ID" "1000000" --signer "$SIGNER"

# --- Withdraw from vault ------------------------------------------------------
# Withdraws a specific amount of the underlying asset.
templar vault withdraw "$VAULT_ID" "500000" --signer "$SIGNER"

# --- Redeem shares ------------------------------------------------------------
# Burns a specific number of shares and receives the underlying asset.
templar vault redeem "$VAULT_ID" "250" --signer "$SIGNER"
