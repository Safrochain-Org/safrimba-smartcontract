#!/bin/bash

# Comprehensive Tontine Smart Contract Test Script
# Tests ALL functions and features systematically

set -e

# Source configuration
if [ -f "scripts/simulation_config.env" ]; then
    source scripts/simulation_config.env
else
    echo "Configuration file not found. Please run from project root."
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_success() {
    echo -e "${BLUE}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_header() {
    echo -e "${PURPLE}================================${NC}"
    echo -e "${PURPLE} $1${NC}"
    echo -e "${PURPLE}================================${NC}"
}

print_subheader() {
    echo -e "${CYAN}--- $1 ---${NC}"
}

# Global variables
CODE_ID=""
CONTRACT_ADDRESS=""
CYCLE_COUNT=0
MAX_CYCLES=3

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# Function to wait for transaction
wait_for_tx() {
    local tx_hash=$1
    local max_wait=60
    local wait_time=0
    
    print_status "Waiting for transaction: $tx_hash"
    
    while [ $wait_time -lt $max_wait ]; do
        if safrochaind query tx "$tx_hash" --node "$NODE_URL" --output json >/dev/null 2>&1; then
            print_success "Transaction confirmed: $tx_hash"
            return 0
        fi
        sleep 2
        wait_time=$((wait_time + 2))
        echo -n "."
    done
    
    print_error "Transaction timeout: $tx_hash"
    return 1
}

# Function to check wallet balance
check_balance() {
    local address=$1
    local balance=$(safrochaind query bank balances "$address" --node "$NODE_URL" --output json 2>/dev/null | \
        jq -r '.balances[] | select(.denom == "usaf") | .amount' 2>/dev/null || echo "0")
    echo "${balance:-0}"
}

# Function to deploy contract
deploy_contract() {
    print_subheader "Deploying Smart Contract"
    
    # Check if we already have a CODE_ID
    if [ -f ".code_id" ] && [ -s ".code_id" ]; then
        CODE_ID=$(cat .code_id)
        print_status "Using existing CODE_ID: $CODE_ID"
        return 0
    fi
    
    print_status "Building and optimizing contract..."
    
    # Build the contract
    if ! cargo build --target wasm32-unknown-unknown --release; then
        print_error "Contract build failed"
        return 1
    fi
    
    # Optimize WASM
    if command -v wasm-opt >/dev/null 2>&1; then
        print_status "Optimizing WASM..."
        wasm-opt -Os target/wasm32-unknown-unknown/release/tontine_contract.wasm -o target/wasm32-unknown-unknown/release/tontine_contract_optimized.wasm
        WASM_FILE="target/wasm32-unknown-unknown/release/tontine_contract_optimized.wasm"
    else
        WASM_FILE="target/wasm32-unknown-unknown/release/tontine_contract.wasm"
        print_warning "wasm-opt not found, using unoptimized WASM"
    fi
    
    print_status "Deploying contract..."
    local deploy_tx=$(safrochaind tx wasm store "$WASM_FILE" \
        --from test_admin \
        --gas auto \
        --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" \
        --chain-id "$CHAIN_ID" \
        --yes \
        --output json | jq -r '.txhash')
    
    if [ -z "$deploy_tx" ] || [ "$deploy_tx" = "null" ]; then
        print_error "Deployment failed - no transaction hash"
        return 1
    fi
    
    print_status "Deployment transaction: $deploy_tx"
    
    if wait_for_tx "$deploy_tx"; then
        print_status "Extracting CODE_ID..."
        sleep 5
        
        # Extract CODE_ID from transaction
        CODE_ID=$(safrochaind query tx "$deploy_tx" --node "$NODE_URL" --output json 2>/dev/null | \
            jq -r '.tx_response.events[] | select(.type == "store_code") | .attributes[] | select(.key == "code_id") | .value' 2>/dev/null || echo "")
        
        if [ -z "$CODE_ID" ] || [ "$CODE_ID" = "null" ]; then
            print_error "Failed to extract CODE_ID"
            return 1
        fi
        
        print_success "Contract deployed with CODE_ID: $CODE_ID"
        echo "$CODE_ID" > .code_id
        return 0
    else
        print_error "Deployment transaction failed"
        return 1
    fi
}

# Function to instantiate contract
instantiate_contract() {
    print_subheader "Instantiating Contract Instance"
    
    if [ -z "$CODE_ID" ]; then
        print_error "No CODE_ID available"
        return 1
    fi
    
    print_status "Instantiating contract with CODE_ID: $CODE_ID"
    
    local instantiate_tx=$(safrochaind tx wasm instantiate "$CODE_ID" "{
        \"admin\": \"$ADMIN_ADDRESS\",
        \"token_denom\": \"usaf\",
        \"contribution_amount\": \"5000000\",
        \"round_frequency\": $ROUND_FREQUENCY,
        \"time_guards\": $TIME_GUARDS,
        \"beneficiaries\": [
            \"$ADMIN_ADDRESS\",
            \"$MEMBER2_ADDRESS\",
            \"$MEMBER3_ADDRESS\"
        ],
        \"late_penalty\": \"$LATE_PENALTY\",
        \"protocol_fees\": \"$PROTOCOL_FEES\",
        \"arbitrator\": \"$ADMIN_ADDRESS\"
    }" \
        --label "Tontine Contract - Cycle $((CYCLE_COUNT + 1))" \
        --admin "$ADMIN_ADDRESS" \
        --from test_admin \
        --gas auto \
        --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" \
        --chain-id "$CHAIN_ID" \
        --yes \
        --output json | jq -r '.txhash')
    
    if [ -z "$instantiate_tx" ] || [ "$instantiate_tx" = "null" ]; then
        print_error "Instantiation failed"
        return 1
    fi
    
    print_status "Instantiation transaction: $instantiate_tx"
    
    if wait_for_tx "$instantiate_tx"; then
        sleep 5
        
        # Get contract address
        CONTRACT_ADDRESS=$(safrochaind query wasm list-contract-by-code "$CODE_ID" --node "$NODE_URL" --output json | \
            jq -r '.contracts | last')
        
        if [ -z "$CONTRACT_ADDRESS" ] || [ "$CONTRACT_ADDRESS" = "null" ]; then
            print_error "Failed to get contract address"
            return 1
        fi
        
        print_success "Contract instantiated at: $CONTRACT_ADDRESS"
        echo "$CONTRACT_ADDRESS" > .contract_address
        return 0
    else
        print_error "Instantiation transaction failed"
        return 1
    fi
}

