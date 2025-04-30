#!/usr/bin/env bash

# Test script for GitHub library repository
# This test verifies using the test-spm-library from GitHub as a dependency

# Wrapper function to run GitHub library tests
run_github_library_tests() {
    echo -e "\n${BLUE}=============================================${NC}"
    echo -e "${BLUE}   Testing GitHub Library Repository Support   ${NC}"
    echo -e "${BLUE}=============================================${NC}"

    # Setup test environment
    local test_dir="$(mktemp -d)"
    cd "$test_dir"
    echo -e "${BLUE}Setting up test in${NC} $test_dir"

    # Test 1: Create a package to use the library
    echo -e "\n${CYAN}Test 1: Creating a package to use the library${NC}"
    mkdir -p test-consumer
    cd test-consumer

    # Initialize as SPM package
    assert_success "run_spm init" "Failed to initialize test package"

    # Test 2: Add the GitHub library as a dependency using spm add
    echo -e "\n${CYAN}Test 2: Adding GitHub library as a dependency${NC}"
    assert_success "run_spm add AspadaX/test-spm-library --version main" "Failed to add GitHub library dependency"

    # Verify the library was added correctly to package.json
    echo -e "\n${CYAN}Checking if library was added correctly to package.json${NC}"
    # assert_contains increments counters internally
    assert_contains "$(cat package.json)" "test-spm-library"
    assert_contains "$(cat package.json)" "AspadaX" # Check for namespace

    # Test 3: Install the library dependency using spm refresh
    echo -e "\n${CYAN}Test 3: Installing the library dependency${NC}"
    assert_success "run_spm refresh" "Failed to install dependencies via refresh"

    # Verify the library was installed in the correct namespaced path
    echo -e "\n${CYAN}Verifying library installation${NC}"
    local expected_dep_path="dependencies/AspadaX/test-spm-library"
    # Use assert_dir_exists which increments counters internally
    assert_dir_exists "$expected_dep_path" "GitHub library installation failed - directory $expected_dep_path not found"

    # Test 6: Test refreshing the library using spm refresh
    echo -e "\n${CYAN}Test 6: Testing refresh of GitHub library${NC}"
    assert_success "run_spm refresh" "Failed to refresh GitHub library using 'spm refresh'"
    # Use assert_dir_exists which increments counters internally
    assert_dir_exists "$expected_dep_path" "GitHub library directory missing after refresh"

    # Clean up
    cd / > /dev/null
    rm -rf "$test_dir"

    # Report results
    echo -e "\n${BLUE}======================================${NC}"
    echo -e "${BLUE}      GITHUB LIBRARY TEST RESULTS      ${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo -e "Total tests: $TOTAL_TESTS"
    echo -e "Passed tests: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed tests: ${RED}$((TOTAL_TESTS - PASSED_TESTS))${NC}"

    if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
        echo -e "\n${GREEN}All GitHub library tests passed!${NC}"
    else
        echo -e "\n${RED}Some GitHub library tests failed!${NC}"
    fi
}