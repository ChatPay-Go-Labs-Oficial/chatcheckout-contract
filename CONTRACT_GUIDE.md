# Guia do Contrato Escrow - ChatCheckout

## Visão Geral

O contrato **ChatCheckout Escrow** é um sistema de pagamento seguro na blockchain Stellar que protege tanto compradores quanto vendedores em transações. O gerenciamento é feito através de contratos inteligentes Soroban com períodos de garantia opcionais e sistema de resolução de disputas.

### Principais Funcionalidades

- ✅ **Pagamento Protegido**: Fundos bloqueados até o vendedor receber pagamento
- ✅ **Período de Garantia**: Proteção automática para o comprador (mínimo 1 dia)
- ✅ **Early Release**: Comprador pode permitir que vendedor receba antes do prazo
- ✅ **Sistema de Disputas**: Resolução de conflitos com acordo mútuo ou arbitragem do admin
- ✅ **Taxas Configuráveis**: Taxas em basis points coletadas na criação ou liberação
- ✅ **Token Allowlist**: Admin controla quais tokens são aceitos

---

## Estrutura de Dados

### EscrowData

Cada escrow contém:

```rust
pub struct EscrowData {
    pub buyer: Address,           // Endereço do comprador
    pub seller: Address,          // Endereço do vendedor
    pub amount: i128,             // Valor do pagamento
    pub asset: Address,           // Endereço do token usado
    pub created_at: u64,          // Timestamp de criação
    pub release_at: u64,          // Timestamp de liberação automática
    pub status: EscrowStatus,     // Status atual (Active, Released, Refunded, Disputed)
    pub product_id: String,       // ID do produto/venda
    pub guarantee_days: u32,      // Dias de garantia
    pub fee_bps: u32,             // Taxa em basis points (100 = 1%)
    pub allow_early_release: bool, // Vendedor pode liberar antes?
    pub buyer_proposal: Option<bool>,    // Proposta do comprador (true = favor buyer)
    pub seller_proposal: Option<bool>,   // Proposta do vendedor (true = favor buyer)
}
```

### EscrowStatus

- `Active`: Escrow em andamento, fundos bloqueados
- `Released`: Pagamento liberado para o vendedor
- `Refunded`: Reembolsado para o comprador
- `Disputed`: Em disputa, aguardando resolução

---

## Funções do Contrato

### 1. Inicialização

#### `__constructor(env, admin, collect_on_create)`

**Quem pode chamar**: Qualquer um (apenas uma vez)
**Propósito**: Inicializa o contrato com o administrador

```javascript
// Parâmetros:
admin: Address           // Endereço do administrador
collect_on_create: bool  // true = coletar taxa na criação, false = coletar na liberação

// Exemplo de chamada:
const tx = new TransactionBuilder(account, { fee: 100 })
  .addOperation(
    Operation.invokeContractFunction({
      contract: contractId,
      function: "__constructor",
      args: [
        Address.admin(),           // admin
        xdr.bool(true)             // collect_on_create
      ]
    })
  )
  .build();
```

---

### 2. Configuração

#### `update_config(env, new_admin, collect_on_create)`

**Quem pode chamar**: Admin atual
**Propósito**: Atualiza configurações do contrato

```javascript
// Exemplo:
await contract.update_config({
  new_admin: newAdminAddress,
  collect_on_create: false
});
```

#### `get_config(env) -> Config`

**Quem pode chamar**: Público
**Retorna**: Configuração atual do contrato

#### `admin_withdraw(env, asset, amount)`

**Quem pode chamar**: Admin apenas
**Propósito**: Retira fundos (taxas acumuladas) do contrato para a conta do administrador.

```javascript
// Exemplo:
await contract.admin_withdraw({
  asset: USDCAddress,
  amount: BigInt(500_000_000) // Retirar 500 USDC
});
```

---

### 3. Gerenciamento de Tokens

#### `add_allowed_token(env, token)`

**Quem pode chamar**: Admin
**Propósito**: Adiciona um token à lista de permitidos

```javascript
// Exemplo:
await contract.add_allowed_token({
  token: USDCAddress
});
```

#### `remove_allowed_token(env, token)`

**Quem pode chamar**: Admin
**Propósito**: Remove um token da lista de permitidos

#### `get_allowed_tokens(env) -> Vec<Address>`

**Quem pode chamar**: Público
**Retorna**: Lista de tokens permitidos

#### `is_token_allowed(env, token) -> bool`

**Quem pode chamar**: Público
**Retorna**: true se o token é permitido

---

### 4. Criação de Escrow

#### `create_escrow(env, buyer, seller, amount, asset, fee_bps, guarantee_days, product_id, allow_early_release) -> u64`

**Quem pode chamar**: Comprador (requer autorização)
**Retorna**: ID do escrow criado

```javascript
// Exemplo de chamada:
const escrowId = await contract.create_escrow({
  buyer: buyerAddress,
  seller: sellerAddress,
  amount: BigInt(100_000_000),  // 100 tokens (6 decimais)
  asset: USDCAddress,
  fee_bps: 400,                 // 4% (400 basis points)
  guarantee_days: 7,            // 7 dias de garantia
  product_id: "product-123",
  allow_early_release: false    // Vendedor deve esperar 7 dias
});
```

