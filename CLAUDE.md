# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Soroban smart contract project for an escrow system on Stellar. The contract manages payments between buyers and sellers with configurable fees, a guarantee period, and a complete dispute resolution system.

## Build and Development Commands

### Building the Contract
```bash
# Build the WebAssembly contract (from contracts/escrow directory)
make build
# or
stellar contract build
```

The compiled `.wasm` file is output to `target/wasm32v1-none/release/`.

### Running Tests
```bash
# Run all tests (from contracts/escrow directory)
make test
# or
cargo test
```

### Running a Single Test
```bash
cargo test test_name
```

### Formatting Code
```bash
make fmt
# or
cargo fmt --all
```

### Cleaning Build Artifacts
```bash
make clean
# or
cargo clean
```

## Architecture

### Module Structure

The contract is organized into modules in `contracts/escrow/src/`:
- **lib.rs** - Main contract logic and public functions
- **storage.rs** - Data structures and storage access functions
- **error.rs** - Custom error types (EscrowError enum)
- **events.rs** - Event definitions for tracking state changes
- **validation.rs** - Input validation and business rule checks
- **signature.rs** - Nonce management for replay attack protection
- **math.rs** - Fee calculations
- **test.rs** - Unit tests

### Data Storage Types

**Persistent Storage** (survives contract invocations):
- `Escrow(u64)` - Individual escrow records keyed by sequential ID
- `Nonce(Address)` - User nonces for replay protection

**Instance Storage** (temporary, contract-wide):
- `Counter` - Tracks the next available escrow ID
- `Config` - Contract-wide configuration (admin address, collect_on_create flag)
- `AllowedTokens` - List of tokens approved for use in escrows

### Key Data Structures

**Config**:
- `admin` - Admin address (can update config and manage token allowlist)
- `collect_on_create` - When to collect fees: true = at creation, false = at release

**EscrowData**:
- `buyer` / `seller` - Participant addresses
- `amount` - Payment amount (in token units)
- `asset` - Token address
- `created_at` / `release_at` - Timestamps
- `status` - Active, Released, Refunded, or Disputed
- `product_id` - Product/venda identifier
- `guarantee_days` - Length of guarantee period
- `fee_bps` - Fee in basis points (snapshotted at creation)
- `allow_early_release` - Whether seller can release before guarantee period expires
- `buyer_proposal` / `seller_proposal` - Dispute resolution votes (Option<bool>)

**EscrowStatus**: Active, Released, Refunded, Disputed

### Fee Collection Logic

Fees are calculated as: `fee = amount * fee_bps / 10000`

The `fee_bps` parameter is provided by the backend when creating escrows and is snapshotted per-escrow. The global `collect_on_create` config determines when fees are collected:
- `true`: Fee is collected from buyer at escrow creation (transferred to admin)
- `false`: Fee is collected from the escrowed amount at release time

**Important**: Fee settings (fee_bps) are set per-escrow at creation time from backend parameters, not from global config.

### Token Allowlist

The admin controls which tokens can be used in escrows:
- `add_allowed_token(token)` - Add token to allowlist (admin only)
- `remove_allowed_token(token)` - Remove token from allowlist (admin only)
- `get_allowed_tokens()` - Get list of allowed tokens (public)
- `is_token_allowed(token)` - Check if token is allowed (public)

Note: Removing a token doesn't affect existing escrows using that token.

### Nonce System (Replay Protection)

Each user has a nonce counter for replay protection in meta-transactions:
- `get_nonce(user)` - Get current nonce for a user
- Nonce is verified and incremented on state-changing operations
- Used in: `release_payment`, `request_refund`, `dispute_escrow`, `propose_resolution`, `resolve_dispute`, `admin_resolve_dispute`

### Key Functions

**Initialization & Config**:
- `__constructor(env, admin, collect_on_create)` - One-time initialization, requires admin auth
- `update_config(env, new_admin, collect_on_create)` - Update config (admin only)
- `get_config(env)` - Get current config (public)

**Token Allowlist** (admin only):
- `add_allowed_token(env, token)` - Add token to allowlist
- `remove_allowed_token(env, token)` - Remove token from allowlist
- `get_allowed_tokens(env)` - Get list of allowed tokens
- `is_token_allowed(env, token)` - Check if token is allowed

**Escrow Operations**:
- `create_escrow(env, buyer, seller, amount, asset, fee_bps, guarantee_days, product_id, allow_early_release)` - Create escrow (buyer auth required)
- `get_escrow(env, escrow_id)` - Get escrow details (public)
- `release_payment(env, escrow_id, seller, nonce)` - Release to seller after guarantee period (or early if allowed) - requires seller auth + nonce
- `request_refund(env, escrow_id, buyer, nonce)` - Request refund within guarantee period - requires buyer auth + nonce

**Dispute Resolution**:
- `dispute_escrow(env, escrow_id, caller, nonce)` - Initiate dispute (buyer or seller)
- `propose_resolution(env, escrow_id, caller, nonce, favor_buyer)` - Propose resolution outcome (buyer or seller)
- `resolve_dispute(env, escrow_id, caller, nonce)` - Finalize dispute when both parties agree (buyer or seller)
- `admin_resolve_dispute(env, escrow_id, favor_buyer, nonce)` - Admin resolves when parties can't agree (admin only)

### Authorization Patterns

- Admin functions: `__constructor`, `update_config`, `add_allowed_token`, `remove_allowed_token`, `admin_resolve_dispute`
- Buyer-only: `create_escrow`, `request_refund`
- Seller-only: `release_payment`
- Buyer or Seller: `dispute_escrow`, `propose_resolution`, `resolve_dispute`
- Public: `get_config`, `get_escrow`, `get_allowed_tokens`, `is_token_allowed`, `get_nonce`

### Dispute Resolution Workflow

1. Either party calls `dispute_escrow()` - status changes to Disputed
2. Each party calls `propose_resolution(favor_buyer)` - true = refund to buyer, false = release to seller
3. When both have proposed and agree, either can call `resolve_dispute()` to execute
4. If parties disagree, admin can call `admin_resolve_dispute(favor_buyer)` to force resolution

Fee is deducted from refunded/released amount if `collect_on_create` is false.

### Event System

All state changes emit events for tracking:
- `CreateEscrowEvent` - New escrow created
- `ReleasePaymentEvent` - Payment released to seller
- `RequestRefundEvent` - Refund issued to buyer
- `TokenAddedEvent` / `TokenRemovedEvent` - Allowlist changes
- `DisputeEscrowEvent` - Dispute initiated
- `ProposeResolutionEvent` - Resolution proposed
- `ResolveDisputeEvent` - Dispute resolved

### Cargo Workspace

The project uses a Cargo workspace with the main contract at `contracts/escrow/`. The workspace dependency is `soroban-sdk = "25.2.0"`.
