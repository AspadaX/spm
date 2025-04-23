#!/bin/bash

# Comprehensive Test Script for SPM (Shell Package Manager)
# This script tests all SPM commands and generates a report at the end.

set -e  # Exit on any error

# Test directory setup
TEST_DIR="./spm_test_dir"
TEST_PACKAGE_NAME="spm-test-package"
TEST_INIT_DIR="$TEST_DIR/init-test"
TOTAL_TESTS=0
PASSED_TESTS=0

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to report test results
report_test() {
    local test_name=$1
    local result=$2
    local message=$3
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" -eq 0 ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name - $message"
    fi
}

# Function to run a test and capture the result
run_test() {
    local test_name=$1
    local command=$2
    
    echo -e "\n${BLUE}Running test:${NC} $test_name"
    echo -e "${YELLOW}Command:${NC} $command"
    
    # Run the command and capture output and exit status
    output=$(eval $command 2>&1)
    result=$?
    
    echo "$output"
    
    if [ $result -eq 0 ]; then
        report_test "$test_name" 0
    else
        report_test "$test_name" 1 "Command failed with exit code $result"
    fi
    
    return $result
}

# Print header
echo -e "${BLUE}=============================${NC}"
echo -e "${BLUE}   SPM COMPREHENSIVE TEST    ${NC}"
echo -e "${BLUE}=============================${NC}"
echo "Starting tests at $(date)"

# Clean up any existing test directory
if [ -d "$TEST_DIR" ]; then
    echo "Removing existing test directory..."
    rm -rf "$TEST_DIR"
fi
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Test 1: Version command
run_test "Version Command" "cargo run --release -- version"

# Test 2: Creating a new package with the "new" command
run_test "New Package Creation" "cargo run --release -- new $TEST_PACKAGE_NAME --interpreter bash"

# Verify package structure
if [ -d "$TEST_PACKAGE_NAME" ] && [ -f "$TEST_PACKAGE_NAME/package.json" ] && [ -f "$TEST_PACKAGE_NAME/main.sh" ]; then
    report_test "Package Structure Verification" 0
else
    report_test "Package Structure Verification" 1 "Package files not created correctly"
fi

# Test 3: Modify the package for testing
cd "$TEST_PACKAGE_NAME"
cat > "main.sh" << 'EOF'
#!/bin/bash
echo "Hello from SPM test package!"
echo "This is a test package created by the SPM test script."
echo "Current time: $(date)"
EOF
chmod +x main.sh
chmod +x install.sh
chmod +x uninstall.sh
cd ..

# Test 4: Install the package
run_test "Package Installation" "cargo run --release -- install $TEST_PACKAGE_NAME --force"

# Test 5: Run the package
run_test "Run Package" "cargo run --release -- run $TEST_PACKAGE_NAME"

# Test 6: List installed packages
run_test "List Packages" "cargo run --release -- list"

# Test 7: Uninstall the package
run_test "Uninstall Package" "cargo run --release -- uninstall $TEST_PACKAGE_NAME"

# Test 8: Verify uninstallation (grep should not find the package)
list_output=$(cargo run --release -- list)
if echo "$list_output" | grep -q "$TEST_PACKAGE_NAME"; then
    report_test "Uninstall Verification" 1 "Package still found in list after uninstall"
else
    report_test "Uninstall Verification" 0
fi

# Test 9: Test the "init" command in a new directory
mkdir -p "$TEST_INIT_DIR"
cd "$TEST_INIT_DIR"
run_test "Init Command" "cargo run --release -- init --interpreter bash"

# Verify init package structure
if [ -f "package.json" ] && [ -f "main.sh" ]; then
    report_test "Init Structure Verification" 0
else
    report_test "Init Structure Verification" 1 "Init command did not create files correctly"
fi

# Test 10: Check command (known to be under development)
cd ..
cat > "syntax_test.sh" << 'EOF'
#!/bin/bash
echo "This is a test script for the check command"
EOF
chmod +x syntax_test.sh

run_test "Check Command" "cargo run --release -- check syntax_test.sh" || true

# Clean up
cd ../..
echo -e "\n${BLUE}Cleaning up...${NC}"
rm -rf "$TEST_DIR"

# Generate the final report
echo -e "\n${BLUE}=============================${NC}"
echo -e "${BLUE}        TEST REPORT          ${NC}"
echo -e "${BLUE}=============================${NC}"
echo -e "Tests completed at $(date)"
echo -e "Total tests: $TOTAL_TESTS"
echo -e "Passed tests: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed tests: ${RED}$((TOTAL_TESTS - PASSED_TESTS))${NC}"
echo -e "Success rate: ${YELLOW}$(( (PASSED_TESTS * 100) / TOTAL_TESTS ))%${NC}"

if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
    echo -e "\n${GREEN}All tests passed successfully!${NC}"
else
    echo -e "\n${RED}Some tests failed. Check the log above for details.${NC}"
    # Don't exit with error code so the script completes
fi

echo -e "\n${BLUE}=============================${NC}"