**Validações**:
- ✅ `amount > 0`
- ✅ `guarantee_days >= 1` e `<= 36500` (mínimo 1 dia!)
- ✅ `fee_bps <= 10000` (máximo 100%)
- ✅ `product_id` não vazio
- ✅ `asset` está na allowlist
- ✅ Comprador tem saldo suficiente
- ✅ Comprador autorizou a transação

**Fluxo de Execução**:

1. **Validação de parâmetros**
2. **Verificação se o token é permitido**
3. **Cálculo da taxa**: `fee = amount * fee_bps / 10000`
4. **Transferência do comprador para o contrato**: `token.transfer(buyer, contract, amount)`
5. **Coleta de taxa** (se `collect_on_create = true`): `token.transfer(buyer, admin, fee)`
6. **Criação do escrow** com ID sequencial
7. **Emissão do evento**: `CreateEscrowEvent`

---

### 5. Liberação de Pagamento

#### `release_payment(env, escrow_id, seller, nonce)`

**Quem pode chamar**: Vendedor (requer autorização e nonce)
**Propósito**: Libera o pagamento para o vendedor

```javascript
// Exemplo de chamada:
await contract.release_payment({
  escrow_id: 1,
  seller: sellerAddress,
  nonce: 0  // Deve obter o nonce atual primeiro
});
```

**Validações**:
- ✅ Escrow existe e está em status `Active`
- ✅ Caller é o vendedor do escrow
- ✅ Nonce está correto (replay protection)
- ✅ **Período de garantia expirou** OU **allow_early_release = true**

**Fluxo de Execução**:

1. **Verificação e incremento do nonce**
2. **Verificação de autorização do vendedor**
3. **Leitura do escrow**
4. **Verificação do período de garantia**:
   - Se `allow_early_release = true`: Pode liberar imediatamente
   - Se `allow_early_release = false`: Só pode liberar após `release_at`
5. **Processamento da taxa** (se não coletada na criação):
   - Calcula: `fee = amount * fee_bps / 10000`
   - Transfere: `token.transfer(contract, admin, fee)`
   - Vendedor recebe: `amount - fee`
6. **Transferência para o vendedor**: `token.transfer(contract, seller, to_seller)`
7. **Atualização do status**: `Released`
8. **Emissão do evento**: `ReleasePaymentEvent`

---

### 6. Reembolso

#### `request_refund(env, escrow_id, buyer, nonce)`

**Quem pode chamar**: Comprador (requer autorização e nonce)
**Propósito**: Solicita reembolso total

```javascript
// Exemplo de chamada:
await contract.request_refund({
  escrow_id: 1,
  buyer: buyerAddress,
  nonce: 1
});
```

**Validações**:
- ✅ Escrow existe e está em status `Active`
- ✅ Caller é o comprador do escrow
- ✅ Nonce está correto
- ✅ **Período de garantia NÃO expirou** (`now < release_at`)

**Fluxo de Execução**:

1. **Verificação e incremento do nonce**
2. **Verificação de autorização do comprador**
3. **Leitura do escrow**
4. **Verificação da janela de reembolso**: `now < release_at`
5. **Transferência de volta para o comprador**: `token.transfer(contract, buyer, amount)`
6. **Atualização do status**: `Refunded`
7. **Emissão do evento**: `RequestRefundEvent`

---

### 7. Sistema de Disputas

#### `dispute_escrow(env, escrow_id, caller, nonce)`

**Quem pode chamar**: Comprador OU Vendedor
**Propósito**: Inicia uma disputa

```javascript
// Exemplo de chamada:
await contract.dispute_escrow({
  escrow_id: 1,
  caller: buyerAddress,  // ou sellerAddress
  nonce: 2
});
```

**Validações**:
- ✅ Escrow existe
- ✅ Status está em `Active` **OU** (`Released` COM `allow_early_release: true` E antes de `release_at`)
- ✅ Caller é comprador ou vendedor
- ✅ Nonce está correto

**Fluxo**:
1. Verifica nonce
2. Verifica autorização
3. Verifica que é parte da transação
4. Altera status para `Disputed`
5. Emite `DisputeEscrowEvent`

---

#### `propose_resolution(env, escrow_id, caller, nonce, favor_buyer)`

**Quem pode chamar**: Comprador OU Vendedor
**Propósito**: Propõe uma resolução para a disputa

```javascript
// Exemplo de chamada:
await contract.propose_resolution({
  escrow_id: 1,
  caller: buyerAddress,  // ou sellerAddress
  nonce: 3,
  favor_buyer: true      // true = reembolsar buyer, false = pagar seller
});
```

**Validações**:
- ✅ Escrow está em status `Disputed`
- ✅ Caller é comprador ou vendedor
- ✅ Caller ainda não propôs (não pode mudar proposta)
- ✅ Nonce está correto

**Fluxo**:
1. Verifica nonce
2. Verifica autorização
3. Verifica que está disputado
4. Armazena proposta:
   - Se caller = buyer: `buyer_proposal = Some(favor_buyer)`
   - Se caller = seller: `seller_proposal = Some(favor_buyer)`
5. Emite `ProposeResolutionEvent`

---

#### `resolve_dispute(env, escrow_id, caller, nonce)`

**Quem pode chamar**: Comprador OU Vendedor
**Propósito**: Resolve disputa quando AMBOS concordam

```javascript
// Exemplo de chamada:
await contract.resolve_dispute({
  escrow_id: 1,
  caller: buyerAddress,  // ou sellerAddress
  nonce: 4
});
```

