# CosmWasm Tontine Contract Makefile

# Variables
CARGO = cargo
WASM_PACK = wasm-pack
SCHEMA_DIR = schema
TARGET_DIR = target
WASM_TARGET = wasm32-unknown-unknown
CONTRACT_NAME = tontine-contract

# Default target
.PHONY: all
all: build

# Build the contract
.PHONY: build
build:
	@echo "Building contract..."
	RUSTFLAGS="-C target-feature=+crt-static" $(CARGO) build --target $(WASM_TARGET) --release
	@echo "Build complete!"

# Build optimized WASM
.PHONY: wasm
wasm: build
	@echo "Generating optimized WASM..."
	@mkdir -p artifacts
	cp $(TARGET_DIR)/$(WASM_TARGET)/release/$(CONTRACT_NAME).wasm artifacts/
	@echo "WASM file generated in artifacts/"

# Run tests
.PHONY: test
test:
	@echo "Running tests..."
	$(CARGO) test
	@echo "Tests complete!"

# Run tests with output
.PHONY: test-verbose
test-verbose:
	@echo "Running tests with output..."
	$(CARGO) test -- --nocapture
	@echo "Tests complete!"

# Generate schema
.PHONY: schema
schema:
	@echo "Generating JSON schema..."
	$(CARGO) build --target $(WASM_TARGET) --release
	@echo "Schema generation complete!"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf artifacts/
	rm -rf $(SCHEMA_DIR)/*.json
	@echo "Clean complete!"

# Format code
.PHONY: fmt
fmt:
	@echo "Formatting code..."
	$(CARGO) fmt
	@echo "Formatting complete!"

# Check code style
.PHONY: check
check:
	@echo "Checking code style..."
	$(CARGO) fmt -- --check
	@echo "Code style check complete!"

# Clippy linting
.PHONY: clippy
clippy:
	@echo "Running clippy..."
	$(CARGO) clippy --target $(WASM_TARGET) -- -D warnings
	@echo "Clippy complete!"

# Security audit
.PHONY: audit
audit:
	@echo "Running security audit..."
	$(CARGO) audit
	@echo "Security audit complete!"

# Check dependencies
.PHONY: outdated
outdated:
	@echo "Checking for outdated dependencies..."
	$(CARGO) outdated
	@echo "Dependency check complete!"

# Install development dependencies
.PHONY: install-deps
install-deps:
	@echo "Installing development dependencies..."
	rustup target add $(WASM_TARGET)
	@echo "Dependencies installed!"

# Development setup
.PHONY: setup
setup: install-deps
	@echo "Development setup complete!"

# Full development cycle
.PHONY: dev
dev: clean build test clippy
	@echo "Development cycle complete!"

# Testnet build
.PHONY: testnet
testnet: clean build wasm
	@echo "Testnet build complete!"
	@echo "Ready for deployment to Safrochain testnet"

# Deploy to testnet
.PHONY: deploy
deploy: testnet
	@echo "Deploying to Safrochain testnet..."
	@echo "Make sure to update config/deploy.toml with your addresses first!"
	@echo "Then run: ./scripts/deploy_tontine.sh"

# Help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build        - Build the contract"
	@echo "  wasm         - Generate optimized WASM"
	@echo "  test         - Run tests"
	@echo "  test-verbose - Run tests with output"
	@echo "  schema       - Generate JSON schema"
	@echo "  clean        - Clean build artifacts"
	@echo "  fmt          - Format code"
	@echo "  check        - Check code style"
	@echo "  clippy       - Run clippy linting"
	@echo "  audit        - Security audit"
	@echo "  outdated     - Check outdated dependencies"
	@echo "  install-deps - Install development dependencies"
	@echo "  setup        - Complete development setup"
	@echo "  dev          - Full development cycle"
	@echo "  testnet      - Build for testnet deployment"
	@echo "  help         - Show this help message"
