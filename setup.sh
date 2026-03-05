#!/usr/bin/env bash
# setup.sh — Build templar-cli and configure everything from scratch.
#
# Run from the repo root:
#   ./setup.sh
#
# What this does:
#   1. Checks that Rust is installed
#   2. Builds the templar binary (release mode)
#   3. Adds the binary to your PATH for this session
#   4. Initializes the Templar config (~/.templar/config.toml)
#   5. Installs near-cli-rs (if not already installed)
#   6. Walks you through importing your NEAR account credentials
#   7. Verifies backend connectivity
set -euo pipefail

REPO_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY="$REPO_DIR/target/release/templar"

# --- Colors (disabled if NO_COLOR is set) ------------------------------------
if [ -z "${NO_COLOR:-}" ]; then
    GOLD='\033[38;2;213;170;81m'
    IVORY='\033[38;2;232;225;211m'
    GREY='\033[38;2;165;155;137m'
    RED='\033[38;2;224;53;53m'
    GREEN='\033[38;2;80;199;89m'
    RESET='\033[0m'
else
    GOLD='' IVORY='' GREY='' RED='' GREEN='' RESET=''
fi

info()  { echo -e "${GOLD}✠${RESET} ${IVORY}$*${RESET}"; }
ok()    { echo -e "${GREEN}✓${RESET} $*"; }
warn()  { echo -e "${RED}✗${RESET} $*"; }
dim()   { echo -e "${GREY}  $*${RESET}"; }

# =============================================================================
# 1. Check Rust toolchain
# =============================================================================
info "Checking Rust toolchain..."
if ! command -v cargo &>/dev/null; then
    warn "Rust not found. Install it from https://rustup.rs/"
    echo
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo
    exit 1
fi
ok "Rust $(rustc --version | awk '{print $2}')"

# =============================================================================
# 2. Build templar
# =============================================================================
info "Building templar (release mode)..."
cargo build --release --manifest-path "$REPO_DIR/Cargo.toml" --quiet
ok "Built: $BINARY"

# =============================================================================
# 3. Add to PATH for this session
# =============================================================================
export PATH="$REPO_DIR/target/release:$PATH"
dim "Added target/release to PATH for this session."
dim "To make it permanent, add this to your shell profile:"
dim "  export PATH=\"$REPO_DIR/target/release:\$PATH\""
dim "Or run: cargo install --path $REPO_DIR"
echo

# =============================================================================
# 4. Initialize Templar config
# =============================================================================
info "Initializing Templar configuration..."
templar config init
ok "Config ready at ~/.templar/config.toml"
echo

# =============================================================================
# 5. Install near-cli-rs
# =============================================================================
info "Checking for near-cli-rs..."
if command -v near &>/dev/null; then
    ok "near-cli-rs already installed: $(near --version 2>/dev/null || echo 'unknown version')"
else
    echo
    echo "  near-cli-rs is needed to import your NEAR account credentials."
    echo "  Install it with one of:"
    echo
    echo "    cargo install near-cli-rs"
    echo "    npm install -g near-cli-rs@latest"
    echo "    curl --proto '=https' --tlsv1.2 -LsSf https://github.com/near/near-cli-rs/releases/latest/download/near-cli-rs-installer.sh | sh"
    echo
    read -rp "  Install via cargo now? [Y/n] " answer
    if [[ "${answer:-Y}" =~ ^[Yy]?$ ]]; then
        cargo install near-cli-rs
        ok "near-cli-rs installed"
    else
        dim "Skipping near-cli-rs install. You can install it later."
    fi
fi
echo

# =============================================================================
# 6. Import NEAR account credentials
# =============================================================================
info "NEAR account setup"
echo
echo "  Write operations (supply, borrow, repay, etc.) require a NEAR signing key."
echo "  Keys are stored at ~/.near-credentials/{network}/{account_id}.json"
echo
echo "  Options:"
echo "    1) Import an existing account via web wallet (opens browser)"
echo "    2) Create a new testnet account (funded by faucet)"
echo "    3) Skip — I'll set up credentials later"
echo

read -rp "  Choose [1/2/3]: " choice
case "${choice:-3}" in
    1)
        read -rp "  Network (mainnet/testnet) [testnet]: " network
        network="${network:-testnet}"
        if command -v near &>/dev/null; then
            near account import-account using-web-wallet network-config "$network"
            ok "Account imported for $network"
        else
            warn "near-cli-rs not found. Install it first, then run:"
            echo "  near account import-account using-web-wallet network-config $network"
        fi
        ;;
    2)
        read -rp "  Account name (e.g. your-name.testnet): " account_name
        if [ -z "$account_name" ]; then
            warn "No account name provided, skipping."
        elif command -v near &>/dev/null; then
            near account create-account sponsor-by-faucet-service "$account_name" \
                autogenerate-new-keypair save-to-keychain network-config testnet create
            ok "Testnet account $account_name created"
        else
            warn "near-cli-rs not found. Install it first, then run:"
            echo "  near account create-account sponsor-by-faucet-service $account_name \\"
            echo "      autogenerate-new-keypair save-to-keychain network-config testnet create"
        fi
        ;;
    *)
        dim "Skipped. You can set up credentials later — see README.md for details."
        ;;
esac
echo

# =============================================================================
# 7. Verify connectivity
# =============================================================================
info "Checking backend connectivity..."
templar health || true
echo

# =============================================================================
# Done
# =============================================================================
echo -e "${GOLD}✠  Setup complete.${RESET}"
echo
dim "Try it out:"
dim "  templar markets list"
dim "  templar prices btc eth sol"
dim "  templar --help"
echo
