#!/usr/bin/env bash
# deploy.sh — Deploy quorumforge-contract to Testnet or Mainnet
# Usage:
#   ./scripts/deploy.sh testnet <source-account>
#   ./scripts/deploy.sh mainnet <source-account>
#
# Options:
#   SAVE_ID=1   Write the deployed contract ID to .contract-id.<network>

set -euo pipefail

NETWORK="${1:-testnet}"
SOURCE="${2:-default}"
SAVE_ID="${SAVE_ID:-0}"
WASM="target/wasm32-unknown-unknown/release/quorumforge_contract.wasm"
OPTIMIZED="target/quorumforge_contract.optimized.wasm"

echo "==> Building contract..."
cargo build --target wasm32-unknown-unknown --release

echo "==> Optimizing wasm..."
stellar contract optimize --wasm "$WASM" --wasm-out "$OPTIMIZED"

WASM_SIZE=$(du -k "$OPTIMIZED" | cut -f1)
echo "    Optimized wasm size: ${WASM_SIZE}KB"

if [[ "$NETWORK" == "mainnet" ]]; then
  echo ""
  echo "⚠️  WARNING: Deploying to MAINNET. This costs real XLM."
  echo "   Source account: $SOURCE"
  read -rp "   Type 'yes' to confirm: " CONFIRM
  [[ "$CONFIRM" == "yes" ]] || { echo "Aborted."; exit 1; }
fi

echo "==> Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$OPTIMIZED" \
  --source "$SOURCE" \
  --network "$NETWORK")

if [[ "$SAVE_ID" == "1" ]]; then
  echo "$CONTRACT_ID" > ".contract-id.${NETWORK}"
  echo "    Contract ID saved to .contract-id.${NETWORK}"
fi

echo ""
echo "✅ Deployed successfully!"
echo "   Contract ID : $CONTRACT_ID"
echo "   Network     : $NETWORK"
echo ""
echo "Next — initialize the contract:"
echo ""
echo "  stellar contract invoke \\"
echo "    --id $CONTRACT_ID \\"
echo "    --source $SOURCE \\"
echo "    --network $NETWORK \\"
echo "    -- initialize \\"
echo "    --admin <ADMIN_ADDRESS> \\"
echo "    --members '[\"<MEMBER_1>\",\"<MEMBER_2>\",\"<MEMBER_3>\"]' \\"
echo "    --threshold 2"
echo ""
echo "Or use the Makefile shortcut:"
echo "  make invoke-initialize CONTRACT_ID=$CONTRACT_ID ADMIN=<ADMIN> MEMBERS='[...]' THRESHOLD=2"
