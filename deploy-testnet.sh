#!/usr/bin/env bash
# deploy-testnet.sh — build and deploy stellar-invoice-protocol to Stellar testnet
#
# Usage:
#   cp deploy-testnet.env.example deploy-testnet.env
#   # edit deploy-testnet.env with your account details
#   source deploy-testnet.env && ./deploy-testnet.sh
#
# The deployed contract ID is written to .last-deploy-testnet after a
# successful deployment.
#
# IMPORTANT: Never commit deploy-testnet.env — it contains your secret key.
# It is already listed in .gitignore.

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration (can be overridden by environment variables)
# ---------------------------------------------------------------------------
STELLAR_SOURCE="${STELLAR_SOURCE:?ERROR: STELLAR_SOURCE is not set. See deploy-testnet.env.example}"
NETWORK="${NETWORK:-testnet}"
WASM_PATH="target/wasm32-unknown-unknown/release/stellar_invoice_protocol.wasm"
DEPLOY_RECORD=".last-deploy-testnet"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
echo "[deploy] Building release WASM..."
cargo build --target wasm32-unknown-unknown --release --locked

if [[ ! -f "$WASM_PATH" ]]; then
    echo "[deploy] ERROR: WASM artifact not found at $WASM_PATH" >&2
    exit 1
fi

echo "[deploy] Build complete: $WASM_PATH"

# ---------------------------------------------------------------------------
# Deploy
# ---------------------------------------------------------------------------
echo "[deploy] Deploying to $NETWORK using source: $STELLAR_SOURCE"

CONTRACT_ID=$(stellar contract deploy \
    --wasm "$WASM_PATH" \
    --source "$STELLAR_SOURCE" \
    --network "$NETWORK")

if [[ -z "$CONTRACT_ID" ]]; then
    echo "[deploy] ERROR: stellar contract deploy returned an empty contract ID." >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Record
# ---------------------------------------------------------------------------
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
cat > "$DEPLOY_RECORD" <<EOF
NETWORK=$NETWORK
CONTRACT_ID=$CONTRACT_ID
DEPLOYED_AT=$TIMESTAMP
SOURCE=$STELLAR_SOURCE
EOF

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Deployed to   : $NETWORK"
echo "  Contract ID   : $CONTRACT_ID"
echo "  Timestamp     : $TIMESTAMP"
echo "  Recorded in   : $DEPLOY_RECORD"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo "  1. Verify the build matches on-chain:"
echo "     ./verify-build.sh $CONTRACT_ID"
echo "  2. Test an invocation:"
echo "     stellar contract invoke \\"
echo "       --id $CONTRACT_ID \\"
echo "       --source \$STELLAR_SOURCE \\"
echo "       --network $NETWORK \\"
echo "       -- get_invoice --invoice_id 1"
echo ""
echo "Pass the contract ID and a transaction hash to the repo maintainer"
echo "to update the live deployment section in README.md."