**Validações**:
- ✅ Escrow está em status `Disputed`
- ✅ Caller é comprador ou vendedor
- ✅ **Ambas as partes propuseram**
- ✅ **Ambas concordaram** (propostas são iguais)
- ✅ Nonce está correto

**Fluxo**:
1. Verifica nonce
2. Verifica autorização
3. Verifica que está disputado
4. **Valida acordo**:
   - `buyer_proposal = seller_proposal`
   - Se diferentes: erro `BothPartiesMustAgree`
5. **Processa resolução**:
   - Se `favor_buyer = true`: Reembolsa comprador
   - Se `favor_buyer = false`: Paga vendedor
6. Deduz taxa se não coletada na criação
7. Atualiza status (`Refunded` ou `Released`)
8. Emite `ResolveDisputeEvent`

---

#### `admin_resolve_dispute(env, escrow_id, favor_buyer, nonce)`

**Quem pode chamar**: **Administrador** apenas
**Propósito**: Resolve disputa por arbitragem (quando partes não concordam)

```javascript
// Exemplo de chamada:
await contract.admin_resolve_dispute({
  escrow_id: 1,
  favor_buyer: false,  // Admin decide em favor do vendedor
  nonce: 0  // Nonce do admin
});
```

**Validações**:
- ✅ Caller é o admin atual
- ✅ Escrow está em status `Disputed`
- ✅ Nonce do admin está correto

**Fluxo**:
1. Verifica nonce do admin
2. Verifica autorização do admin
3. Verifica que está disputado
4. **Processa resolução imediatamente** (não precisa de acordo)
5. Deduz taxa se não coletada na criação
6. Atualiza status
7. Emite `ResolveDisputeEvent` com `resolved_by = admin`

---

### 8. Funções Auxiliares

#### `get_escrow(env, escrow_id) -> EscrowData`

**Quem pode chamar**: Público
**Retorna**: Dados completos do escrow

```javascript
const escrow = await contract.get_escrow({ escrow_id: 1 });
console.log(escrow.status);        // "Active", "Released", "Refunded", "Disputed"
console.log(escrow.amount);        // 100000000
console.log(escrow.allow_early_release);  // false
```

#### `get_nonce(env, user) -> u64`

**Quem pode chamar**: Público
**Retorna**: Nonce atual do usuário (para replay protection)

```javascript
const nonce = await contract.get_nonce({ user: buyerAddress });
console.log(nonce);  // 0, 1, 2, ...
```

---

## Fluxos de Execução

### Fluxo 1: Happy Path (Sem Early Release)

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. SETUP                                                        │
└─────────────────────────────────────────────────────────────────┘
     Admin              Contrato                Buyer
        │                    │                     │
        │ ① __constructor() │                     │
        │───────────────────>│                     │
        │                    │                     │
        │ ② add_allowed_token()                     │
        │───────────────────>│                     │
        │                    │                     │

┌─────────────────────────────────────────────────────────────────┐
│ 2. CRIAÇÃO DO ESCROW                                            │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Contrato                Seller               Token
        │                    │                     │                    │
        │ ① create_escrow(   │                     │                    │
        │    amount=100,      │                     │                    │
        │    guarantee_days=7 │                     │                    │
        │    allow_early=false│                     │                    │
        │                   )│                     │                    │
        │───────────────────>│                     │                    │
        │                    │ ② transfer()        │                    │
        │                    │───────────────────> │                    │
        │                    │<─────────────────── │                    │
        │                    │                     │                    │
        │                    │ ③ Se collect_on_create=true:            │
        │                    │    transfer(fee)───>│                    │
        │                    │                     │                    │
        │ ④ return escrow_id=1                     │                    │
        │<───────────────────│                     │                    │
        │                    │                     │                    │
     [STATUS: Active, release_at = now + 7 days]                     │

┌─────────────────────────────────────────────────────────────────┐
│ 3. VENDEDOR AGUARDA 7 DIOS                                      │
└─────────────────────────────────────────────────────────────────┘
     [Time passes... guarantee_days = 7 elapsed]

┌─────────────────────────────────────────────────────────────────┐
│ 4. LIBERAÇÃO DE PAGAMENTO                                       │
└─────────────────────────────────────────────────────────────────┘
     Seller             Contrato                Token               Admin
        │                    │                     │                    │
        │ ① get_nonce()=0    │                     │                    │
        │───────────────────>│                     │                    │
        │<───────────────────│                     │                    │
        │                    │                     │                    │
        │ ② release_payment( │                     │                    │
        │    escrow_id=1,     │                     │                    │
        │    nonce=0          │                     │                    │
        │                   )│                     │                    │
        │───────────────────>│                     │                    │
        │                    │                     │                    │
        │                    │ ③ Se collect_on_create=false:           │
        │                    │    transfer(fee)───>│                    │
        │                    │<────────────────────│                    │
        │                    │                     │                    │
        │                    │ ④ transfer(amount-fee)                   │
        │                    │───────────────────>│                    │
        │                    │<────────────────────│                    │
        │                    │                     │                    │
        │ ⑤ OK               │                     │                    │
        │<───────────────────│                     │                    │
        │                    │                     │                    │
     [STATUS: Released]                                                  │
