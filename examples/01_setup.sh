#!/usr/bin/env bash
# 01_setup.sh — First-time configuration and connectivity check.
#
# Creates the Templar config file and verifies the backend is reachable.
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

# --- Check backend health ----------------------------------------------------
# Verifies the Templar backend API is reachable and returns status.
templar health
#> { "status": "ok", ... }

# --- Switch profiles ----------------------------------------------------------
# Use --profile to target a different network:
templar health --profile testnet