# Function to test member registration
test_member_registration() {
    print_subheader "Testing Member Registration"
    
    local test_name="Member Registration"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Register admin
    print_status "Registering admin..."
    local tx1=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"register_member\": {
            \"address\": \"$ADMIN_ADDRESS\"
        }
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx1"; then
        print_success "Admin registered successfully"
    else
        print_error "Admin registration failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Register member 2
    print_status "Registering member 2..."
    local tx2=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"register_member\": {
            \"address\": \"$MEMBER2_ADDRESS\"
        }
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx2"; then
        print_success "Member 2 registered successfully"
    else
        print_error "Member 2 registration failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Register member 3
    print_status "Registering member 3..."
    local tx3=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"register_member\": {
            \"address\": \"$MEMBER3_ADDRESS\"
        }
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx3"; then
        print_success "Member 3 registered successfully"
    else
        print_error "Member 3 registration failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Test duplicate registration (should fail)
    print_status "Testing duplicate registration (should fail)..."
    local tx4=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"register_member\": {
            \"address\": \"$ADMIN_ADDRESS\"
        }
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json 2>&1 || echo "DUPLICATE_ERROR")
    
    if echo "$tx4" | grep -q "DUPLICATE_ERROR\|already registered"; then
        print_success "Duplicate registration correctly rejected"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_error "Duplicate registration should have failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    print_success "$test_name: PASSED"
    return 0
}

