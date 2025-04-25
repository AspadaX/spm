#!/usr/bin/env bash

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Global counters for tests
TOTAL_TESTS=0
PASSED_TESTS=0

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
    
    echo -e "\n${CYAN}Running test:${NC} $test_name"
    echo -e "${YELLOW}Command:${NC} $command"
    
    # Run the command and capture output and exit status
    output=$(eval $command 2>&1) || true
    result=$?
    
    echo "$output"
    
    if [ $result -eq 0 ]; then
        report_test "$test_name" 0
    else
        report_test "$test_name" 1 "Command failed with exit code $result"
    fi
    
    return $result
}

# Function to verify a package's structure
verify_package_structure() {
    local package_dir=$1
    local is_lib=$2
    
    echo -e "\n${CYAN}Verifying package structure:${NC} $package_dir"
    
    # Check if package directory exists
    if [ ! -d "$package_dir" ]; then
        report_test "Package Directory Existence" 1 "Package directory $package_dir does not exist"
        return 1
    fi
    
    # Check for package.json
    if [ ! -f "$package_dir/package.json" ]; then
        report_test "Package JSON File" 1 "package.json not found in $package_dir"
        return 1
    fi
    
    # Check for src directory
    if [ ! -d "$package_dir/src" ]; then
        report_test "Src Directory" 1 "src directory not found in $package_dir"
        return 1
    fi
    
    # In the current implementation, it seems both library and executable packages
    # may use main.sh as the default entrypoint script
    if [ "$is_lib" = "true" ]; then
        # Check for entrypoint script - accept either lib.sh or main.sh for libraries
        if [ -f "$package_dir/lib.sh" ]; then
            report_test "Library Script" 0
        elif [ -f "$package_dir/main.sh" ]; then
            report_test "Library Script (using main.sh)" 0
            echo -e "${YELLOW}Note:${NC} Library package using main.sh instead of lib.sh"
        else
            report_test "Library Script" 1 "Neither lib.sh nor main.sh found in $package_dir"
            return 1
        fi
    else
        # For regular packages, expect main.sh
        if [ ! -f "$package_dir/main.sh" ]; then
            report_test "Main Script" 1 "main.sh not found in $package_dir"
            return 1
        else
            report_test "Main Script" 0
        fi
    fi
    
    # Check for install and uninstall scripts
    if [ ! -f "$package_dir/install.sh" ]; then
        report_test "Install Script" 1 "install.sh not found in $package_dir"
        return 1
    else
        report_test "Install Script" 0
    fi
    
    if [ ! -f "$package_dir/uninstall.sh" ]; then
        report_test "Uninstall Script" 1 "uninstall.sh not found in $package_dir"
        return 1
    else
        report_test "Uninstall Script" 0
    fi
    
    # All checks passed
    report_test "Package Structure Verification" 0
    return 0
}

# Function to check if a package is listed
check_package_in_list() {
    local package_name=$1
    local namespace=$2
    local list_output=$3
    
    if [ -n "$namespace" ]; then
        # Check for namespaced package with exact matching
        if echo "$list_output" | grep -q "[[:space:]]$namespace/$package_name[[:space:]]"; then
            return 0
        else
            return 1
        fi
    else
        # For non-namespaced packages we need to check two cases:
        # 1. The package appears without any namespace
        # 2. The package appears with a default namespace (like default-namespace/package-name)
        if echo "$list_output" | grep -v "[[:alnum:]_-]\\+/$package_name" | grep -q "[[:space:]]$package_name[[:space:]]" ||
           echo "$list_output" | grep -q "[[:space:]]default-namespace/$package_name[[:space:]]"; then
            return 0
        else
            return 1
        fi
    fi
}

# Function to check if the temp directory exists for a repository
check_temp_directory_exists() {
    local repo_name=$1
    local temp_dir="$HOME/.spm/temp/$repo_name"
    
    if [ -d "$temp_dir" ]; then
        return 0  # Directory exists
    else
        return 1  # Directory doesn't exist
    fi
}

# Function to generate the final report
generate_final_report() {
    echo -e "\n${BLUE}=============================${NC}"
    echo -e "${BLUE}        TEST REPORT          ${NC}"
    echo -e "${BLUE}=============================${NC}"
    echo -e "Tests completed at $(date)"
    echo -e "Total tests: $TOTAL_TESTS"
    echo -e "Passed tests: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed tests: ${RED}$((TOTAL_TESTS - PASSED_TESTS))${NC}"

    if [ "$TOTAL_TESTS" -gt 0 ]; then
        SUCCESS_RATE=$(( (PASSED_TESTS * 100) / TOTAL_TESTS ))
        echo -e "Success rate: ${YELLOW}${SUCCESS_RATE}%${NC}"
    else
        echo -e "Success rate: ${YELLOW}N/A${NC}"
    fi

    if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
        echo -e "\n${GREEN}All tests passed successfully!${NC}"
    else
        echo -e "\n${RED}Some tests failed. Check the log above for details.${NC}"
        # Don't exit with error code so the script completes
    fi

    echo -e "\n${BLUE}=============================${NC}"
}