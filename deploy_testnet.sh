#!/bin/bash
# Script de deploy para Stellar Testnet

set -e

echo "=== Escrow Contract Deploy Script ==="
echo ""

# Configurações
WASM_FILE="target/wasm32v1-none/release/escrow.wasm"
NETWORK="testnet"

# Verificar se o arquivo WASM existe
if [ ! -f "$WASM_FILE" ]; then
	echo "❌ Erro: Arquivo WASM não encontrado em $WASM_FILE"
	echo "   Execute 'make build' primeiro"
	exit 1
fi

echo "✓ Arquivo WASM encontrado: $WASM_FILE"
echo ""

# Verificar se Stella CLI está instalado
if ! command -v stellar &>/dev/null; then
	echo "❌ Erro: soroban CLI não está instalado"
	echo "   Execute: cargo install soroban-cli"
	exit 1
fi

echo "✓ Stella CLI instalado:"
stellar --version
echo ""

# Verificar rede
echo "📡 Configurando rede para Testnet..."
stellar network use testnet
echo ""

# Verificar identidades configuradas
echo "🔑 Identidades configuradas:"
stellar config identity list || echo "   Nenhuma identidade configurada"
echo ""

# Prompt para admin address
echo "📝 Informações necessárias:"
echo "   - Admin address (seu endereço na Testnet)"
echo "   - Collect on create: true/false (cobrar taxa na criação?)"
echo ""
# Verificar se o endereço precisa ser fundido (Friendbot)
if [[ "$ADMIN_ADDRESS" == G* ]]; then
	echo "🔍 Verificando se a conta $ADMIN_ADDRESS existe na Testnet..."
	if ! stellar keys balance "$ADMIN_ADDRESS" --network "$NETWORK" &>/dev/null; then
		echo "💡 Conta não encontrada. Tentando fundir via Friendbot..."
		curl -X POST "https://friendbot.stellar.org?addr=$ADMIN_ADDRESS"
		echo -e "\n✅ Conta fundida! Aguardando alguns segundos para sincronização..."
		sleep 5
	fi
fi

# Converter para boolean
if [ "$COLLECT_ON_CREATE" = "true" ]; then
	COLLECT_BOOL="true"
else
	COLLECT_BOOL="false"
fi

echo ""
echo "🚀 Iniciando deploy..."
echo ""

# Deploy do contrato
stellar contract deploy \
	--wasm "$WASM_FILE" \
	--source "$ADMIN_ADDRESS" \
	--network "$NETWORK" \
	-- --admin "$ADMIN_ADDRESS" \
	--collect-on-create "$COLLECT_BOOL"

echo ""
echo "✓ Deploy concluído!"
echo ""
echo "⚠ Próximos passos:"
echo "   1. Anote o Contract ID retornado acima"
echo "   2. Execute 'soroban contract invoke <CONTRACT_ID> --id <ID> --source <ADMIN_ADDRESS> --network testnet' para interagir"
echo "   3. Verifique em: https://stellar.expert/testnet"
