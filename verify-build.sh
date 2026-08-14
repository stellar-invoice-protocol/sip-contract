#!/usr/bin/env bash
# verify-build.sh — reproducible build verification for stellar-invoice-protocol
#
# Usage:
#   ./verify-build.sh                        # build only, print local WASM hash
#   ./verify-build.sh <CONTRACT_ID>          # build + compare against on-chain hash
#
# The script pins the Rust toolchain from rust-toolchain.toml when that file
# exists; otherwise it uses the explicitly pinned TOOLCHAIN variable below.
# Rebuild is always done from clean state (cargo clean) to guarantee the hash
# reflects only the committed source tree.
#
# Requirements:
#   - rustup
#   - cargo
#   - sha256sum (or shasum on macOS)
#   - stellar CLI (optional — only needed for on-chain comparison)

set -euo pipefail

# ---------------------------------------------------------------------------
# Toolchain pinning
# ---------------------------------------------------------------------------
TOOLCHAIN_FILE="rust-toolchain.toml"
EXPLICIT_TOOLCHAIN="stable"   # change to e.g. "1.78.0" once you pin a version

if [[ -f "$TOOLCHAIN_FILE" ]]; then
    # Let rustup pick it up automatically from the file.
    TOOLCHAIN_ARG=""
    echo "[verify-build] Using toolchain from $TOOLCHAIN_FILE"
else
    TOOLCHAIN_ARG="+${EXPLICIT_TOOLCHAIN}"
    echo "[verify-build] No rust-toolchain.toml found — using toolchain: ${EXPLICIT_TOOLCHAIN}"
fi

# ---------------------------------------------------------------------------
# Ensure the wasm32 target is available
# ---------------------------------------------------------------------------
rustup target add wasm32-unknown-unknown ${TOOLCHAIN_ARG:+"--toolchain" "$EXPLICIT_TOOLCHAIN"} 2>/dev/null || true

# ---------------------------------------------------------------------------
# Clean previous artifacts and rebuild deterministically
# ---------------------------------------------------------------------------
echo "[verify-build] Running cargo clean..."
cargo ${TOOLCHAIN_ARG} clean

echo "[verify-build] Building release WASM (--locked)..."
cargo ${TOOLCHAIN_ARG} build \
    --target wasm32-unknown-unknown \
    --release \
    --locked

WASM_PATH="target/wasm32-unknown-unknown/release/stellar_invoice_protocol.wasm"

if [[ ! -f "$WASM_PATH" ]]; then
    echo "[verify-build] ERROR: WASM artifact not found at $WASM_PATH" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Compute local SHA-256
# ---------------------------------------------------------------------------
if command -v sha256sum &>/dev/null; then
    LOCAL_HASH=$(sha256sum "$WASM_PATH" | awk '{print $1}')
elif command -v shasum &>/dev/null; then
    LOCAL_HASH=$(shasum -a 256 "$WASM_PATH" | awk '{print $1}')
else
    echo "[verify-build] ERROR: sha256sum / shasum not found" >&2
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  WASM artifact : $WASM_PATH"
echo "  Local SHA-256 : $LOCAL_HASH"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ---------------------------------------------------------------------------
# Optional: compare against on-chain hash
# ---------------------------------------------------------------------------
CONTRACT_ID="${1:-}"

if [[ -z "$CONTRACT_ID" ]]; then
    echo ""
    echo "Tip: pass a CONTRACT_ID as the first argument to compare against the"
    echo "on-chain WASM hash:"
    echo "  ./verify-build.sh <CONTRACT_ID>"
    exit 0
fi

if ! command -v stellar &>/dev/null; then
    echo "" >&2
    echo "[verify-build] ERROR: stellar CLI not found — cannot fetch on-chain hash." >&2
    echo "Install it with: cargo install --locked stellar-cli --features opt" >&2
    exit 1
fi

NETWORK="${NETWORK:-testnet}"
echo ""
echo "[verify-build] Fetching on-chain WASM for contract $CONTRACT_ID on $NETWORK..."

ONCHAIN_WASM_TMP=$(mktemp /tmp/sip-onchain-XXXXXX.wasm)
trap 'rm -f "$ONCHAIN_WASM_TMP"' EXIT

stellar contract fetch \
    --id "$CONTRACT_ID" \
    --network "$NETWORK" \
    --out-file "$ONCHAIN_WASM_TMP"

if command -v sha256sum &>/dev/null; then
    ONCHAIN_HASH=$(sha256sum "$ONCHAIN_WASM_TMP" | awk '{print $1}')
else
    ONCHAIN_HASH=$(shasum -a 256 "$ONCHAIN_WASM_TMP" | awk '{print $1}')
fi

echo "  On-chain SHA-256 : $ONCHAIN_HASH"
echo ""

if [[ "$LOCAL_HASH" == "$ONCHAIN_HASH" ]]; then
    echo "  ✅  MATCH — local build matches on-chain deployment"
    exit 0
else
    echo "  ❌  MISMATCH — local build does NOT match on-chain deployment"
    echo ""
    echo "  Local   : $LOCAL_HASH"
    echo "  On-chain: $ONCHAIN_HASH"
    exit 1
fi
