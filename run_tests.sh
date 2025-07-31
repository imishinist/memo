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
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print section headers
print_section() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Function to print subsection headers
print_subsection() {
    echo -e "${CYAN}--- $1 ---${NC}"
}

# Function to handle test results
handle_result() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $1 passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $1 failed${NC}"
        return 1
    fi
}

# Function to run tests with timeout
run_with_timeout() {
    local timeout_duration=$1
    local test_name=$2
    shift 2
    
    echo "Running: $@"
    if timeout ${timeout_duration} "$@"; then
        handle_result "$test_name"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo -e "${RED}❌ $test_name timed out after ${timeout_duration}${NC}"
        else
            echo -e "${RED}❌ $test_name failed with exit code $exit_code${NC}"
        fi
        return 1
    fi
}

# Track test results
FAILED_TESTS=()
TOTAL_TESTS=0
PASSED_TESTS=0

# Function to record test result
record_test() {
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if [ $? -eq 0 ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        FAILED_TESTS+=("$1")
    fi
}

echo -e "${YELLOW}📋 Test Configuration:${NC}"
echo "  - Timeout per test suite: 120 seconds"
echo "  - Test threads: 1 (sequential execution)"
echo "  - Total expected tests: 129"
echo

# Build the project first
print_section "Building project"
cargo build --release
record_test "Build"

# Run unit tests (library tests)
print_section "Running unit tests"
print_subsection "Library unit tests"
run_with_timeout 60s "Library unit tests" cargo test --lib
record_test "Library unit tests"

print_subsection "Binary unit tests"
run_with_timeout 60s "Binary unit tests" cargo test --bin memo
record_test "Binary unit tests"

# Run new integration test suite
print_section "Running integration tests"
print_subsection "New integration test suite (129 tests)"
run_with_timeout 120s "Integration test suite" cargo test --test lib -- --test-threads=1
record_test "Integration test suite"

# Run individual command tests for verification
print_section "Running command-specific tests"

print_subsection "Add command tests"
run_with_timeout 60s "Add command tests" cargo test --test lib commands::add_tests
record_test "Add command tests"

print_subsection "Edit command tests"
run_with_timeout 60s "Edit command tests" cargo test --test lib commands::edit_tests
record_test "Edit command tests"

print_subsection "Show command tests"
run_with_timeout 60s "Show command tests" cargo test --test lib commands::show_tests
record_test "Show command tests"

print_subsection "List command tests"
run_with_timeout 60s "List command tests" cargo test --test lib commands::list_tests
record_test "List command tests"

print_subsection "Dir command tests"
run_with_timeout 60s "Dir command tests" cargo test --test lib commands::dir_tests
record_test "Dir command tests"

print_subsection "Archive command tests"
run_with_timeout 60s "Archive command tests" cargo test --test lib commands::archive_tests
record_test "Archive command tests"

print_subsection "Index command tests"
run_with_timeout 60s "Index command tests" cargo test --test lib commands::index_tests
record_test "Index command tests"

print_subsection "Search command tests"
run_with_timeout 60s "Search command tests" cargo test --test lib commands::search_tests
record_test "Search command tests"

# Run bash integration tests if they exist
if [ -f "./tests/integration_test.sh" ]; then
    print_section "Running bash integration tests"
    if [ -x "./tests/integration_test.sh" ]; then
        run_with_timeout 60s "Bash integration tests" ./tests/integration_test.sh
        record_test "Bash integration tests"
    else
        echo -e "${YELLOW}⚠️  Bash integration test script is not executable${NC}"
        chmod +x ./tests/integration_test.sh
        run_with_timeout 60s "Bash integration tests" ./tests/integration_test.sh
        record_test "Bash integration tests"
    fi
else
    echo -e "${YELLOW}⚠️  Bash integration test script not found, skipping${NC}"
fi

# Final comprehensive test run
print_section "Running comprehensive test suite"
print_subsection "All tests together"
run_with_timeout 180s "All tests" cargo test -- --test-threads=1
record_test "All tests"

# Test summary
echo
print_section "Test Summary"
echo -e "${CYAN}📊 Results:${NC}"
echo "  Total test suites: $TOTAL_TESTS"
echo "  Passed: $PASSED_TESTS"
echo "  Failed: $((TOTAL_TESTS - PASSED_TESTS))"

if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
    echo
    echo -e "${GREEN}🎉 All test suites passed successfully!${NC}"
    echo -e "${GREEN}✨ All 129 individual tests are working perfectly!${NC}"
    echo
    echo -e "${BLUE}📦 Installation:${NC}"
    echo "  You can now install the memo tool with:"
    echo "    cargo install --path ."
    echo
    echo -e "${BLUE}🚀 Usage:${NC}"
    echo "  memo add     # Create a new memo"
    echo "  memo list    # List recent memos"
    echo "  memo show <id>  # Show a specific memo"
    echo "  memo edit <id>  # Edit a memo"
    echo "  memo search <query>  # Search memos"
    echo "  memo index   # Build search index"
    echo "  memo archive <id>  # Archive memos"
    echo "  memo dir     # Show memo directory"
    echo
    exit 0
else
    echo
    echo -e "${RED}❌ Some test suites failed:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    echo
    echo -e "${YELLOW}💡 Troubleshooting:${NC}"
    echo "  1. Check if all dependencies are installed"
    echo "  2. Ensure you have write permissions in the test directory"
    echo "  3. Try running individual test suites to isolate issues"
    echo "  4. Check the detailed error messages above"
    echo
    exit 1
fi