```

---

### Fluxo 2: Happy Path (COM Early Release)

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. CRIAÇÃO COM EARLY RELEASE                                    │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Contrato                Seller
        │                    │                     │
        │ ① create_escrow(   │                     │
        │    allow_early=true │  ← FLAG DIFERENTE!  │
        │                   )│                     │
        │───────────────────>│                     │
        │ ② escrow_id=1      │                     │
        │<───────────────────│                     │
        │                    │                     │

┌─────────────────────────────────────────────────────────────────┐
│ 2. VENDEDOR PODE LIBERAR IMEDIATAMENTE                         │
└─────────────────────────────────────────────────────────────────┘
     Seller             Contrato
        │                    │
        │ ① release_payment( │
        │    escrow_id=1      │  ← NÃO PRECISA ESPERAR!
        │                   )│
        │───────────────────>│
        │                    │
        │ ② ✅ PODE LIBERAR   │
        │   (allow_early=true)│
        │<───────────────────│
        │                    │
     [STATUS: Released]
```

---

### Fluxo 3: Reembolso Dentro do Período de Garantia

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. CRIAÇÃO DO ESCROW                                            │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Contrato
        │                    │
        │ ① create_escrow(   │
        │    guarantee_days=7 │
        │                   )│
        │───────────────────>│
        │ ② escrow_id=1      │
        │<───────────────────│
        │                    │

┌─────────────────────────────────────────────────────────────────┐
│ 2. COMPRADOR QUER REEMBOLSO (ANTES DOS 7 DIOS)                │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Contrato                Token
        │                    │                     │
        │ ① get_nonce()=1    │                     │
        │───────────────────>│                     │
        │<───────────────────│                     │
        │                    │                     │
        │ ② request_refund(  │                     │
        │    escrow_id=1,     │                     │
        │    nonce=1          │                     │
        │                   )│                     │
        │───────────────────>│                     │
        │                    │                     │
        │                    │ ③ transfer(amount)──>│
        │                    │<────────────────────││
        │                    │                     │
        │ ④ ✅ REEMBOLSADO    │                     │
        │<───────────────────│                     │
        │                    │                     │
     [STATUS: Refunded]
```

---

### Fluxo 4: Disputa e Resolução (Acordo Mútuo)

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. CRIAÇÃO DO ESCROW                                            │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Seller
        │                    │
        │ ① create_escrow() │
        │───────────────────>│
        │ ② escrow_id=1      │
        │<───────────────────│
        │                    │

┌─────────────────────────────────────────────────────────────────┐
│ 2. PROBLEMA! COMPRADOR INICIA DISPUTA                         │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Contrato
        │                    │
        │ ① get_nonce()=2    │
        │───────────────────>│
        │<───────────────────│
        │                    │
        │ ② dispute_escrow(   │
        │    escrow_id=1,     │
        │    nonce=2          │
        │                   )│
        │───────────────────>│
        │                    │
        │ ③ ✅ DISPUTA CRIADA│
        │<───────────────────│
        │                    │
     [STATUS: Disputed]

┌─────────────────────────────────────────────────────────────────┐
│ 3. AMBAS AS PARTES PROPOEM RESOLUÇÃO                          │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Seller             Contrato
        │                    │                    │
        │ ① propose_resolution(│                    │
        │    favor_buyer=true  │                    │
        │                   )  │                    │
        │─────────────────────>│                    │
        │                    │                    │
        │                    │ ② propose_resolution(│
        │                    │    favor_buyer=true  │  ← CONCORDAM!
        │                    │                   )  │
        │                    │<─────────────────────│
        │                    │                    │
        │                    │ ③ buyer_proposal = Some(true)
        │                    │    seller_proposal = Some(true)
        │                    │                    │

┌─────────────────────────────────────────────────────────────────┐
│ 4. RESOLUÇÃO DA DISPUTA (ACORDO MÚTUO)                        │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Seller             Contrato                Token
        │                    │                    │                    │
        │ ① resolve_dispute(  │                    │                    │
        │    nonce=3          │                    │                    │
        │                   )  │                    │                    │
        │─────────────────────>│                    │                    │
        │                    │                    │                    │
        │                    │                    │ ② ✅ ACORDO!       │
        │                    │                    │   buyer_proposal == │
        │                    │                    │   seller_proposal   │
        │                    │                    │                    │
        │                    │                    │ ③ Se collect_on_create=false:           │
        │                    │                    │    transfer(fee)───>│                    │
        │                    │                    │<────────────────────│                    │
        │                    │                    │                    │
        │                    │                    │ ④ transfer(amount-fee)                   │
        │                    │                    │───────────────────>│                    │
        │                    │                    │<────────────────────│                    │
        │                    │                    │                    │
        │ ⑤ ✅ REEMBOLSADO    │                    │                    │
        │<───────────────────││                    │                    │
        │                    │                    │                    │
     [STATUS: Refunded, favor_buyer=true]
```

---

### Fluxo 5: Disputa e Arbitragem do Admin

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. DISPUTA CRIADA (mesmo do Fluxo 4)                          │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Seller             Contrato
        │                    │                    │
        │ ① dispute_escrow()  │                    │
        │─────────────────────>│                    │
        │                    │                    │
     [STATUS: Disputed]

