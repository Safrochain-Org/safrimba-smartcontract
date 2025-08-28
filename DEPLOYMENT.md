# 🚀 Tontine Contract Deployment Guide

This guide will help you deploy the Tontine Smart Contract to the Safrochain testnet.

## 📋 Prerequisites

1. **safrochaind CLI** - Make sure you have safrochaind installed and configured
2. **jq** - JSON processor for parsing responses
3. **Account Setup** - You need an account with testnet USAF tokens
4. **Network Access** - Access to Safrochain testnet

## 🔧 Installation

### Install jq (if not already installed)

```bash
# On macOS
brew install jq

# On Ubuntu/Debian
sudo apt-get install jq

# On CentOS/RHEL
sudo yum install jq
```

### Check safrochaind installation

```bash
safrochaind version
```

## 🏗️ Building the Contract

1. **Build the WASM file:**

```bash
make testnet
```

2. **Verify the WASM file was created:**

```bash
ls -la target/wasm32-unknown-unknown/release/tontine_contract.wasm
```

## ⚙️ Configuration

1. **Update the deployment configuration:**

   - Edit `config/deploy.toml` with your actual addresses
   - Update admin, beneficiary, and arbitrator addresses
   - Adjust contribution amounts and time settings as needed

2. **Key configuration parameters:**
   - `admin`: Your admin address (safro1...)
   - `beneficiaries`: List of beneficiary addresses
   - `contribution_amount`: Amount in micro USF (1 USF = 1,000,000 micro USF)
   - `round_frequency`: Time between rounds in seconds
   - `time_guards`: Various deadline settings

## 🚀 Deployment

### Option 1: Use the automated script

```bash
# Make sure the script is executable
chmod +x scripts/deploy_tontine.sh

# Run the deployment script
./scripts/deploy_tontine.sh
```

### Option 2: Manual deployment

1. **Store the contract:**

```bash
safrochaind tx wasm store target/wasm32-unknown-unknown/release/tontine_contract.wasm \
    --from mycontractadmin \
    --keyring-backend file \
    --chain-id safro-testnet-1 \
    --node https://rpc.testnet.safrochain.com \
    --gas-prices 0.025usaf \
    --gas auto \
    --gas-adjustment 1.3 \
    --yes \
    --output json
```

2. **Get the Code ID** from the response and instantiate:

```bash
# Replace CODE_ID with the actual code ID from step 1
safrochaind tx wasm instantiate CODE_ID '{
    "admin": "safro1your_admin_address_here",
    "token_denom": "usaf",
    "contribution_amount": "1000000",
    "round_frequency": 604800,
    "beneficiaries": [
        "safro1beneficiary1_address_here",
        "safro1beneficiary2_address_here",
        "safro1beneficiary3_address_here"
    ],
    "late_penalty": "50000",
    "protocol_fees": "10000",
    "arbitrator": "safro1arbitrator_address_here",
    "time_guards": {
        "registration_deadline": 86400,
        "contribution_deadline": 3600,
        "distribution_delay": 1800
    }
}' \
    --from mycontractadmin \
    --keyring-backend file \
    --chain-id safro-testnet-1 \
    --node https://rpc.testnet.safrochain.com \
    --gas-prices 0.025usaf \
    --gas auto \
    --gas-adjustment 1.3 \
    --label "tontine-contract" \
    --yes \
    --output json
```

## 🧪 Testing the Contract

1. **Query configuration:**

```bash
safrochaind query wasm contract-state smart CONTRACT_ADDRESS '{"get_config": {}}' \
    --node https://rpc.testnet.safrochain.com \
    --output json
```

2. **Query tontine state:**

```bash
safrochaind query wasm contract-state smart CONTRACT_ADDRESS '{"get_tontine_state": {}}' \
    --node https://rpc.testnet.safrochain.com \
    --output json
```

3. **Query members:**

```bash
safrochaind query wasm contract-state smart CONTRACT_ADDRESS '{"get_members": {}}' \
    --node https://rpc.testnet.safrochain.com \
    --output json
```

## 📝 Post-Deployment Steps

1. **Register members:**

```bash
safrochaind tx wasm execute CONTRACT_ADDRESS '{"register_member": {"address": "safro1member_address_here"}}' \
    --from mycontractadmin \
    --chain-id safro-testnet-1 \
    --node https://rpc.testnet.safrochain.com \
    --gas-prices 0.025usaf \
    --yes
```

2. **Start the tontine:**

```bash
safrochaind tx wasm execute CONTRACT_ADDRESS '{"start_tontine": {}}' \
    --from mycontractadmin \
    --chain-id safro-testnet-1 \
    --node https://rpc.testnet.safrochain.com \
    --gas-prices 0.025usaf \
    --yes
```

3. **Make contributions:**

```bash
safrochaind tx wasm execute CONTRACT_ADDRESS '{"deposit_contribution": {}}' \
    --from member_address \
    --amount 1000000usaf \
    --chain-id safro-testnet-1 \
    --node https://rpc.testnet.safrochain.com \
    --gas-prices 0.025usaf \
    --yes
```

## 🔍 Troubleshooting

### Common Issues

1. **"reference-types not enabled" error:**

   - This was fixed by updating the Cargo configuration
   - Make sure you're using the latest build

2. **Memory size issues:**

   - The WASM memory settings have been optimized
   - Current settings: 2MB initial and max memory

3. **Gas estimation failures:**
   - Try increasing `--gas-adjustment` to 1.5 or higher
   - Check your account has sufficient USAF for gas fees

### Getting Help

- Check the contract logs: `safrochaind query wasm contract-state smart CONTRACT_ADDRESS '{"get_tontine_state": {}}'`
- Verify your account balance: `safrochaind query bank balances YOUR_ADDRESS --node https://rpc.testnet.safrochain.com`
- Check transaction status: `safrochaind query tx TX_HASH --node https://rpc.testnet.safrochain.com`

## 📊 Contract Information

- **Contract Type**: Tontine (Rotating Savings and Credit Association)
- **Token**: USAF (Safrochain's native token)
- **Features**: Member management, contribution tracking, automatic distributions, penalty system, dispute resolution
- **Security**: Reentrancy protection, access control, escrow logic

## 🎯 Next Steps

After successful deployment:

1. Test all contract functions
2. Register test members
3. Simulate a complete tontine cycle
4. Monitor gas usage and optimize if needed
5. Consider upgrading to mainnet when ready

---

**Happy Deploying! 🚀**
