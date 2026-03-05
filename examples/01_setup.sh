#!/usr/bin/env bash
# 01_setup.sh — First-time configuration, NEAR key setup, and connectivity check.
#
# Creates the Templar config file, sets up NEAR signing credentials,
# and verifies the backend is reachable.
set -euo pipefail

# --- Initialize configuration ------------------------------------------------
# Creates ~/.templar/config.toml with default mainnet and testnet profiles.
# Safe to run multiple times — it will not overwrite an existing config.
templar config init

# --- View the active configuration -------------------------------------------
# Shows which profile is active and all resolved settings.
templar config show

# --- Override a setting -------------------------------------------------------
# Point the RPC URL at a different endpoint (useful for local nodes or
# alternative RPC providers).
# templar config set near_rpc_url https://rpc.mainnet.near.org

# =============================================================================
# NEAR Signing Keys
# =============================================================================
#
# Write operations (supply, borrow, repay, etc.) require a NEAR signing key.
# Keys are loaded from ~/.near-credentials/{network}/{account_id}.json — the
# same format used by the official NEAR CLI.
#
# --- Option A: Login with NEAR CLI (recommended) ----------------------------
# This opens a browser to authorize your account and stores the key file
# in ~/.near-credentials/ automatically.
#
#   npm install -g near-cli          # install NEAR CLI if needed
#   near login                       # mainnet
#   near login --networkId testnet   # testnet
#
# --- Option B: Generate a key pair ------------------------------------------
# Creates a new key pair and writes it to ~/.near-credentials/. You then
# need to add the public key as a full-access key to your account.
#
#   near generate-key your-account.testnet --networkId testnet
#
# --- Option C: Manual key file -----------------------------------------------
# Create the key file yourself. Replace the placeholder values with your
# actual keys.
#
#   mkdir -p ~/.near-credentials/testnet
#
#   cat > ~/.near-credentials/testnet/your-account.testnet.json << 'KEYEOF'
#   {
#     "account_id": "your-account.testnet",
#     "public_key": "ed25519:YOUR_PUBLIC_KEY",
#     "private_key": "ed25519:YOUR_PRIVATE_KEY"
#   }
#   KEYEOF
#
#   chmod 600 ~/.near-credentials/testnet/your-account.testnet.json

# --- Verify credentials exist ------------------------------------------------
# Check that a key file is present for your account. Uncomment and replace
# the account ID with your own.
#
# ACCOUNT="your-account.testnet"
# NETWORK="testnet"
# KEY_FILE="$HOME/.near-credentials/$NETWORK/$ACCOUNT.json"
#
# if [ -f "$KEY_FILE" ]; then
#     echo "Key file found: $KEY_FILE"
# else
#     echo "No key file at $KEY_FILE"
#     echo "Run 'near login --networkId $NETWORK' to create one."
#     exit 1
# fi

# --- Check backend health ----------------------------------------------------
# Verifies the Templar backend API is reachable and returns status.
templar health
#> Backend: healthy (https://api.templarfi.org)

# --- Switch profiles ----------------------------------------------------------
# Use --profile to target a different network:
templar health --profile testnet

# --- Test a read-only command -------------------------------------------------
# Read-only commands work without signing keys:
templar markets list
#> Lists all available lending markets

# --- Test a write command (requires signing key) ------------------------------
# Uncomment to test a write operation on testnet:
# templar supply deposit ibtc-usdc.v1.tmplr.testnet 1000 \
#     --signer your-account.testnet --profile testnet