┌─────────────────────────────────────────────────────────────────┐
│ 2. PARTES PROPOEM RESOLUÇÕES DIFERENTES (DESENCORDO)          │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Seller             Contrat
        │                    │                    │
        │ ① propose_resolution(│                    │
        │    favor_buyer=true  │   Quero reembolso! │
        │                   )  │                    │
        │─────────────────────>│                    │
        │                    │                    │
        │                    │ ② propose_resolution(│
        │                    │    favor_buyer=false │   Quero pagamento!
        │                    │                   )  │
        │                    │<─────────────────────│
        │                    │                    │
        │                    │ ③ buyer_proposal = Some(true)
        │                    │    seller_proposal = Some(false)
        │                    │    DIVERGENTES!     │

┌─────────────────────────────────────────────────────────────────┐
│ 3. TENTATIVA DE RESOLUÇÃO FALHA                                │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Contrato
        │                    │
        │ ① resolve_dispute()│
        │───────────────────>│
        │ ② ❌ BothPartiesMustAgree
        │<───────────────────│
        │                    │
     [PRECISA DE ARBITRAGEM!]

┌─────────────────────────────────────────────────────────────────┐
│ 4. ADMIN FAZ ARBITRAGEM                                        │
└─────────────────────────────────────────────────────────────────┘
     Admin              Contrato                Token
        │                    │                     │
        │ ① get_nonce()=0     │                     │
        │───────────────────>│                     │
        │<───────────────────│                     │
        │                    │                     │
        │ ② admin_resolve_dispute(                  │
        │      favor_buyer=false  ← DECIDE EM FAVOR DO SELLER│
        │      nonce=0                              │
        │                                         )  │
        │───────────────────>│                     │
        │                    │                     │
        │                    │ ③ ✅ ADMIN DECIDE!  │
        │                    │                     │
        │                    │ ④ transfer(fee)────>│
        │                    │<────────────────────│
        │                    │                     │
        │                    │ ⑤ transfer(amount-fee)                   │
        │                    │───────────────────>│                    │
        │                    │<────────────────────│                    │
        │                    │                     │
        │ ⑥ ✅ RESOLVIDO      │                     │
        │<───────────────────│                     │
        │                    │                     │
     [STATUS: Released, resolved_by=admin]
```

---

### Fluxo 6: Early Release + Disputa (Caso de Edge)

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. CRIAÇÃO COM EARLY RELEASE                                   │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Seller
        │                    │
        │ ① create_escrow(   │
        │    allow_early=true │  Buyer confia no seller
        │                   )│
        │───────────────────>│
        │                    │

┌─────────────────────────────────────────────────────────────────┐
│ 2. SELLER LIBERA IMEDIATAMENTE                                 │
└─────────────────────────────────────────────────────────────────┘
     Seller             Contrato                Token
        │                    │                     │
        │ ① release_payment() │                     │
        │───────────────────>│                     │
        │                    │ ② transfer()───────>│
        │                    │<────────────────────│
        │                    │                     │
        │ ③ ✅ PAGO          │                     │
        │<───────────────────│                     │
        │                    │                     │
     [STATUS: Released] ⚠️ ATENÇÃO: Status mudou!

┌─────────────────────────────────────────────────────────────────┐
│ 3. COMPRADOR QUER REEMBOLSO (FUI ENGANADO!)                   │
└─────────────────────────────────────────────────────────────────┘
     Buyer              Contrato
        │                    │
        │ ① dispute_escrow()  │
        │───────────────────>│
        │ ② ✅ DISPUTA ACEITA │
        │   (Dentro do prazo)│
        │<───────────────────│
        │                    │
     [STATUS: Disputed]

**⚠️ Observação**: Mesmo que o status fosse `Released` (devido ao Early Release), a disputa ainda é permitida se estiver dentro de `guarantee_days`.
```

**⚠️ Proteção no Early Release**: Mesmo com o pagamento liberado antecipadamente, o comprador mantém o direito de abrir uma disputa até o fim do período de garantia (`guarantee_days`). Isso desencoraja fraudes de vendedores que solicitam liberação imediata.

---

## Sistema de Nonce (Replay Protection)

Cada usuário tem um nonce que incrementa a cada chamada de função que requer proteção contra replay attacks:

**Funções que usam nonce**:
- `release_payment()`
- `request_refund()`
- `dispute_escrow()`
- `propose_resolution()`
- `resolve_dispute()`
- `admin_resolve_dispute()`

**Como funciona**:

```javascript
// 1. Obter nonce atual
const nonce = await contract.get_nonce({ user: buyerAddress });
// nonce = 0

// 2. Chamar função com nonce atual
await contract.release_payment({
  escrow_id: 1,
  seller: sellerAddress,
  nonce: 0  // ← Deve ser o nonce atual!
});

// 3. Nonce é incrementado automaticamente para 1
// Próxima chamada deve usar nonce = 1

// Se tentar usar nonce = 0 novamente:
await contract.release_payment({
  escrow_id: 1,
  seller: sellerAddress,
  nonce: 0  // ❌ InvalidNonce!
});
```

**Por que isso é importante?**

Previne que alguém capture sua transação assinada e a reexecute múltiplas vezes (ataque de replay).

---

## Cálculo de Taxas

### Fórmula

```
fee = (amount × fee_bps) / 10000
```

### Exemplos

| Amount  | fee_bps | Fee      | Valor Líquido |
|---------|---------|----------|---------------|
| 100     | 100     | 1        | 99            |
| 1000    | 400     | 40       | 960           |
| 10000   | 500     | 500      | 9500          |
| 100     | 10000   | 100      | 0             |

### Quando a Taxa é Cobrada?

