#!/bin/bash

# Test runner script for memo CLI tool
set -e

echo "🧪 Running memo CLI tests..."
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print section headers
print_section() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Function to handle test results
handle_result() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $1 passed${NC}"
    else
        echo -e "${RED}❌ $1 failed${NC}"
        exit 1
    fi
}

# Build the project first
print_section "Building project"
cargo build
handle_result "Build"

# Run unit tests
print_section "Running unit tests"
cargo test --lib
handle_result "Unit tests"

# Run Rust integration tests
print_section "Running Rust integration tests"
cargo test --test integration_tests
handle_result "Rust integration tests"

# Run bash integration tests
print_section "Running bash integration tests"
./tests/integration_test.sh
handle_result "Bash integration tests"

# Run all tests together
print_section "Running all tests"
cargo test
handle_result "All cargo tests"

echo
echo -e "${GREEN}🎉 All tests passed successfully!${NC}"
echo
echo "You can now install the memo tool with:"
echo "  cargo install --path ."
