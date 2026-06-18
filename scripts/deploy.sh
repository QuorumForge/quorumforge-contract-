#!/usr/bin/env bash
# deploy.sh — Deploy quorumforge-contract to Testnet or Mainnet
# Usage:
#   ./scripts/deploy.sh testnet <source-account>
#   ./scripts/deploy.sh mainnet <source-account>

set -euo pipefail

NETWORK="${1:-testnet}"
SOURCE="${2:-default}"
WASM="target/wasm32-unknown-unknown/release/quorumforge_contract.wasm"
OPTIMIZED="target/quorumforge_contract.optimized.wasm"

echo "==> Building contract..."
cargo build --target wasm32-unknown-unknown --release

echo "==> Optimizing wasm..."
stellar contract optimize --wasm "$WASM" --wasm-out "$OPTIMIZED"

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