**1. collect_on_create = true** (configuração do contrato)
```
create_escrow():
  1. buyer → contract: amount
  2. buyer → admin: fee
  3. No release: seller recebe amount total
```

**2. collect_on_create = false** (configuração do contrato)
```
create_escrow():
  1. buyer → contract: amount total

release_payment():
  1. contract → admin: fee
  2. contract → seller: amount - fee
```

---

## Codificação de Status

| Status        | Código | Descrição                          |
|---------------|--------|------------------------------------|
| Active        | 0      | Escrow em andamento                |
| Released      | 1      | Pagamento liberado para vendedor   |
| Refunded      | 2      | Reembolsado para comprador         |
| Disputed      | 3      | Em disputa                         |

---

## Codificação de Erros

| Código | Erro                        | Quando Ocorre                              |
|--------|-----------------------------|--------------------------------------------|
| 1      | AlreadyInitialized          | __constructor chamado mais de uma vez      |
| 2      | ConfigNotInitialized        | Config não encontrada                      |
| 3      | Unauthorized                | Caller não tem permissão                   |
| 4      | EscrowNotFound              | Escrow ID não existe                       |
| 5      | InvalidAmount               | amount <= 0                                |
| 6      | InvalidFeeBps               | fee_bps > 10000                            |
| 7      | InvalidGuaranteeDays        | guarantee_days < 1 ou > 36500              |
| 8      | InvalidProductId            | product_id vazio                           |
| 9      | EscrowNotActive             | Escrow não está Active                     |
| 10     | GuaranteePeriodNotExpired   | Tentou release antes do prazo              |
| 11     | GuaranteePeriodExpired      | Tentou refund depois do prazo              |
| 12     | FeeExceedsAmount            | Taxa maior que o amount                    |
| 13     | InvalidSignature            | Assinatura meta-transação inválida         |
| 14     | InvalidNonce                | Nonce incorreto (replay attack detectado)  |
| 15     | SignatureExpired            | Assinatura expirada                        |
| 16     | InvalidFunctionSelector     | Seletor de função inválido                 |
| 17     | CounterOverflow             | Muitos escrows (overflow do contador)      |
| 18     | TokenNotAllowed             | Token não está na allowlist                |
| 19     | NotDisputed                 | Escrow não está Disputed                   |
| 20     | AlreadyDisputed             | Escrow já está Disputed                    |
| 21     | NoDisputeToResolve          | Não há propostas para resolver             |
| 22     | DisputeAlreadyResolved      | Disputa já resolvida                       |
| 23     | BothPartiesMustAgree        | Propostas divergentes                      |
| 24     | AlreadyProposed             | Parte já propôs (não pode mudar)          |

---

## Eventos Emitidos

### CreateEscrowEvent
```javascript
{
  topics: ["create"],
  data: {
    escrow_id: 1,
    buyer: "GD...",
    seller: "GD...",
    amount: 100000000,
    asset: "CC...",
    fee_bps: 400,
    guarantee_days: 7,
    product_id: "product-123",
    allow_early_release: false
  }
}
```

### ReleasePaymentEvent
```javascript
{
  topics: ["release"],
  data: {
    escrow_id: 1,
    seller: "GD...",
    amount: 100000000,
    fee: 4000000,
    to_seller: 96000000
  }
}
```

### RequestRefundEvent
```javascript
{
  topics: ["refund"],
  data: {
    escrow_id: 1,
    buyer: "GD...",
    amount: 100000000,
    asset: "CC..."
  }
}
```

### DisputeEscrowEvent
```javascript
{
  topics: ["dispute"],
  data: {
    escrow_id: 1,
    initiator: "GD..."  // buyer ou seller
  }
}
```

### ProposeResolutionEvent
```javascript
{
  topics: ["propose"],
  data: {
    escrow_id: 1,
    proposer: "GD...",     // buyer ou seller
    favor_buyer: true
  }
}
```

### ResolveDisputeEvent
```javascript
{
  topics: ["resolve"],
  data: {
    escrow_id: 1,
    favor_buyer: true,
    amount: 100000000,
    fee: 4000000,
    recipient: "GD...",    // Quem recebeu os fundos
    resolved_by: "GD..."   // buyer, seller, ou admin
  }
}
### AdminWithdrawEvent
```javascript
{
  topics: ["admin_withdraw"],
  data: {
    admin: "GD...",
    asset: "CC...",
    amount: 500000000
  }
}
```

---

## Exemplos de Uso Completo

### Exemplo 1: Compra Básica com Sucesso

```javascript
import { Contract } from './generated';
import { Server, TransactionBuilder, Operation, Asset } from 'stellar-sdk';

// Setup
const server = new Server('https://horizon-testnet.stellar.org');
const contractId = 'CD...';
const contract = new Contract(contractId);

// 1. Setup inicial (admin)
const adminKeypair = Keypair.random();
await server.installContract({ contractId, wasm: contractWasm });
await contract.__constructor({
  admin: adminKeypair.publicKey(),
  collect_on_create: false
});

// Adicionar USDC à allowlist
const usdcAddress = 'CC...';
await contract.add_allowed_token({
  token: usdcAddress
});