# Function to test tontine start
test_tontine_start() {
    print_subheader "Testing Tontine Start"
    
    local test_name="Tontine Start"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    print_status "Starting tontine..."
    local tx=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"start_tontine\": {}
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx"; then
        print_success "Tontine started successfully"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "Tontine start failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to test contribution deposits
test_contribution_deposits() {
    print_subheader "Testing Contribution Deposits"
    
    local test_name="Contribution Deposits"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Admin deposit
    print_status "Admin making contribution..."
    local tx1=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"deposit_contribution\": {}
    }" --from test_admin --amount 5000000usaf --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx1"; then
        print_success "Admin contribution successful"
    else
        print_error "Admin contribution failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Member 2 deposit
    print_status "Member 2 making contribution..."
    local tx2=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"deposit_contribution\": {}
    }" --from test_member2 --amount 5000000usaf --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx2"; then
        print_success "Member 2 contribution successful"
    else
        print_error "Member 2 contribution failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Member 3 deposit
    print_status "Member 3 making contribution..."
    local tx3=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"deposit_contribution\": {}
    }" --from test_member3 --amount 5000000usaf --gas auto --gas-adjustment 1.3 \
        --gas-prices "$GAS_PRICES" --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx3"; then
        print_success "Member 3 contribution successful"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "Member 3 contribution failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to test all query functions
test_all_queries() {
    print_subheader "Testing All Query Functions"
    
    local test_name="Query Functions"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    print_status "Testing get_tontine_state..."
    local state=$(safrochaind query wasm contract-state smart "$CONTRACT_ADDRESS" '{"get_tontine_state": {}}' \
        --node "$NODE_URL" --output json 2>/dev/null || echo "QUERY_FAILED")
    
    if [ "$state" != "QUERY_FAILED" ]; then
        print_success "get_tontine_state working"
    else
        print_error "get_tontine_state failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    print_status "Testing get_member_count..."
    local member_count=$(safrochaind query wasm contract-state smart "$CONTRACT_ADDRESS" '{"get_member_count": {}}' \
        --node "$NODE_URL" --output json 2>/dev/null || echo "QUERY_FAILED")
    
    if [ "$member_count" != "QUERY_FAILED" ]; then
        print_success "get_member_count working"
    else
        print_error "get_member_count failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    print_status "Testing get_members..."
    local members=$(safrochaind query wasm contract-state smart "$CONTRACT_ADDRESS" '{"get_members": {}}' \
        --node "$NODE_URL" --output json 2>/dev/null || echo "QUERY_FAILED")
    
    if [ "$members" != "QUERY_FAILED" ]; then
        print_success "get_members working"
    else
        print_error "get_members failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    print_status "Testing get_statistics..."
    local stats=$(safrochaind query wasm contract-state smart "$CONTRACT_ADDRESS" '{"get_statistics": {}}' \
        --node "$NODE_URL" --output json 2>/dev/null || echo "QUERY_FAILED")
    
    if [ "$stats" != "QUERY_FAILED" ]; then
        print_success "get_statistics working"
    else
        print_error "get_statistics failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    print_status "Testing get_round_info..."
    local round_info=$(safrochaind query wasm contract-state smart "$CONTRACT_ADDRESS" '{"get_round_info": {"round_number": 1}}' \
        --node "$NODE_URL" --output json 2>/dev/null || echo "QUERY_FAILED")
    
    if [ "$round_info" != "QUERY_FAILED" ]; then
        print_success "get_round_info working"
    else
        print_error "get_round_info failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    print_status "Testing get_round_deposits..."
    local deposits=$(safrochaind query wasm contract-state smart "$CONTRACT_ADDRESS" '{"get_round_deposits": {"round_number": 1}}' \
        --node "$NODE_URL" --output json 2>/dev/null || echo "QUERY_FAILED")
    
    if [ "$deposits" != "QUERY_FAILED" ]; then
        print_success "get_round_deposits working"
    else
        print_error "get_round_deposits failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    print_success "$test_name: PASSED"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    return 0
}

