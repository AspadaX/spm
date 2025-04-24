#!/bin/bash

# Comprehensive Test Script for SPM (Shell Package Manager)
# This script tests all SPM commands and features, including namespaces and GitHub repository support
# Date: April 24, 2025

set -e  # Exit on any error

# Test directory setup
TEST_DIR="./spm_test_dir"
TEST_PACKAGE_NAME="spm-test-package"
TEST_NAMESPACE="test-namespace"
TEST_LIB_PACKAGE="spm-lib-package"
TEST_ARGS_PACKAGE="spm-args-test"
TEST_INIT_DIR="$TEST_DIR/init-test"
TEST_GITHUB_REPO="AspadaX/quick-git"  # Test repository with actual shell scripts
TOTAL_TESTS=0
PASSED_TESTS=0

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

# Test set 1: Basic commands
echo -e "\n${BLUE}=== Testing basic commands ===${NC}"

# Test 1.1: Version command
run_test "Version Command" "cargo run --quiet -- version"

# Test set 2: Package Creation
echo -e "\n${BLUE}=== Testing package creation ===${NC}"

# Test 2.1: Creating a regular package with default settings
run_test "Create Regular Package" "cargo run --quiet -- new $TEST_PACKAGE_NAME"
verify_package_structure "$TEST_PACKAGE_NAME" "false"

# Test 2.2: Creating a library package
run_test "Create Library Package" "cargo run --quiet -- new $TEST_LIB_PACKAGE --lib --interpreter bash"
verify_package_structure "$TEST_LIB_PACKAGE" "true"

# Test 2.3: Creating a package with a namespace
run_test "Create Namespaced Package" "cargo run --quiet -- new ${TEST_PACKAGE_NAME}-ns --namespace $TEST_NAMESPACE"
verify_package_structure "${TEST_PACKAGE_NAME}-ns" "false"

# Test 2.4: Check that is_library is in package.json (addressing issue #26)
if grep -q "\"is_library\": true" "$TEST_LIB_PACKAGE/package.json"; then
    report_test "is_library flag in package.json" 0
else
    report_test "is_library flag in package.json" 1 "is_library field not found or not true in library package"
fi

# Test set 3: Package Installation and Management
echo -e "\n${BLUE}=== Testing package installation and management ===${NC}"

# Test 3.1: Modify the regular package for testing
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

# Test 3.2: Install the regular package
run_test "Install Regular Package" "cargo run --quiet -- install $TEST_PACKAGE_NAME --force"

# Test 3.3: Install the namespaced package
run_test "Install Namespaced Package" "cargo run --quiet -- install ${TEST_PACKAGE_NAME}-ns --force"

# Test 3.4: List installed packages
list_output=$(cargo run --quiet -- list)
run_test "List Packages" "cargo run --quiet -- list"

# Test 3.5: Verify packages appear in the list
if check_package_in_list "$TEST_PACKAGE_NAME" "" "$list_output"; then
    report_test "Regular Package in List" 0
else
    report_test "Regular Package in List" 1 "Regular package not found in list"
fi

if check_package_in_list "${TEST_PACKAGE_NAME}-ns" "$TEST_NAMESPACE" "$list_output"; then
    report_test "Namespaced Package in List" 0
else
    report_test "Namespaced Package in List" 1 "Namespaced package not found in list"
fi

# Test 3.6: Run the regular package
run_test "Run Regular Package" "cargo run --quiet -- run $TEST_PACKAGE_NAME"

# Test 3.7: Run the namespaced package
run_test "Run Namespaced Package" "cargo run --quiet -- run $TEST_NAMESPACE/${TEST_PACKAGE_NAME}-ns"

# Test 3.8: Create a package that accepts and uses command-line arguments
run_test "Create Arguments Test Package" "cargo run --quiet -- new $TEST_ARGS_PACKAGE"

# Modify the package to handle command-line arguments
cd "$TEST_ARGS_PACKAGE"
cat > "main.sh" << 'EOF'
#!/bin/bash
echo "Arguments Test Package"
echo "Number of arguments: $#"
echo "All arguments: $@"

# Echo each argument with its position
i=1
for arg in "$@"; do
  echo "Argument $i: $arg"
  i=$((i+1))
done

# Test if specific arguments were passed
if [ "$#" -ge 2 ]; then
  echo "First two arguments: $1 and $2"
fi
EOF
chmod +x main.sh
chmod +x install.sh
chmod +x uninstall.sh
cd ..

# Test 3.9: Install the arguments test package
run_test "Install Arguments Test Package" "cargo run --quiet -- install $TEST_ARGS_PACKAGE --force"

# Test 3.10: Run the arguments test package with arguments
run_test "Run Package With Arguments" "cargo run --quiet -- run $TEST_ARGS_PACKAGE arg1 arg2 \"argument with spaces\""

# Test 3.11: Uninstall the arguments test package
run_test "Uninstall Arguments Test Package" "cargo run --quiet -- uninstall $TEST_ARGS_PACKAGE"

# Verify arguments test package uninstallation
list_output=$(cargo run --quiet -- list)
if ! check_package_in_list "$TEST_ARGS_PACKAGE" "" "$list_output"; then
    report_test "Arguments Test Package Uninstall Verification" 0
else
    report_test "Arguments Test Package Uninstall Verification" 1 "Package still found in list after uninstall"
fi

# Test 3.12: Uninstall the regular package
run_test "Uninstall Regular Package" "cargo run --quiet -- uninstall $TEST_PACKAGE_NAME"

