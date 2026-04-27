default: build

all: test

# ── Variáveis ──────────────────────────────────────────────────────────────
# Substitua pelo endereço do contrato multisig após o deploy
MULTISIG_ID ?= CHANGE_ME
SOURCE      ?= maycon
NETWORK     ?= testnet

# ── Build / Dev ─────────────────────────────────────────────────────────────
test: build
	cargo test

build:
	stellar contract build
	@ls -l ../../target/wasm32v1-none/release/*.wasm

# Build otimizado para mainnet (sem spec metadata, reduz taxa de upload)
build-mainnet:
	stellar contract build --optimize
	python3 ../../scripts/strip_spec.py \
		../../target/wasm32v1-none/release/multisig.wasm \
		../../target/wasm32v1-none/release/multisig_mainnet.wasm
	@echo ""
	@ls -l ../../target/wasm32v1-none/release/multisig*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

# ── Deploy ───────────────────────────────────────────────────────────────────
# Parâmetros:
#   signers  — array JSON de chaves Ed25519 em hex (32 bytes cada), ordenado
#              crescente. Ex: make deploy signers='["aabbcc...","ddeeff..."]'
#   threshold — número mínimo de assinaturas. Ex: make deploy threshold=2
#
# Exemplo (2-de-3):
#   make deploy \
#     signers='["<HEX_PK1>","<HEX_PK2>","<HEX_PK3>"]' \
#     threshold=2
deploy:
	stellar contract deploy \
	  --wasm ../../target/wasm32v1-none/release/multisig.wasm \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  -- \
	  --signers '$(signers)' \
	  --threshold $(threshold)

# ── Consultas (somente leitura) ──────────────────────────────────────────────
get_signers:
	stellar contract invoke \
	  --id $(MULTISIG_ID) \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  -- get_signers

get_threshold:
	stellar contract invoke \
	  --id $(MULTISIG_ID) \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  -- get_threshold

# ── TTL ──────────────────────────────────────────────────────────────────────
# Renova o TTL do contrato (qualquer conta pode chamar, sem auth especial)
bump:
	stellar contract invoke \
	  --id $(MULTISIG_ID) \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  -- bump

# ── Gestão de signatários (requer N-of-M assinaturas do próprio multisig) ────
#
# ATENÇÃO: add_signer, remove_signer e set_threshold exigem que o próprio
# contrato multisig autorize a operação (N-of-M signatários devem assinar
# o auth entry). Isso não pode ser feito diretamente com --source-account.
#
# Use o SDK (JS/TS) para:
#   1. Simular a tx e obter o auth entry hash
#   2. Coletar N assinaturas Ed25519 dos signatários
#   3. Montar Vec<AccountSignature> e submeter
#
# Os alvos abaixo servem como referência da assinatura da função.
# Para invocar via CLI em modo de simulação (--send=no), use:

# Adicionar novo signatário
# Parâmetro: new_key=<HEX_32_BYTES>
# Ex: make add_signer new_key=aabbccdd...
add_signer:
	stellar contract invoke \
	  --id $(MULTISIG_ID) \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  --send=no \
	  -- add_signer \
	  --new_key $(new_key)

# Remover signatário existente
# Parâmetro: key=<HEX_32_BYTES>
# Ex: make remove_signer key=aabbccdd...
remove_signer:
	stellar contract invoke \
	  --id $(MULTISIG_ID) \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  --send=no \
	  -- remove_signer \
	  --key $(key)

# Alterar o threshold
# Parâmetro: threshold=<N>
# Ex: make set_threshold threshold=3
set_threshold:
	stellar contract invoke \
	  --id $(MULTISIG_ID) \
	  --source-account $(SOURCE) \
	  --network $(NETWORK) \
	  --send=no \
	  -- set_threshold \
	  --threshold $(threshold)
