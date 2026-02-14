# ChatCheckout Escrow - Soroban Smart Contract

A decentralized escrow system built on Stellar using Soroban smart contracts. This contract facilitates secure transactions between buyers and sellers with configurable fees, guarantee periods, and a comprehensive dispute resolution system.

## Features

- **Secure Escrow Management**: Create and manage escrow transactions with configurable fee collection
- **Flexible Fee Configuration**: Set percentage-based fees (in basis points) and/or flat fees
- **Configurable Fee Collection**: Choose to collect fees on escrow creation or on release
- **Guarantee Period**: Buyer protection with a time-limited refund window
- **Dispute Resolution System**: Complete workflow for dispute initiation, resolution proposals, and final agreement
- **Event System**: Track all contract state changes through emitted events
- **Admin Controls**: Contract administrator can update fee configurations
- **TypeScript SDK**: Auto-generated client SDK for easy frontend integration

## Project Structure

```text
.
├── .agents/                          # Claude Code agent skills for Stellar development
├── .claude/                          # Claude Code configuration
├── contracts/
│   └── escrow/                        # Main escrow smart contract
│       ├── src/
│       │   ├── error.rs               # Custom error types
│       │   ├── events.rs              # Event definitions
│       │   ├── lib.rs                 # Main contract logic
│       │   ├── math.rs                # Fee calculations
│       │   ├── storage.rs             # Data structures and storage keys
│       │   ├── test.rs                # Unit tests
│       │   └── validation.rs         # Input validation
│       ├── Cargo.toml                 # Contract dependencies
│       ├── Makefile                   # Build automation
│       └── test_snapshots/           # Test snapshots for regression testing
├── generated/                         # TypeScript client SDK
│   ├── src/
│   │   └── index.ts                  # Auto-generated SDK
│   ├── package.json
│   └── tsconfig.json
├── deploy_testnet.sh                  # Testnet deployment script
├── Cargo.toml                        # Workspace configuration
├── CLAUDE.md                         # Project instructions for Claude Code
└── README.md                         # This file
```

## Build and Development

### Prerequisites

- Rust toolchain (latest stable)
- Soroban CLI: `cargo install stellar-cli`
- Stellar Testnet account (for deployment)

### Building the Contract

```bash
# From contracts/escrow directory
make build
# or
stellar contract build
```

The compiled `.wasm` file is output to `target/wasm32v1-none/release/`.

### Running Tests

```bash
# From contracts/escrow directory
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

## Contract Architecture

### Data Storage

**Persistent Storage** (survives contract invocations):
- `Escrow(u64)` - Individual escrow records keyed by sequential ID
  - Buyer/Seller addresses
  - Amount and asset information
  - Timestamps (created_at, expires_at)
  - Status (Active, Released, Refunded, Disputed)
  - Product ID
  - Fee snapshot (settings at creation time)

**Instance Storage** (temporary):
- `Counter` - Tracks the next available escrow ID
- `Config` - Contract-wide configuration
  - Admin address
  - Fee settings (fee_bps, flat_fee, collect_on_create)

### Key Functions

| Function | Description | Authorization |
|----------|-------------|----------------|
| `__constructor()` | Initialize contract with admin and fee config | Admin |
| `create_escrow()` | Create new escrow, transfer funds from buyer | Buyer |
| `release_payment()` | Release funds to seller, collect fee if needed | Anyone (after guarantee period) or Seller |
| `request_refund()` | Buyer requests full refund within guarantee period | Buyer |
| `dispute_escrow()` | Initiate dispute on active escrow | Buyer or Seller |
| `propose_resolution()` | Propose resolution for disputed escrow | Buyer or Seller |
| `resolve_dispute()` | Finalize dispute with both parties' agreement | Buyer or Seller |
| `update_config()` | Update fee configuration | Admin |
| `get_escrow()` | Retrieve escrow details by ID | Public |
| `get_seller_escrows()` | Get all escrows for a seller | Public |

### Fee Calculation

```
fee = (amount × fee_bps / 10000) + flat_fee
```

**Fee Collection Timing** (configurable via `collect_on_create`):
- `true`: Fee collected from buyer at escrow creation (transferred to admin)
- `false`: Fee collected from escrowed amount at release time

**Important**: Fee settings are snapshotted in each escrow at creation time. Changes to global config don't affect existing escrows.

### Escrow Status Flow

```
[Active] → [Released]   (normal completion)
    ↓
[Disputed] → [Released]   (after resolution agreement)
    ↓
[Refunded]                 (buyer refund within guarantee period)
```

### Dispute Resolution Workflow

1. **Initiate Dispute**: Buyer or seller calls `dispute_escrow()`
2. **Propose Resolution**: Each party calls `propose_resolution()` with their preferred outcome
3. **Resolve**: `resolve_dispute()` is called when both parties have voted
   - If both agree to favor buyer: Buyer receives refund (minus fee if collected on release)
   - If both agree to favor seller: Seller receives payment (minus fee)

## Deployment

### Deploy to Testnet

```bash
# From project root
./deploy_testnet.sh
```

The script will:
1. Build the contract
2. Deploy to Stellar Testnet
3. Return the contract ID for use in frontend applications

### Using the Generated SDK

The project includes an auto-generated TypeScript SDK in the `generated/` directory for easy frontend integration:

```typescript
import { Contract } from './generated';

// Initialize with contract ID
const contract = new Contract(contractId);

// Create escrow
await contract.create_escrow({
  seller: sellerAddress,
  amount: BigInt(1000000),
  asset: assetAddress,
  product_id: "product_123",
  guarantee_period: 604800 // 7 days in seconds
});
```

See `generated/README.md` for complete API documentation.

## Testing

The contract includes comprehensive test coverage including:
- Basic escrow creation and flow
- Fee collection on creation vs release
- Refund within guarantee period
- Dispute initiation and resolution
- Edge cases and error conditions

Run tests with snapshot validation:
```bash
cargo test -- --nocapture
```

## Security Considerations

- All token transfers require proper authorization
- Admin-only functions are protected with signature verification
- Fee snapshots prevent retroactive fee changes on existing escrows
- Dispute resolution requires mutual agreement between parties
- Guarantee period provides buyer protection with clear expiration

## License

This project is part of the ChatCheckout ecosystem.

## Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar SDK for JavaScript](https://github.com/stellar/js-stellar-sdk)
- [Claude Code](https://claude.com/claude-code)