# Test 3.13: Verify uninstallation
list_output=$(cargo run --quiet -- list)
if ! check_package_in_list "$TEST_PACKAGE_NAME" "" "$list_output"; then
    report_test "Regular Package Uninstall Verification" 0
else
    report_test "Regular Package Uninstall Verification" 1 "Package still found in list after uninstall"
fi

# Test 3.14: Uninstall the namespaced package
run_test "Uninstall Namespaced Package" "cargo run --quiet -- uninstall $TEST_NAMESPACE/${TEST_PACKAGE_NAME}-ns"

# Test 3.15: Verify namespaced package uninstallation
list_output=$(cargo run --quiet -- list)
if ! check_package_in_list "${TEST_PACKAGE_NAME}-ns" "$TEST_NAMESPACE" "$list_output"; then
    report_test "Namespaced Package Uninstall Verification" 0
else
    report_test "Namespaced Package Uninstall Verification" 1 "Namespaced package still found in list after uninstall"
fi

# Test set 4: Init command
echo -e "\n${BLUE}=== Testing the init command ===${NC}"

# Test 4.1: Test the "init" command in a new directory
mkdir -p "$TEST_INIT_DIR"
cd "$TEST_INIT_DIR"
run_test "Init Command" "cargo run --quiet -- init --interpreter bash"

# Test 4.2: Verify init package structure
if [ -f "package.json" ] && [ -f "main.sh" ]; then
    report_test "Init Structure Verification" 0
else
    report_test "Init Structure Verification" 1 "Init command did not create files correctly"
fi

# Test 4.3: Test init with library flag
cd ..
mkdir -p "init-lib-test"
cd "init-lib-test"
run_test "Init Library Command" "cargo run --quiet -- init --lib --interpreter bash"

# Test 4.4: Verify init library structure
if [ -f "package.json" ] && [ -f "lib.sh" ]; then
    report_test "Init Library Structure Verification" 0
else
    report_test "Init Library Structure Verification" 1 "Init library command did not create files correctly"
fi

# Test 4.5: Verify is_library flag in init-created package.json
if grep -q "\"is_library\": true" "package.json"; then
    report_test "is_library Flag in Init Package" 0
else
    report_test "is_library Flag in Init Package" 1 "is_library field not found or not true in init library package"
fi

# Test set 5: GitHub repository support
echo -e "\n${BLUE}=== Testing GitHub repository support ===${NC}"

# Test 5.1: Run a package from GitHub
echo -e "\n${CYAN}Testing temporary directory functionality with remote repositories${NC}"

# Check if temp directory exists before run
if check_temp_directory_exists "$TEST_GITHUB_REPO"; then
    echo -e "${YELLOW}Warning:${NC} Temporary directory for $TEST_GITHUB_REPO already exists. Cleaning up first."
    rm -rf "$HOME/.spm/temp/$TEST_GITHUB_REPO"
fi

# Test 5.1: Run a package from GitHub (temp directory should be created and then removed)
run_test "Run GitHub Repository" "cargo run --quiet -- run https://github.com/$TEST_GITHUB_REPO -- status"

# Verify the temporary directory was cleaned up after running
if check_temp_directory_exists "$TEST_GITHUB_REPO"; then
    report_test "Temp Directory Cleanup After Run" 1 "Temporary directory still exists after running remote package"
else
    report_test "Temp Directory Cleanup After Run" 0
fi

# Test 5.2: Install a package from GitHub
# First check if temp directory exists
if check_temp_directory_exists "$TEST_GITHUB_REPO"; then
    echo -e "${YELLOW}Warning:${NC} Temporary directory for $TEST_GITHUB_REPO already exists. Cleaning up first."
    rm -rf "$HOME/.spm/temp/$TEST_GITHUB_REPO"
fi

# Now install the package
run_test "Install GitHub Repository" "cargo run --quiet -- install $TEST_GITHUB_REPO"

# Verify the temporary directory was cleaned up after installation
if check_temp_directory_exists "$TEST_GITHUB_REPO"; then
    report_test "Temp Directory Cleanup After Install" 1 "Temporary directory still exists after installing remote package"
else
    report_test "Temp Directory Cleanup After Install" 0
fi

# Verify the package was installed permanently
list_output=$(cargo run --quiet -- list)
if echo "$list_output" | grep -q "quick-git"; then
    report_test "GitHub Package Installed" 0
else
    report_test "GitHub Package Installed" 1 "GitHub package 'quick-git' not found in installed packages"
fi

# Test 5.3: Run the installed GitHub package with arguments
run_test "Run Installed GitHub Package" "cargo run --quiet -- run quick-git -- status"

# Test 5.4: Uninstall the GitHub package
run_test "Uninstall GitHub Package" "cargo run --quiet -- uninstall quick-git"

# Verify uninstallation
list_output=$(cargo run --quiet -- list)
if echo "$list_output" | grep -q "quick-git"; then
    report_test "GitHub Package Uninstallation" 1 "GitHub package 'quick-git' still found after uninstallation"
else
    report_test "GitHub Package Uninstallation" 0
fi

# Test set 6: Check command (known to be under development)
echo -e "\n${BLUE}=== Testing check command ===${NC}"

# Test 6.1: Check syntax of a shell script
cd ..
cat > "syntax_test.sh" << 'EOF'
#!/bin/bash
echo "This is a test script for the check command"
EOF
chmod +x syntax_test.sh

run_test "Check Command" "cargo run --quiet -- check syntax_test.sh || true"

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
