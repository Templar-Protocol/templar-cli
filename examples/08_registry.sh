#!/usr/bin/env bash
# 08_registry.sh — Query the Templar deployment registry.
#
# The registry tracks all deployed Templar contracts: markets, vaults,
# and their versions. All commands here are read-only.
set -euo pipefail

ACCOUNT_ID="alice.near"   # Replace with a real account ID

# --- List registered deployments ----------------------------------------------
# Shows all contracts registered in the protocol, with pagination.
templar registry list
templar registry list --count 10 --offset 0

# --- Show deployment for a specific account -----------------------------------
# Looks up a contract's deployment metadata: version, code hash, and kind.
templar registry show "$ACCOUNT_ID"

# --- List available versions --------------------------------------------------
# Shows all published contract versions in the registry.
templar registry versions
templar registry versions --count 5
