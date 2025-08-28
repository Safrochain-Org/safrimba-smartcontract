#!/bin/bash

# Demo Tontine Simulation Script
# Runs a single cycle to demonstrate the simulation functionality
# This is useful for testing before running the full infinite loop

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_success() {
    echo -e "${CYAN}[SUCCESS]${NC} $1"
}

print_header() {
    echo -e "${PURPLE}================================${NC}"
    echo -e "${PURPLE} $1${NC}"
    echo -e "${PURPLE}================================${NC}"
}

print_demo_header() {
    echo -e "${BLUE}🎯 DEMO MODE 🎯${NC}"
    echo -e "${BLUE}This script will run ONE complete tontine cycle${NC}"
    echo -e "${BLUE}Perfect for testing the simulation before running the full loop${NC}"
    echo ""
}

# Configuration
ADMIN_ADDRESS="addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w"
MEMBER2_ADDRESS="addr_safro1djg2upp4enfm57wxc3h4ary8mddn36js33484h"
MEMBER3_ADDRESS="addr_safro12yx0z58fu69xgdq4kr4s4qsqgjpl8hf8lunm4w"
ADMIN_KEY="test_admin"
MEMBER2_KEY="test_member2"
MEMBER3_KEY="test_member3"
NODE_URL="https://rpc.testnet.safrochain.com"
CHAIN_ID="safro-testnet-1"
GAS_PRICES="0.025usaf"

# Contract parameters (shorter for demo)
ROUND_FREQUENCY=3600   # 1 hour (for demo)
TIME_GUARDS=300        # 5 minutes (for demo)
LATE_PENALTY="100000"
PROTOCOL_FEES="50000"

# Function to generate random deposit amount
generate_random_deposit() {
    local min=1000000
    local max=10000000
    local range=$((max - min))
    local random=$((RANDOM % range))
    echo $((min + random))
}

# Function to wait for transaction confirmation
wait_for_tx() {
    local txhash=$1
    local max_attempts=30
    local attempt=1
    
    print_status "Waiting for transaction confirmation: $txhash"
    
    while [ $attempt -le $max_attempts ]; do
        if safrochaind query tx "$txhash" --node "$NODE_URL" --output json > /dev/null 2>&1; then
            print_success "Transaction confirmed after $attempt attempts"
            return 0
        fi
        
        print_status "Attempt $attempt/$max_attempts - Transaction not yet confirmed, waiting..."
        sleep 10
        ((attempt++))
    done
    
    print_error "Transaction not confirmed after $max_attempts attempts"
    return 1
}

