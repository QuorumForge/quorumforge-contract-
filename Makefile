NETWORK        ?= testnet
CONTRACT_ID    ?=
ADMIN          ?=
SOURCE_ACCOUNT ?= default

WASM_PATH      = target/wasm32-unknown-unknown/release/quorumforge_contract.wasm
OPTIMIZED_WASM = target/quorumforge_contract.optimized.wasm

.PHONY: build test fmt lint clean optimize deploy-testnet deploy-mainnet \
        invoke-initialize invoke-create-proposal invoke-sign \
        invoke-get-proposal invoke-get-board invoke-get-stats \
        invoke-get-pending invoke-is-member invoke-has-signed

# ── Build ─────────────────────────────────────────────────────────────────────

build:
	cargo build --target wasm32-unknown-unknown --release

optimize: build
	stellar contract optimize --wasm $(WASM_PATH) --wasm-out $(OPTIMIZED_WASM)

# ── Dev ───────────────────────────────────────────────────────────────────────

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy --target wasm32-unknown-unknown -- -D warnings

clean:
	cargo clean

# ── Deploy ────────────────────────────────────────────────────────────────────

deploy-testnet: optimize
	stellar contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source $(SOURCE_ACCOUNT) \
		--network testnet

deploy-mainnet: optimize
	@echo "⚠️  Deploying to MAINNET. Press Ctrl-C to abort, Enter to continue."
	@read _
	stellar contract deploy \
		--wasm $(OPTIMIZED_WASM) \
		--source $(SOURCE_ACCOUNT) \
		--network mainnet

# ── Invoke helpers ────────────────────────────────────────────────────────────
# Usage: make invoke-initialize CONTRACT_ID=C... ADMIN=G... MEMBERS='["G1","G2","G3"]' THRESHOLD=2

invoke-initialize:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		-- initialize \
		--admin $(ADMIN) \
		--members '$(MEMBERS)' \
		--threshold $(THRESHOLD)

# Usage: make invoke-create-proposal CONTRACT_ID=C... PROPOSER=G... TYPE=ResolveIssue PAYLOAD='...'
invoke-create-proposal:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		-- create_proposal \
		--proposer $(PROPOSER) \
		--proposal_type '"$(TYPE)"' \
		--payload '$(PAYLOAD)' \
		--ttl_seconds null

# Usage: make invoke-sign CONTRACT_ID=C... SIGNER=G... PROPOSAL_ID=1
invoke-sign:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--source $(SOURCE_ACCOUNT) \
		--network $(NETWORK) \
		-- sign_proposal \
		--signer $(SIGNER) \
		--proposal_id $(PROPOSAL_ID)

# Usage: make invoke-get-proposal CONTRACT_ID=C... PROPOSAL_ID=1
invoke-get-proposal:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		-- get_proposal \
		--proposal_id $(PROPOSAL_ID)

# Usage: make invoke-get-board CONTRACT_ID=C...
invoke-get-board:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		-- get_board

# Usage: make invoke-get-stats CONTRACT_ID=C...
invoke-get-stats:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		-- get_stats

# Usage: make invoke-get-pending CONTRACT_ID=C...
invoke-get-pending:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		-- get_pending_proposals

# Usage: make invoke-is-member CONTRACT_ID=C... ADDR=G...
invoke-is-member:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		-- is_member \
		--addr $(ADDR)

# Usage: make invoke-has-signed CONTRACT_ID=C... PROPOSAL_ID=1 ADDR=G...
invoke-has-signed:
	stellar contract invoke \
		--id $(CONTRACT_ID) \
		--network $(NETWORK) \
		-- has_signed \
		--proposal_id $(PROPOSAL_ID) \
		--addr $(ADDR)
