#!/usr/bin/env bash

# Comprehensive Test Script for SPM (Shell Package Manager)
# This script tests all SPM commands and features, including namespaces and GitHub repository support
# Date: April 26, 2025

set -e  # Exit on any error

# Test directory setup
TEST_DIR="./spm_test_dir"
TEST_PACKAGE_NAME="spm-test-package"
TEST_NAMESPACE="test-namespace"
TEST_LIB_PACKAGE="spm-lib-package"
TEST_ARGS_PACKAGE="spm-args-test"
TEST_INIT_DIR="$TEST_DIR/init-test"
TEST_GITHUB_REPO="AspadaX/quick-git"  # Test repository with actual shell scripts

# Get the directory of this script to establish the base path
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Source the utility module
source "$SCRIPT_DIR/src/utils/test_utils.sh"

# Source all test modules
source "$SCRIPT_DIR/src/tests/basic_commands.sh"
source "$SCRIPT_DIR/src/tests/package_creation.sh"
source "$SCRIPT_DIR/src/tests/package_installation.sh"
source "$SCRIPT_DIR/src/tests/init_command.sh"
source "$SCRIPT_DIR/src/tests/github_repository.sh"
source "$SCRIPT_DIR/src/tests/check_command.sh"
source "$SCRIPT_DIR/src/tests/error_handling.sh"
source "$SCRIPT_DIR/src/tests/execution_path.sh"

# -- Main Function --
main() {
    # Print header
    echo -e "${BLUE}=============================${NC}"
    echo -e "${BLUE}   SPM COMPREHENSIVE TEST    ${NC}"
    echo -e "${BLUE}=============================${NC}"
    echo "Starting tests at $(date)"
    echo -e "Testing SPM version: $(cargo run --quiet -- version)"

    # Clean up any existing test directory and .spm directory in home for clean tests
    if [ -d "$TEST_DIR" ]; then
        echo "Removing existing test directory..."
        rm -rf "$TEST_DIR"
    fi
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"

    # Run test sets
    run_basic_commands
    run_package_creation_tests
    run_package_installation_tests
    run_init_command_tests
    run_github_repository_tests
    run_check_command_tests
    run_error_handling_tests
    run_execution_path_tests

    # Clean up
    cd ../..
    echo -e "\n${BLUE}Cleaning up...${NC}"
    rm -rf "$TEST_DIR"

    # Generate the final report
    generate_final_report
}

# Call the main function
main "$@"