# Function to test round execution
test_round_execution() {
    print_subheader "Testing Round Execution"
    
    local test_name="Round Execution"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Wait for round to be ready
    print_status "Waiting for round to be ready..."
    sleep 10
    
    print_status "Executing round..."
    local tx=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"execute_round\": {}
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx"; then
        print_success "Round executed successfully"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "Round execution failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to test distribution
test_distribution() {
    print_subheader "Testing Distribution"
    
    local test_name="Distribution"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    print_status "Distributing to beneficiary..."
    local tx=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"distribute_beneficiary\": {
            \"round_number\": 1,
            \"beneficiary\": \"$ADMIN_ADDRESS\"
        }
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx"; then
        print_success "Distribution successful"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "Distribution failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to test admin operations
test_admin_operations() {
    print_subheader "Testing Admin Operations"
    
    local test_name="Admin Operations"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Test update admin
    print_status "Testing update admin..."
    local tx1=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"update_admin\": {
            \"new_admin\": \"$MEMBER2_ADDRESS\"
        }
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx1"; then
        print_success "Admin updated successfully"
    else
        print_error "Admin update failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
    
    # Test change back to original admin
    print_status "Changing admin back to original..."
    local tx2=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"update_admin\": {
            \"new_admin\": \"$ADMIN_ADDRESS\"
        }
    }" --from test_member2 --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx2"; then
        print_success "Admin changed back successfully"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "Admin change back failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to test contract closure
test_contract_closure() {
    print_subheader "Testing Contract Closure"
    
    local test_name="Contract Closure"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    print_status "Closing contract early..."
    local tx=$(safrochaind tx wasm execute "$CONTRACT_ADDRESS" "{
        \"close_early\": {
            \"reason\": \"Test cycle completed\"
        }
    }" --from test_admin --gas auto --gas-adjustment 1.3 --gas-prices "$GAS_PRICES" \
        --node "$NODE_URL" --chain-id "$CHAIN_ID" --yes --output json | jq -r '.txhash')
    
    if wait_for_tx "$tx"; then
        print_success "Contract closed successfully"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        print_error "Contract closure failed"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

# Function to run one complete cycle
run_cycle() {
    print_header "Starting Cycle $((CYCLE_COUNT + 1))"
    
    if ! instantiate_contract; then
        return 1
    fi
    
    if ! test_member_registration; then
        return 1
    fi
    
    if ! test_tontine_start; then
        return 1
    fi
    
    if ! test_contribution_deposits; then
        return 1
    fi
    
    if ! test_all_queries; then
        return 1
    fi
    
    if ! test_round_execution; then
        return 1
    fi
    
    if ! test_distribution; then
        return 1
    fi
    
    if ! test_admin_operations; then
        return 1
    fi
    
    if ! test_contract_closure; then
        return 1
    fi
    
    print_success "Cycle $((CYCLE_COUNT + 1)) completed successfully!"
    CYCLE_COUNT=$((CYCLE_COUNT + 1))
    return 0
}

# Function to display test results
display_results() {
    print_header "Test Results Summary"
    
    echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
    echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
    echo -e "${BLUE}Total Tests: $TOTAL_TESTS${NC}"
    echo -e "${YELLOW}Cycles Completed: $CYCLE_COUNT${NC}"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}🎉 ALL TESTS PASSED! 🎉${NC}"
        return 0
    else
        echo -e "${RED}❌ Some tests failed. Please review the errors above.❌${NC}"
        return 1
    fi
}

# Main function
main() {
    print_header "Comprehensive Tontine Smart Contract Test"
    
    print_status "Checking prerequisites..."
    
    # Check if safrochaind is available
    if ! command -v safrochaind >/dev/null 2>&1; then
        print_error "safrochaind not found. Please install it first."
        exit 1
    fi
    
    # Check if jq is available
    if ! command -v jq >/dev/null 2>&1; then
        print_error "jq not found. Please install it first."
        exit 1
    fi
    
    # Check if cargo is available
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "cargo not found. Please install it first."
        exit 1
    fi
    
    print_success "All prerequisites met"
    
    # Deploy contract once
    if ! deploy_contract; then
        print_error "Contract deployment failed. Exiting."
        exit 1
    fi
    
    # Run test cycles
    while [ $CYCLE_COUNT -lt $MAX_CYCLES ]; do
        if ! run_cycle; then
            print_error "Cycle $((CYCLE_COUNT + 1)) failed. Stopping."
            break
        fi
        
        if [ $CYCLE_COUNT -lt $MAX_CYCLES ]; then
            print_status "Waiting before next cycle..."
            sleep 5
        fi
    done
    
    # Display final results
    display_results
}

# Run main function
main "$@"