// 2. Comprador cria escrow
const buyerKeypair = Keypair.random();
const sellerKeypair = Keypair.random();
const escrowId = await contract.create_escrow({
  buyer: buyerKeypair.publicKey(),
  seller: sellerKeypair.publicKey(),
  amount: BigInt(100_000_000),  // 100 USDC
  asset: usdcAddress,
  fee_bps: 400,  // 4%
  guarantee_days: 7,  // 7 dias
  product_id: "product-123",
  allow_early_release: false
});
console.log('Escrow criado:', escrowId);  // 1

// 3. Vendedor aguarda 7 dias...

// 4. Vendedor libera pagamento
await contract.release_payment({
  escrow_id: 1,
  seller: sellerKeypair.publicKey(),
  nonce: 0
});
console.log('Pagamento liberado!');

// Verificar status
const escrow = await contract.get_escrow({ escrow_id: 1 });
console.log('Status:', escrow.status);  // "Released"
```

---

### Exemplo 2: Compra com Early Release

```javascript
// Comprador confia no vendedor, permite early release
const escrowId = await contract.create_escrow({
  buyer: buyerKeypair.publicKey(),
  seller: sellerKeypair.publicKey(),
  amount: BigInt(100_000_000),
  asset: usdcAddress,
  fee_bps: 400,
  guarantee_days: 30,  // 30 dias de garantia
  product_id: "product-456",
  allow_early_release: true  // ← FLAG IMPORTANTE!
});

// Vendedor pode liberar IMEDIATAMENTE
await contract.release_payment({
  escrow_id: 1,
  seller: sellerKeypair.publicKey(),
  nonce: 0
});
console.log('Pagamento liberado imediatamente!');
```

---

### Exemplo 3: Reembolso

```javascript
// 1. Criar escrow
const escrowId = await contract.create_escrow({
  buyer: buyerKeypair.publicKey(),
  seller: sellerKeypair.publicKey(),
  amount: BigInt(100_000_000),
  asset: usdcAddress,
  fee_bps: 400,
  guarantee_days: 7,
  product_id: "product-789",
  allow_early_release: false
});

// 2. Comprador decide querer reembolso (dentro de 7 dias)
await contract.request_refund({
  escrow_id: 1,
  buyer: buyerKeypair.publicKey(),
  nonce: 1  // Nonce do buyer
});
console.log('Reembolsado com sucesso!');

// Verificar status
const escrow = await contract.get_escrow({ escrow_id: 1 });
console.log('Status:', escrow.status);  // "Refunded"
```

---

### Exemplo 4: Disputa com Acordo Mútuo

```javascript
// 1. Criar escrow
const escrowId = await contract.create_escrow({...});
// ...tempo passa...

// 2. Problema! Comprador inicia disputa
await contract.dispute_escrow({
  escrow_id: 1,
  caller: buyerKeypair.publicKey(),
  nonce: 2
});
console.log('Disputa iniciada');

// 3. Ambas as partes propõem resolução
await contract.propose_resolution({
  escrow_id: 1,
  caller: buyerKeypair.publicKey(),
  nonce: 3,
  favor_buyer: true  // Buyer quer reembolso
});

await contract.propose_resolution({
  escrow_id: 1,
  caller: sellerKeypair.publicKey(),
  nonce: 1,  // Nonce do seller
  favor_buyer: true  // Seller concorda com reembolso
});
console.log('Ambos concordaram em reembolsar');

// 4. Resolver disputa
await contract.resolve_dispute({
  escrow_id: 1,
  caller: buyerKeypair.publicKey(),
  nonce: 4
});
console.log('Disputa resolvida! Buyer reembolsado.');
```

---

### Exemplo 5: Disputa com Arbitragem do Admin

```javascript
// 1. Disputa iniciada
await contract.dispute_escrow({
  escrow_id: 1,
  caller: buyerKeypair.publicKey(),
  nonce: 2
});

// 2. Partes discordam
await contract.propose_resolution({
  escrow_id: 1,
  caller: buyerKeypair.publicKey(),
  nonce: 3,
  favor_buyer: true  // Buyer quer reembolso
});

await contract.propose_resolution({
  escrow_id: 1,
  caller: sellerKeypair.publicKey(),
  nonce: 1,
  favor_buyer: false  // Seller quer pagamento
});

// 3. Tentativa de resolução falha
try {
  await contract.resolve_dispute({
    escrow_id: 1,
    caller: buyerKeypair.publicKey(),
    nonce: 4
  });
} catch (e) {
  console.log('Erro:', e.message);  // "BothPartiesMustAgree"
}

