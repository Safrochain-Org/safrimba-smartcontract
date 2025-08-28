# CosmWasm Tontine Smart Contract

A comprehensive smart contract implementation for managing tontines (rotating savings and credit associations) on the Cosmos blockchain using CosmWasm.

## Overview

A tontine is a financial arrangement where a group of people contribute to a common fund, and each member receives the entire fund in rotation. This smart contract provides a secure, automated, and transparent way to manage tontines with features like:

- **Member Management**: Registration, removal, and replacement of members
- **Round Management**: Automated rounds with configurable frequency and deadlines
- **Contribution Handling**: Support for both native tokens and CW20 tokens
- **Penalty System**: Late payment penalties and dispute resolution
- **Fee Management**: Protocol fees and administrative fee collection
- **Security Features**: Reentrancy protection, escrow mechanisms, and access controls

## Features

### Core Functionality

- **Member Registration**: Admin-controlled member management
- **Tontine Lifecycle**: Start, pause, resume, and close tontine operations
- **Round Management**: Automated round progression with beneficiary rotation
- **Contribution Processing**: Handle deposits from members with deadline enforcement
- **Distribution**: Automated fund distribution to beneficiaries
- **Penalty Management**: Late payment detection and penalty application

### Advanced Features

- **Multi-Token Support**: Native Cosmos tokens and CW20 tokens
- **Dispute Resolution**: Arbitration system for conflict resolution
- **Escrow Management**: Secure fund holding until distribution
- **Early Closure**: Quorum-based early termination options
- **Migration Support**: Contract upgrade capabilities
- **Comprehensive Queries**: Extensive state and historical data access

### Security Features

- **Reentrancy Protection**: Prevents recursive contract calls
- **Access Control**: Role-based permissions (admin, arbitrator, members)
- **Input Validation**: Comprehensive parameter validation
- **State Consistency**: Atomic operations and rollback protection
- **Event Logging**: Complete audit trail for all operations

## Architecture

### Contract Structure

```
src/
├── lib.rs          # Main entry points and module exports
├── contract.rs     # Core contract logic and orchestration
├── error.rs        # Custom error types and handling
├── msg.rs          # Message structures and types
├── state.rs        # State management and storage
├── execute.rs      # Execute message implementations
├── query.rs        # Query message implementations
└── receive.rs      # CW20 and native token handling
```

### Key Components

#### State Management

- **Config**: Immutable configuration parameters
- **TontineState**: Current contract status and round information
- **Members**: Member registry with status and balance tracking
- **Rounds**: Round-by-round state and deposit tracking
- **Penalties**: Outstanding penalty tracking
- **Disputes**: Active dispute management

#### Storage Patterns

- **Indexed Storage**: Efficient member and round queries
- **Atomic Operations**: Consistent state updates
- **Optimized Access**: Minimized storage reads/writes

## Usage

### Instantiation

```rust
let msg = InstantiateMsg {
    admin: "cosmos1...".to_string(),
    token_denom: "usaf".to_string(),
    contribution_amount: "1000000".to_string(), // 1 ATOM
    round_frequency: 86400, // 1 day in seconds
    beneficiaries: vec!["cosmos1...".to_string()],
    late_penalty: "50000".to_string(), // 0.05 ATOM
    protocol_fees: "10000".to_string(), // 0.01 ATOM
    arbitrator: "cosmos1...".to_string(),
    time_guards: 3600, // 1 hour in seconds
};
```

### Key Operations

#### Member Management

```rust
// Register a new member
ExecuteMsg::RegisterMember { address: "cosmos1...".to_string() }

// Replace a member
ExecuteMsg::ReplaceMember {
    old_address: "cosmos1...".to_string(),
    new_address: "cosmos1...".to_string()
}
```

#### Tontine Control

```rust
// Start the tontine
ExecuteMsg::StartTontine {}

// Pause operations
ExecuteMsg::PauseTontine {}

// Resume operations
ExecuteMsg::ResumeTontine {}
```

#### Round Operations

```rust
// Deposit contribution
ExecuteMsg::DepositContribution {}

// Distribute to beneficiary
ExecuteMsg::DistributeToBeneficiary {}
```

### Queries

#### Configuration

```rust
// Get full configuration
QueryMsg::GetConfig {}

// Get admin address
QueryMsg::GetAdmin {}
```

#### Member Information

```rust
// Get all members
QueryMsg::GetMembers {}

// Get specific member
QueryMsg::GetMember { address: "cosmos1...".to_string() }
```

#### Round Information

```rust
// Get current round
QueryMsg::GetCurrentRound {}

// Get round details
QueryMsg::GetRoundInfo { round: 1 }
```

## Development

### Prerequisites

- Rust 1.70+
- CosmWasm SDK 1.4.0+
- wasm-pack (for building)

### Building

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Generate schema
cargo schema
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_instantiate

# Run with output
cargo test -- --nocapture
```

## Security Considerations

### Access Control

- **Admin**: Full control over contract configuration and member management
- **Arbitrator**: Dispute resolution and emergency operations
- **Members**: Limited to contribution and penalty payment operations

### Reentrancy Protection

- Global reentrancy guard prevents recursive calls
- State updates are atomic and consistent

### Input Validation

- Comprehensive parameter validation
- Address format verification
- Amount range checking
- State consistency validation

### Escrow Mechanism

- Funds are held securely until distribution
- No direct withdrawal capabilities for members
- Admin-controlled fund release

## Deployment

### Network Compatibility

- Cosmos Hub (ATOM)
- Osmosis (OSMO)
- Any Cosmos SDK chain with CosmWasm support

### Deployment Steps

1. **Compile Contract**: Build the WASM binary
2. **Upload Code**: Upload to target network
3. **Instantiate**: Deploy with configuration parameters
4. **Verify**: Confirm contract functionality
5. **Register Members**: Add initial member set
6. **Start Tontine**: Begin operations

### Configuration Parameters

- **Token Denomination**: Native token or CW20 contract address
- **Contribution Amount**: Fixed amount per round per member
- **Round Frequency**: Time between rounds in seconds
- **Beneficiaries**: Ordered list of beneficiary addresses
- **Penalties**: Late payment penalty amounts
- **Fees**: Protocol and administrative fees
- **Time Guards**: Additional time buffers for operations

## Contributing

### Development Guidelines

- Follow Rust best practices and CosmWasm patterns
- Comprehensive error handling and validation
- Extensive testing coverage
- Clear documentation and comments
- Consistent code formatting

### Testing Strategy

- Unit tests for all functions
- Integration tests for workflows
- Edge case coverage
- Security scenario testing
- Performance benchmarking

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For questions, issues, or contributions:

- Open an issue on GitHub
- Submit a pull request
- Contact the development team

## Disclaimer

This software is provided "as is" without warranty. Users should conduct their own security audits and testing before using in production environments.
