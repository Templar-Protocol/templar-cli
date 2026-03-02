#!/usr/bin/env bash
# 10_json_output.sh — Machine-readable JSON output for scripting and AI agents.
#
# Every templar command supports --output json for structured output.
# This makes it easy to pipe into jq, parse from scripts, or feed to
# AI agents that need structured data.
set -euo pipefail

MARKET_ID="market.v1.tmplr.near"   # Replace with a real market account ID
ACCOUNT_ID="alice.near"             # Replace with a real account ID

# --- JSON flag works on all commands ------------------------------------------
templar health --output json
templar markets list --output json
templar markets show "$MARKET_ID" --output json
templar account positions "$ACCOUNT_ID" --output json
templar prices NEAR USDC --output json

# --- Pipe into jq for field extraction ----------------------------------------
# Get just the market names:
# templar markets list --output json | jq '.[].name'

# Get the NEAR price as a number (price field is a string, so convert it):
# templar prices NEAR --output json | jq '.[0].price | tonumber'

# Check if a borrow position is healthy:
# templar account health "$MARKET_ID" "$ACCOUNT_ID" --output json | jq '.healthy'

# --- Combine with other tools -------------------------------------------------
# Monitor a position in a loop:
# while true; do
#   templar account health "$MARKET_ID" "$ACCOUNT_ID" --output json -q
#   sleep 60
# done

# --- Quiet mode suppresses banner and animations -----------------------------
# Use -q (--quiet) with --output json for clean machine-readable output:
templar markets list --output json -q