// 4. Admin intervém
await contract.admin_resolve_dispute({
  escrow_id: 1,
  favor_buyer: false,  // Admin decide em favor do seller
  nonce: 0  // Nonce do admin
});
console.log('Admin decidiu: Seller recebe pagamento');
```

---

## Considerações de Segurança

### Para Compradores

1. **Sempre use `allow_early_release = false`** para vendedores desconhecidos
2. **Verifique o período de garantia** adequado para o tipo de produto/serviço
3. **Se usar early release**, entenda que está abrindo mão da proteção
4. **Inicie disputa prontamente** se houver problemas

### Para Vendedores

1. **Aguarde o período de garantia expirar** antes de liberar pagamento
2. **Se receber early release**, cumpra com sua obrigação de entrega
3. **Em disputas**, proponha resolução de boa-fé
4. **Lembre-se**: Disputas ainda podem ser abertas após o release se o `allow_early_release` foi usado e o prazo de garantia ainda não venceu.

### Para Admins

1. **Use `admin_resolve_dispute`** apenas quando necessário
2. **Analise ambos os lados** antes de decidir
3. **Documente suas decisões** para transparência
4. **Considere implementar um sistema de votação** ou múltiplos árbitros para decisões importantes

---

## Melhores Práticas

### 1. Escolha de guarantee_days

```
Tipo de Produto/Serviço    | guarantee_days Recomendado
----------------------------|---------------------------
Produto digital            | 1-7 dias
Produto físico (envio)     | 14-30 dias
Serviço recorrente          | 7-14 dias
Projeto customizado         | 30-90 dias
```

### 2. Configuração de Taxas

```
Cenário                      | collect_on_create | Razão
------------------------------|-------------------|------------------------------
Marketplace padrão           | true              | Taxa garantida para admin
Transações P2P               | false             | Taxa apenas se sucesso
Serviços de alto valor       | false             | Menos fricção para refund
```

### 3. Token Allowlist

```
✅ SEMPRE adicione tokens testados antes
✅ Remova tokens com problemas
✅ Documente cada token adicionado
✅ Considere adicionar stablecoins apenas
```

---

## Solução de Problemas

### Erro: "InvalidGuaranteeDays"

**Causa**: `guarantee_days < 1` ou `> 36500`

**Solução**:
```javascript
// ❌ Errado
guarantee_days: 0

// ✅ Correto
guarantee_days: 1  // Mínimo 1 dia!
```

---

### Erro: "GuaranteePeriodNotExpired"

**Causa**: Tentou liberar pagamento antes do período de garantia

**Soluções**:
1. Aguarde `guarantee_days` passarem
2. Use `allow_early_release = true` na criação
3. Inicie disputa se houver problema

---

### Erro: "InvalidNonce"

**Causa**: Nonce incorreto (ataque de replay detectado)

**Solução**:
```javascript
// 1. Obter nonce atual PRIMEIRO
const nonce = await contract.get_nonce({ user: myAddress });

// 2. Usar nonce correto na chamada
await contract.release_payment({
  escrow_id: 1,
  seller: myAddress,
  nonce: nonce  // ← Use o nonce atual!
});
```

---

### Erro: "BothPartiesMustAgree"

**Causa**: Partes propuseram resoluções diferentes

**Solução**:
1. Negocie fora da blockchain
2. Ambas chamam `propose_resolution` com MESMO valor
3. Ou use `admin_resolve_dispute`

---

### Erro: "TokenNotAllowed"

**Causa**: Token não está na allowlist

**Solução**:
```javascript
// Admin precisa adicionar primeiro:
await contract.add_allowed_token({
  token: newTokenAddress
});
```

---

## Métricas e Monitoramento

### KPIs Sugeridos

1. **Taxa de Reembolso**: `refunded / total_escrows`
2. **Taxa de Disputas**: `disputed / total_escrows`
3. **Taxa de Acordo**: `resolved_among_disputed / disputed`
4. **Tempo Médio de Resolução**: Tempo até dispute → resolved
5. **Early Release Usage**: `allow_early_release=true / total_escrows`

### Eventos para Monitorar

```javascript
// Monitorar todos os eventos
server.transactions()
  .forContract(contractId)
  .stream({
    onMessage: (tx) => {
      tx.operations.forEach(op => {
        if (op.type === 'invoke_contract_function') {
          // Parse events e armazenar em analytics
        }
      })
    }
  })
```

---

## Integração com Frontend

### Exemplo React Hook

```typescript
import { useState, useEffect } from 'react';
import { Contract } from './generated';

export function useEscrow(escrowId: number) {
  const [escrow, setEscrow] = useState<EscrowData | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchEscrow() {
      const contract = new Contract(contractId);
      const data = await contract.get_escrow({ escrow_id: escrowId });
      setEscrow(data);
      setLoading(false);
    }
    fetchEscrow();
  }, [escrowId]);

  const canRelease = escrow?.allow_early_release ||
    Date.now() >= escrow?.release_at * 1000;

  const canRefund = Date.now() < escrow?.release_at * 1000;

  return { escrow, loading, canRelease, canRefund };
}
```

---

## Deploy e Configuração

### Deploy na Testnet

```bash
# 1. Build
cd contracts/escrow
make build

# 2. Deploy
./deploy_testnet.sh

# 3. Salvar contract ID
export CHATCHECKOUT_CONTRACT_ID="CD..."
```

### Configuração Inicial

```javascript
// 1. Setup
await contract.__constructor({
  admin: "GADMIN...",
  collect_on_create: false
});

// 2. Adicionar tokens
await contract.add_allowed_token({
  token: "CC...USDC"
});

await contract.add_allowed_token({
  token: "CC...USDT"
});

// 3. Configurar frontend
const config = {
  contractId: "CD...",
  allowedTokens: ["CC...USDC", "CC...USDT"],
  defaultFeeBps: 400,
  defaultGuaranteeDays: 7
};
```

---

## Conclusão

Este contrato oferece um sistema completo de pagamento seguro com:

✅ **Proteção para compradores** através de período de garantia
✅ **Flexibilidade** com early release opcional
✅ **Resolução de disputas** com acordo mútuo ou arbitragem
✅ **Taxas configuráveis** coletadas na criação ou liberação
✅ **Token allowlist** para segurança
✅ **Replay protection** através de nonces

Para dúvidas ou suporte, consulte a equipe ChatCheckout ou abra uma issue no repositório.
