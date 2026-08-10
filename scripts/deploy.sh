#!/usr/bin/env bash
# deploy.sh — Deploy quorumforge-contract to Testnet or Mainnet
# Usage:
#   ./scripts/deploy.sh testnet <source-account>
#   SAVE_ID=1 ./scripts/deploy.sh testnet <source-account>

set -euo pipefail

NETWORK="${1:-testnet}"
SOURCE="${2:-default}"
SAVE_ID="${SAVE_ID:-0}"
WASM="target/wasm32v1-none/release/quorumforge_contract.wasm"

echo "==> Building contract (stellar contract build --optimize=false)..."
stellar contract build --optimize=false

if [[ ! -f "$WASM" ]]; then
  echo "❌ Expected wasm at $WASM"
  exit 1
fi

WASM_SIZE=$(du -k "$WASM" | cut -f1)
echo "    Wasm size: ${WASM_SIZE}KB"

if [[ "$NETWORK" == "mainnet" ]]; then
  echo ""
  echo "⚠️  WARNING: Deploying to MAINNET. This costs real XLM."
  echo "   Source account: $SOURCE"
  read -rp "   Type 'yes' to confirm: " CONFIRM
  [[ "$CONFIRM" == "yes" ]] || { echo "Aborted."; exit 1; }
fi

echo "==> Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "$WASM" \
  --source-account "$SOURCE" \
  --network "$NETWORK" \
  --optimize=false)

if [[ "$SAVE_ID" == "1" ]]; then
  echo "$CONTRACT_ID" > ".contract-id.${NETWORK}"
  echo "    Contract ID saved to .contract-id.${NETWORK}"
fi

echo ""
echo "✅ Deployed successfully!"
echo "   Contract ID : $CONTRACT_ID"
echo "   Network     : $NETWORK"
echo ""
echo "Next — initialize:"
echo "  stellar contract invoke --id $CONTRACT_ID --source-account $SOURCE --network $NETWORK --send=yes -- initialize --admin <ADMIN> --members '[\"...\"]' --threshold 2"