# Function to deploy contract
deploy_contract() {
    print_header "Deploying New Tontine Contract (Demo)"
    
    print_status "Building contract..."
    cargo build --target wasm32-unknown-unknown --release
    
    print_status "Optimizing WASM..."
    wasm-opt target/wasm32-unknown-unknown/release/tontine_contract.wasm \
        -o target/wasm32-unknown-unknown/release/tontine_contract_optimized.wasm \
        -O4 --strip-debug --strip-producers
    
    print_status "Deploying contract to testnet..."
    local deploy_tx=$(safrochaind tx wasm store \
        target/wasm32-unknown-unknown/release/tontine_contract_optimized.wasm \
        --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    print_status "Deploy transaction hash: $deploy_tx"
    
    if wait_for_tx "$deploy_tx"; then
        print_status "Extracting CODE_ID from transaction..."
        
        # Wait a bit more for transaction to be fully processed
        sleep 10
        
        # Try multiple methods to extract CODE_ID
        local code_id=""
        
        # Method 1: Direct query with simpler jq
        code_id=$(safrochaind query tx "$deploy_tx" --node "$NODE_URL" --output json 2>/dev/null | \
            jq -r '.tx_response.logs[0].events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value' 2>/dev/null || echo "")
        
        # Method 2: If first method fails, try alternative approach
        if [ -z "$code_id" ] || [ "$code_id" = "null" ]; then
            print_status "Method 1 failed, trying alternative approach..."
            code_id=$(safrochaind query tx "$deploy_tx" --node "$NODE_URL" --output json 2>/dev/null | \
                jq -r '.tx_response.logs[0].events[0].attributes[] | select(.key == "code_id") | .value' 2>/dev/null || echo "")
        fi
        
        # Method 3: If still fails, try to get from list-contract-by-code
        if [ -z "$code_id" ] || [ "$code_id" = "null" ]; then
            print_status "Method 2 failed, trying to query latest code..."
            sleep 5
            # Get the latest deployed code
            code_id=$(safrochaind query wasm list-code --node "$NODE_URL" --output json 2>/dev/null | \
                jq -r '.code_infos | sort_by(.code_id) | last | .code_id' 2>/dev/null || echo "")
        fi
        
        # Verify we got a valid CODE_ID
        if [ -z "$code_id" ] || [ "$code_id" = "null" ] || ! [[ "$code_id" =~ ^[0-9]+$ ]]; then
            print_error "Failed to extract CODE_ID from transaction"
            print_status "Transaction hash: $deploy_tx"
            print_status "Trying to use helper script..."
            
            # Try to use the helper script
            if [ -f "scripts/extract_code_id.sh" ]; then
                print_status "Using helper script to extract CODE_ID..."
                if ./scripts/extract_code_id.sh "$deploy_tx"; then
                    code_id=$(cat .code_id)
                    if [ -n "$code_id" ] && [[ "$code_id" =~ ^[0-9]+$ ]]; then
                        print_success "Helper script successfully extracted CODE_ID: $code_id"
                    else
                        print_error "Helper script failed to extract valid CODE_ID"
                        return 1
                    fi
                else
                    print_error "Helper script failed"
                    return 1
                fi
            else
                print_error "Helper script not found. Please check the transaction manually and extract CODE_ID"
                return 1
            fi
        fi
        
        print_success "Contract deployed with CODE_ID: $code_id"
        echo "$code_id" > .code_id
        return 0
    else
        print_error "Failed to deploy contract"
        return 1
    fi
}

# Function to instantiate contract
instantiate_contract() {
    local code_id=$1
    
    print_header "Instantiating Tontine Contract (Demo)"
    
    print_status "Instantiating contract with CODE_ID: $code_id"
    print_status "Demo settings: 1 hour rounds, 5 minute time guards"
    
    local instantiate_tx=$(safrochaind tx wasm instantiate "$code_id" \
        "{\"admin\": \"$ADMIN_ADDRESS\", \"token_denom\": \"usaf\", \"contribution_amount\": \"5000000\", \"round_frequency\": $ROUND_FREQUENCY, \"time_guards\": $TIME_GUARDS, \"beneficiaries\": [\"$ADMIN_ADDRESS\", \"$MEMBER2_ADDRESS\", \"$MEMBER3_ADDRESS\"], \"late_penalty\": \"$LATE_PENALTY\", \"protocol_fees\": \"$PROTOCOL_FEES\", \"arbitrator\": \"$ADMIN_ADDRESS\"}" \
        --label "Tontine Contract - Demo $(date +%Y%m%d_%H%M%S)" \
        --admin "$ADMIN_ADDRESS" --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    print_status "Instantiate transaction hash: $instantiate_tx"
    
    if wait_for_tx "$instantiate_tx"; then
        sleep 5
        local contract_addr=$(safrochaind query wasm list-contract-by-code "$code_id" \
            --node "$NODE_URL" --output json | jq -r '.contracts[0]')
        print_success "Contract instantiated at: $contract_addr"
        echo "$contract_addr" > .contract_address
        return 0
    else
        print_error "Failed to instantiate contract"
        return 1
    fi
}

# Function to register members
register_members() {
    local contract_addr=$1
    
    print_header "Registering Tontine Members (Demo)"
    
    # Register admin
    print_status "Registering admin member..."
    local admin_tx=$(safrochaind tx wasm execute "$contract_addr" \
        "{\"register_member\": {\"address\": \"$ADMIN_ADDRESS\"}}" \
        --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$admin_tx"; then
        print_success "Admin registered successfully"
    else
        print_error "Failed to register admin"
        return 1
    fi
    
    # Register member 2
    print_status "Registering member 2..."
    local member2_tx=$(safrochaind tx wasm execute "$contract_addr" \
        "{\"register_member\": {\"address\": \"$MEMBER2_ADDRESS\"}}" \
        --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$member2_tx"; then
        print_success "Member 2 registered successfully"
    else
        print_error "Failed to register member 2"
        return 1
    fi
    
    # Register member 3
    print_status "Registering member 3..."
    local member3_tx=$(safrochaind tx wasm execute "$contract_addr" \
        "{\"register_member\": {\"address\": \"$MEMBER3_ADDRESS\"}}" \
        --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$member3_tx"; then
        print_success "Member 3 registered successfully"
    else
        print_error "Failed to register member 3"
        return 1
    fi
    
    print_success "All members registered successfully"
}

# Function to start tontine
start_tontine() {
    local contract_addr=$1
    
    print_header "Starting Tontine (Demo)"
    
    print_status "Starting tontine with 3 rounds..."
    local start_tx=$(safrochaind tx wasm execute "$contract_addr" \
        '{"start_tontine": {}}' \
        --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    print_status "Start tontine transaction hash: $start_tx"
    
    if wait_for_tx "$start_tx"; then
        print_success "Tontine started successfully"
        return 0
    else
        print_error "Failed to start tontine"
        return 1
    fi
}

# Function to execute a single round
execute_round() {
    local contract_addr=$1
    local round_num=$2
    
    print_header "Executing Round $round_num (Demo)"
    
    # Get current beneficiary
    local beneficiary=$(safrochaind query wasm contract-state smart "$contract_addr" \
        '{"get_current_beneficiary":{}}' --node "$NODE_URL" --output json | \
        jq -r '.data')
    
    print_status "Round $round_num - Current beneficiary: $beneficiary"
    
    # Member 1 (Admin) deposit
    local deposit1=$(generate_random_deposit)
    print_status "Member 1 depositing $deposit1 usaf..."
    local tx1=$(safrochaind tx wasm execute "$contract_addr" \
        '{"deposit_contribution": {}}' \
        --from "$ADMIN_KEY" --amount "${deposit1}usaf" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx1"; then
        print_success "Member 1 deposited $deposit1 usaf"
    else
        print_error "Member 1 deposit failed"
        return 1
    fi
    
    # Member 2 deposit
    local deposit2=$(generate_random_deposit)
    print_status "Member 2 depositing $deposit2 usaf..."
    local tx2=$(safrochaind tx wasm execute "$contract_addr" \
        '{"deposit_contribution": {}}' \
        --from "$MEMBER2_KEY" --amount "${deposit2}usaf" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx2"; then
        print_success "Member 2 deposited $deposit2 usaf"
    else
        print_error "Member 2 deposit failed"
        return 1
    fi
    
    # Member 3 deposit
    local deposit3=$(generate_random_deposit)
    print_status "Member 3 depositing $deposit3 usaf..."
    local tx3=$(safrochaind tx wasm execute "$contract_addr" \
        '{"deposit_contribution": {}}' \
        --from "$MEMBER3_KEY" --amount "${deposit3}usaf" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx3"; then
        print_success "Member 3 deposited $deposit3 usaf"
    else
        print_error "Member 3 deposit failed"
        return 1
    fi
    
    local total_deposit=$((deposit1 + deposit2 + deposit3))
    print_success "Round $round_num completed - Total deposits: $total_deposit usaf"
    
    # Wait for round deadline (simulate with shorter time for demo)
    print_status "Waiting for round $round_num to complete (demo: 2 minutes)..."
    sleep 120  # Wait 2 minutes for demo
    
    # Distribute to beneficiary
    print_status "Distributing round $round_num to beneficiary..."
    local distribute_tx=$(safrochaind tx wasm execute "$contract_addr" \
        '{"distribute_to_beneficiary": {}}' \
        --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$distribute_tx"; then
        print_success "Round $round_num distributed successfully"
    else
        print_error "Failed to distribute round $round_num"
        return 1
    fi
    
    return 0
}

# Function to execute full tontine cycle
execute_full_cycle() {
    local contract_addr=$1
    
    print_header "Executing Full Tontine Cycle (Demo)"
    
    # Execute 3 rounds
    for round in 1 2 3; do
        if ! execute_round "$contract_addr" "$round"; then
            print_error "Failed to execute round $round"
            return 1
        fi
        
        # Wait between rounds
        if [ $round -lt 3 ]; then
            print_status "Waiting between rounds (demo: 1 minute)..."
            sleep 60
        fi
    done
    
    print_success "Full tontine cycle completed successfully"
}

# Function to display cycle statistics
display_cycle_stats() {
    local contract_addr=$1
    
    print_header "Demo Cycle Statistics"
    
    # Get tontine state
    local state=$(safrochaind query wasm contract-state smart "$contract_addr" \
        '{"get_tontine_state":{}}' --node "$NODE_URL" --output json)
    
    local total_balance=$(echo "$state" | jq -r '.data.total_balance')
    local member_count=$(echo "$state" | jq -r '.data.member_count')
    local total_rounds=$(echo "$state" | jq -r '.data.total_rounds')
    
    print_status "Total Balance: $total_balance usaf"
    print_status "Member Count: $member_count"
    print_status "Total Rounds: $total_rounds"
    
    # Get statistics
    local stats=$(safrochaind query wasm contract-state smart "$contract_addr" \
        '{"get_statistics":{}}' --node "$NODE_URL" --output json)
    
    local total_contributions=$(echo "$stats" | jq -r '.data.total_contributions')
    local total_distributions=$(echo "$stats" | jq -r '.data.total_distributions')
    local total_fees=$(echo "$stats" | jq -r '.data.total_fees')
    
    print_status "Total Contributions: $total_contributions usaf"
    print_status "Total Distributions: $total_distributions usaf"
    print_status "Total Fees: $total_fees usaf"
}

# Function to close tontine
close_tontine() {
    local contract_addr=$1
    
    print_header "Closing Tontine (Demo)"
    
    print_status "Closing tontine early..."
    local close_tx=$(safrochaind tx wasm execute "$contract_addr" \
        '{"close_early": {"reason": "Demo cycle completed"}}' \
        --from "$ADMIN_KEY" --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes \
        --output json | jq -r '.txhash')
    
    if wait_for_tx "$close_tx"; then
        print_success "Tontine closed successfully"
        return 0
    else
        print_error "Failed to close tontine"
        return 1
    fi
}

# Main demo function
main() {
    print_demo_header
    
    print_header "Tontine Simulation Demo"
    print_status "This demo will run ONE complete tontine cycle"
    print_status "Demo settings: 1 hour rounds, 5 minute time guards"
    print_status "Total demo time: approximately 15-20 minutes"
    echo ""
    
    read -p "Press Enter to continue or Ctrl+C to cancel..."
    
    # Deploy contract once at the beginning
    print_header "Initial Contract Deployment (Demo)"
    print_status "Deploying contract once for this demo..."
    
    if ! deploy_contract; then
        print_error "Failed to deploy initial contract"
        exit 1
    fi
    
    local code_id=$(cat .code_id)
    print_success "Contract deployed with CODE_ID: $code_id"
    
    # Instantiate contract
    if ! instantiate_contract "$code_id"; then
        print_error "Failed to instantiate contract for demo"
        exit 1
    fi
    
    local contract_addr=$(cat .contract_address)
    
    # Register members
    if ! register_members "$contract_addr"; then
        print_error "Failed to register members for demo"
        exit 1
    fi
    
    # Start tontine
    if ! start_tontine "$contract_addr"; then
        print_error "Failed to start tontine for demo"
        exit 1
    fi
    
    # Execute full cycle
    if ! execute_full_cycle "$contract_addr"; then
        print_error "Failed to execute full demo cycle"
        exit 1
    fi
    
    # Display statistics
    display_cycle_stats "$contract_addr"
    
    # Close tontine
    if ! close_tontine "$contract_addr"; then
        print_error "Failed to close tontine for demo"
        exit 1
    fi
    
    print_success "Demo cycle completed successfully!"
    echo ""
    print_status "Demo Summary:"
    print_status "✅ Contract deployed and instantiated"
    print_status "✅ 3 members registered"
    print_status "✅ Tontine started with 3 rounds"
    print_status "✅ All rounds executed with random deposits"
    print_status "✅ Funds distributed to beneficiaries"
    print_status "✅ Tontine closed successfully"
    echo ""
    print_status "Ready to run the full simulation:"
    print_status "  ./scripts/tontine_simulation.sh"
    echo ""
    print_success "🎉 Demo completed successfully! 🎉"
}

# Error handling
trap 'print_error "Demo interrupted. Exiting..."; exit 1' INT TERM

# Check dependencies
if ! command -v jq &> /dev/null; then
    print_error "jq is required but not installed. Please install jq first."
    exit 1
fi

if ! command -v safrochaind &> /dev/null; then
    print_error "safrochaind is required but not installed. Please install safrochaind first."
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory (where Cargo.toml is located)"
    exit 1
fi

# Run main function
main "$@"
