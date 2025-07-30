#!/bin/bash

# Integration test for memo CLI tool
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
TEST_DIR=$(mktemp -d)
MEMO_BINARY="./target/debug/memo"
EDITOR="echo"  # Use echo as a dummy editor for testing

echo -e "${YELLOW}Starting memo integration tests...${NC}"
echo "Test directory: $TEST_DIR"

# Set up test environment
export XDG_DATA_HOME="$TEST_DIR"
export EDITOR="$EDITOR"

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_pattern="$3"
    
    echo -n "Testing $test_name... "
    
    if output=$(eval "$test_command" 2>&1); then
        if [[ -z "$expected_pattern" ]] || echo "$output" | grep -q "$expected_pattern"; then
            echo -e "${GREEN}PASS${NC}"
            return 0
        else
            echo -e "${RED}FAIL${NC}"
            echo "Expected pattern: $expected_pattern"
            echo "Actual output: $output"
            return 1
        fi
    else
        echo -e "${RED}FAIL${NC}"
        echo "Command failed: $test_command"
        echo "Output: $output"
        return 1
    fi
}

# Build the binary if it doesn't exist
if [[ ! -f "$MEMO_BINARY" ]]; then
    echo "Building memo binary..."
    cargo build
fi

# Test 1: memo dir command
run_test "memo dir" "$MEMO_BINARY dir" "$TEST_DIR/memo"

# Test 2: memo list with no memos
run_test "memo list (empty)" "$MEMO_BINARY list" "No memos found"

# Test 3: memo help
run_test "memo help" "$MEMO_BINARY --help" "memo"

# Test 4: Test memo edit with non-existent ID (expect failure)
if output=$($MEMO_BINARY edit 999999 2>&1); then
    echo -e "Testing memo edit non-existent... ${RED}FAIL${NC} - Expected failure but command succeeded"
    exit 1
else
    if echo "$output" | grep -q "not found"; then
        echo -e "Testing memo edit non-existent... ${GREEN}PASS${NC}"
    else
        echo -e "Testing memo edit non-existent... ${RED}FAIL${NC} - Wrong error message: $output"
        exit 1
    fi
fi

# Test 5: Test directory structure after running memo dir
memo_dir="$TEST_DIR/memo"
$MEMO_BINARY dir > /dev/null  # This should create the directory structure
if [[ "$($MEMO_BINARY dir)" == "$memo_dir" ]]; then
    echo -e "Testing directory path consistency... ${GREEN}PASS${NC}"
else
    echo -e "Testing directory path consistency... ${RED}FAIL${NC}"
    exit 1
fi

# Cleanup
echo "Cleaning up test directory: $TEST_DIR"
rm -rf "$TEST_DIR"

echo -e "${GREEN}All integration tests completed!${NC}"
