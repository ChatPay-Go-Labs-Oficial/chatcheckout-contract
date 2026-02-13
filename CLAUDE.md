# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Soroban smart contract project for an escrow system on Stellar. The contract manages payments between buyers and sellers with configurable fees and a guarantee period.

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

### Contract Structure

The main contract is in `contracts/escrow/src/lib.rs` with the following key components:

**Data Storage Types** (in persistent storage):
- `Escrow(u64)` - Individual escrow records keyed by sequential ID
- `Counter` - Tracks the next available escrow ID
- `Config` - Contract-wide configuration (admin address, fee settings)

**Key Data Structures**:
- `Config` - Stores fee configuration (`fee_bps` as basis points, `flat_fee` in token units, `collect_on_create` flag)
- `EscrowData` - Contains buyer/seller addresses, amount, asset, timestamps, status, product_id, and a snapshot of fee settings used at creation
- `EscrowStatus` - Enum: Active, Released, Refunded, Disputed

**Fee Collection Logic**:
- Fees are calculated as: `fee = amount * fee_bps / 10000 + flat_fee`
- The `collect_on_create` config flag determines when fees are collected:
  - `true`: Fee is collected from buyer at escrow creation (transferred to admin)
  - `false`: Fee is collected from the escrowed amount at release time
- **Important**: Fee settings are snapshotted in each escrow at creation time, so changes to the global config don't affect existing escrows

**Key Functions**:
- `__constructor()` - One-time initialization, sets admin and fee config
- `create_escrow()` - Creates escrow, transfers funds from buyer to contract, optionally collects fee
- `release_payment()` - Releases funds to seller (after guarantee period expires or by seller), collects fee if not collected at creation
- `request_refund()` - Buyer can request full refund within guarantee period
- `dispute_escrow()` - Placeholder for future dispute resolution workflow

### Authorization Patterns

- `__constructor` and `update_config` require authorization from the current admin
- `create_escrow` requires buyer authorization (for token transfers)
- `request_refund` requires buyer authorization
- `release_payment` is callable by anyone after guarantee period expires, or by seller before expiry

### Storage

- **Instance storage**: Used for `Counter` and `Config`
- **Persistent storage**: Used for `Escrow` records (survives contract invocation)

### Cargo Workspace

The project uses a Cargo workspace with the main contract at `contracts/escrow/`. The workspace dependency is `soroban-sdk = "22.0.0"`.
