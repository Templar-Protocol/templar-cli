#!/usr/bin/env bash
# 03_prices.sh — Query oracle prices via Pyth/Hermes.
#
# Fetches real-time prices for one or more assets. Prices come from the
# Pyth Network Hermes API and include confidence intervals.
set -euo pipefail

# --- Single asset price -------------------------------------------------------
templar prices NEAR
#> Prices:
#>   NEAR: $4.12 (conf: 0.02, expo: -8)

# --- Multiple assets at once --------------------------------------------------
templar prices NEAR USDC ETH BTC

# --- JSON output for scripting ------------------------------------------------
templar prices NEAR --output json
#> [{ "id": "...", "price": "412000000", "conf": "2000000", "expo": -8 }]
