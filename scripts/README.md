# Tontine Simulation Scripts

This directory contains professional scripts for testing and simulating the Tontine smart contract.

## 🚀 Tontine Simulation Script

### Overview

The `tontine_simulation.sh` script simulates a complete tontine lifecycle in an infinite loop, automatically:

- Deploying the contract once at the beginning (reused for all cycles)
- Registering 3 members for each cycle
- Starting tontines with 3 rounds
- Executing rounds with random deposits
- Distributing funds to beneficiaries
- Closing tontines and starting new cycles

### Features

- **3 Rounds per Day**: Each round lasts 8 hours (configurable)
- **Random Deposits**: Each member deposits between 1,000,000 and 10,000,000 usaf per round
- **Smart Contract Optimization**: Contract deployed once, reused for all cycles (saves time and gas)
- **Automatic Contract Recreation**: New contract instance for each cycle (using same CODE_ID)
- **Professional Logging**: Colored output with detailed status information
- **Error Handling**: Robust error handling and transaction confirmation
- **Configurable Parameters**: Easy customization via configuration file

### Prerequisites

- `safrochaind` CLI tool installed and configured
- `jq` JSON processor installed
- 3 wallet keys with sufficient usaf balance
- Rust toolchain for contract compilation

### Configuration

1. Copy `simulation_config.env` to `.env` in the project root
2. Modify the configuration values as needed:
   - Member addresses
   - Key names
   - Network parameters
   - Contract parameters
   - Simulation timing

### Usage

#### Basic Usage

```bash
# From project root directory
./scripts/tontine_simulation.sh
```

#### With Custom Configuration

```bash
# Load custom environment variables
source scripts/simulation_config.env
./scripts/tontine_simulation.sh
```

#### Background Execution

```bash
# Run in background
nohup ./scripts/tontine_simulation.sh > simulation.log 2>&1 &

# Monitor progress
tail -f simulation.log
```

### Configuration Parameters

#### Member Configuration

- `ADMIN_ADDRESS`: Admin member's Safrochain address
- `MEMBER2_ADDRESS`: Second member's address
- `MEMBER3_ADDRESS`: Third member's address
- `ADMIN_KEY`: Admin's key name in safrochaind
- `MEMBER2_KEY`: Member 2's key name
- `MEMBER3_KEY`: Member 3's key name

#### Network Configuration

- `NODE_URL`: Safrochain RPC endpoint
- `CHAIN_ID`: Network chain ID
- `GAS_PRICES`: Gas price for transactions

#### Contract Parameters

- `ROUND_FREQUENCY`: Duration of each round in seconds (default: 28800 = 8 hours)
- `TIME_GUARDS`: Time buffer for late contributions (default: 1800 = 30 minutes)
- `LATE_PENALTY`: Penalty amount for late contributions
- `PROTOCOL_FEES`: Protocol fee amount

#### Simulation Parameters

- `MIN_DEPOSIT`: Minimum deposit amount per round
- `MAX_DEPOSIT`: Maximum deposit amount per round
- `ROUND_WAIT_TIME`: Wait time between rounds (for simulation)
- `CYCLE_WAIT_TIME`: Wait time between complete cycles
- `TRANSACTION_TIMEOUT`: Maximum attempts to wait for transaction confirmation

### Simulation Flow

1. **Cycle Start**

   - Deploy new contract
   - Instantiate with parameters
   - Register 3 members
   - Start tontine

2. **Round Execution** (3 rounds per cycle)

   - Each member makes random deposit
   - Wait for round completion
   - Distribute to beneficiary
   - Move to next round

3. **Cycle Completion**
   - Display cycle statistics
   - Close tontine
   - Wait before next cycle
   - Repeat from step 1

### Output and Logging

The script provides detailed, colored output:

- 🟢 **Green**: Information messages
- 🟡 **Yellow**: Warning messages
- 🔴 **Red**: Error messages
- 🔵 **Blue**: Success messages
- 🟣 **Purple**: Section headers
- 🟠 **Cyan**: Success confirmations

### Monitoring and Control

#### View Current Status

```bash
# Check if script is running
ps aux | grep tontine_simulation

# View log file
tail -f simulation.log
```

#### Stop Simulation

```bash
# Find process ID
ps aux | grep tontine_simulation

# Stop gracefully
kill <process_id>

# Force stop if needed
kill -9 <process_id>
```

### Troubleshooting

#### Common Issues

1. **Insufficient Funds**: Ensure all wallets have enough usaf
2. **Network Issues**: Check RPC endpoint connectivity
3. **Transaction Failures**: Verify gas prices and network status
4. **Key Issues**: Ensure all keys are properly configured

#### Debug Mode

```bash
# Run with debug output
bash -x ./scripts/tontine_simulation.sh
```

### Performance Considerations

- **Transaction Timing**: Each cycle takes approximately 10-15 minutes
- **Gas Costs**: Budget for multiple transactions per cycle
- **Network Load**: Consider running during off-peak hours
- **Storage**: Each cycle creates new contract instances

### Security Notes

- **Testnet Only**: This script is designed for testnet use
- **Key Management**: Never use production keys in simulation
- **Fund Limits**: Set reasonable deposit limits for testing
- **Monitoring**: Always monitor simulation execution

### Customization Examples

#### Faster Simulation (1 hour rounds)

```bash
export ROUND_FREQUENCY=3600  # 1 hour
export ROUND_WAIT_TIME=30    # 30 seconds
./scripts/tontine_simulation.sh
```

#### Higher Deposit Range

```bash
export MIN_DEPOSIT=5000000   # 5M usaf
export MAX_DEPOSIT=20000000  # 20M usaf
./scripts/tontine_simulation.sh
```

#### Different Network

```bash
export NODE_URL="https://rpc.mainnet.safrochain.com"
export CHAIN_ID="safro-mainnet-1"
./scripts/tontine_simulation.sh
```

### Support

For issues or questions:

1. Check the log output for error messages
2. Verify all prerequisites are met
3. Ensure sufficient funds in all wallets
4. Check network connectivity and status
