#!/bin/bash

# Quick Test Script for Tontine Simulation
# Tests basic functionality without running full cycles

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_header() {
    echo -e "${YELLOW}================================${NC}"
    echo -e "${YELLOW} $1${NC}"
    echo -e "${YELLOW}================================${NC}"
}

# Test configuration
print_header "Testing Tontine Simulation Setup"

# Check dependencies
print_status "Checking dependencies..."

if ! command -v jq &> /dev/null; then
    print_error "jq is required but not installed. Please install jq first."
    exit 1
fi

if ! command -v safrochaind &> /dev/null; then
    print_error "safrochaind is required but not installed. Please install safrochaind first."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    print_error "cargo is required but not installed. Please install Rust first."
    exit 1
fi

if ! command -v wasm-opt &> /dev/null; then
    print_warning "wasm-opt is not installed. WASM optimization will be skipped."
fi

print_success "All dependencies are available"

# Check project structure
print_status "Checking project structure..."

if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory (where Cargo.toml is located)"
    exit 1
fi

if [ ! -f "scripts/tontine_simulation.sh" ]; then
    print_error "Tontine simulation script not found"
    exit 1
fi

print_success "Project structure is correct"

# Check wallet balances
print_status "Checking wallet balances..."

# Function to check balance
check_balance() {
    local address=$1
    local key_name=$2
    
    print_status "Checking balance for $key_name ($address)..."
    
    local balance=$(safrochaind query bank balances "$address" \
        --node https://rpc.testnet.safrochain.com --output json 2>/dev/null | \
        jq -r '.balances[] | select(.denom == "usaf") | .amount' 2>/dev/null || echo "0")
    
    if [ "$balance" = "0" ] || [ -z "$balance" ]; then
        print_warning "$key_name has no usaf balance"
        return 1
    else
        print_success "$key_name balance: $balance usaf"
        return 0
    fi
}

# Check all wallet balances
ADMIN_ADDRESS="addr_safro1g30jss4zxz5ra7efux45etpcmw7vk36vhqm47w"
MEMBER2_ADDRESS="addr_safro1djg2upp4enfm57wxc3h4ary8mddn36js33484h"
MEMBER3_ADDRESS="addr_safro12yx0z58fu69xgdq4kr4s4qsqgjpl8hf8lunm4w"

check_balance "$ADMIN_ADDRESS" "Admin"
check_balance "$MEMBER2_ADDRESS" "Member 2"
check_balance "$MEMBER3_ADDRESS" "Member 3"

# Test contract compilation
print_status "Testing contract compilation..."

if cargo build --target wasm32-unknown-unknown --release; then
    print_success "Contract compiled successfully"
else
    print_error "Contract compilation failed"
    exit 1
fi

# Test WASM optimization (if available)
if command -v wasm-opt &> /dev/null; then
    print_status "Testing WASM optimization..."
    
    if wasm-opt target/wasm32-unknown-unknown/release/tontine_contract.wasm \
        -o target/wasm32-unknown-unknown/release/tontine_contract_test.wasm \
        -O4 --strip-debug --strip-producers; then
        print_success "WASM optimization successful"
        
        # Show file sizes
        original_size=$(ls -lh target/wasm32-unknown-unknown/release/tontine_contract.wasm | awk '{print $5}')
        optimized_size=$(ls -lh target/wasm32-unknown-unknown/release/tontine_contract_test.wasm | awk '{print $5}')
        
        print_status "Original size: $original_size"
        print_status "Optimized size: $optimized_size"
        
        # Clean up test file
        rm target/wasm32-unknown-unknown/release/tontine_contract_test.wasm
    else
        print_error "WASM optimization failed"
        exit 1
    fi
fi

# Test network connectivity
print_status "Testing network connectivity..."

if safrochaind query block --node https://rpc.testnet.safrochain.com --output json > /dev/null 2>&1; then
    print_success "Network connectivity confirmed"
else
    print_error "Network connectivity failed"
    exit 1
fi

# Test key availability
print_status "Testing key availability..."

check_key() {
    local key_name=$1
    
    if safrochaind keys show "$key_name" --output json > /dev/null 2>&1; then
        print_success "Key '$key_name' is available"
        return 0
    else
        print_error "Key '$key_name' is not available"
        return 1
    fi
}

check_key "test_admin"
check_key "test_member2"
check_key "test_member3"

# Test configuration
print_status "Testing configuration..."

if [ -f "scripts/simulation_config.env" ]; then
    print_success "Simulation configuration file exists"
    
    # Test loading configuration
    if source scripts/simulation_config.env 2>/dev/null; then
        print_success "Configuration loaded successfully"
        
        # Display key configuration values
        echo ""
        print_status "Configuration Summary:"
        echo "  Round Frequency: ${ROUND_FREQUENCY:-'Not set'} seconds ($((${ROUND_FREQUENCY:-0} / 3600)) hours)"
        echo "  Time Guards: ${TIME_GUARDS:-'Not set'} seconds ($((${TIME_GUARDS:-0} / 60)) minutes)"
        echo "  Min Deposit: ${MIN_DEPOSIT:-'Not set'} usaf"
        echo "  Max Deposit: ${MAX_DEPOSIT:-'Not set'} usaf"
        echo "  Node URL: ${NODE_URL:-'Not set'}"
        echo "  Chain ID: ${CHAIN_ID:-'Not set'}"
    else
        print_warning "Configuration file exists but could not be loaded"
    fi
else
    print_warning "Simulation configuration file not found"
fi

# Final summary
print_header "Test Results Summary"

echo ""
print_success "✅ All basic tests passed!"
echo ""
print_status "Your system is ready to run the tontine simulation."
echo ""
print_status "To start the simulation:"
echo "  ./scripts/tontine_simulation.sh"
echo ""
print_status "To run in background:"
echo "  nohup ./scripts/tontine_simulation.sh > simulation.log 2>&1 &"
echo ""
print_status "To monitor progress:"
echo "  tail -f simulation.log"
echo ""

print_success "🎉 Ready to simulate! 🎉"
