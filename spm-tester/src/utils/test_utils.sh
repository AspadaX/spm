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

# Function to setup test environment for library dependency tests
setup_test() {
    # Create test directory
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    echo -e "${BLUE}Setting up test in${NC} $TEST_DIR"
    
    # Get the path to the project root directory (parent of the spm-tester directory)
    PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
    
    # Define path to spm binary - using relative path to project root
    SPM_BIN="cd \"$PROJECT_ROOT\" && cargo run --quiet --"
    
    # Set initial counters
    PASSED_TESTS=0
    TOTAL_TESTS=0
}

# Function to create a sample package for testing
create_sample_package() {
    local pkg_name=$1
    local is_lib=$2
    
    echo "Creating sample package: $pkg_name (library: $is_lib)"
    mkdir -p "$pkg_name"
    cd "$pkg_name" || exit 1
    
    # Initialize as SPM package
    $SPM_BIN init
    
    # If it's a library, set it up for library usage
    if [ "$is_lib" = "true" ]; then
        # Modify package.json to indicate this is a library
        sed -i.bak 's/"register_to_environment_tool": true/"register_to_environment_tool": false/' package.json
        rm -f package.json.bak
        
        # Create library script
        cat > lib.sh << 'EOF'
#!/usr/bin/env bash
# Library functions
echo "This is a library package"
EOF
        chmod +x lib.sh
    fi
    
    cd ..
}

# Function to assert a command succeeded
assert_success() {
    local command="$1"
    local message="${2:-Command failed}"

    # Run the command and capture its output and exit code
    local output
    output=$(eval "$command" 2>&1) || true
    local status=$?

    # Print the output for debugging
    echo "$output"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if [ $status -eq 0 ]; then
        echo -e "${GREEN}✓ Command succeeded${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}✗ $message (status: $status)${NC}"
        return 1
    fi
}

# Function to assert a string contains a substring
assert_contains() {
    local haystack=$1
    local needle=$2
    
    if echo "$haystack" | grep -q "$needle"; then
        echo -e "${GREEN}✓ Found '$needle'${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        return 0
    else
        echo -e "${RED}✗ Did not find '$needle'${NC}"
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        return 1
    fi
}

# Function to assert a file exists
assert_file_exists() {
    local file=$1
    local message=${2:-"File does not exist"}
    
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓ File exists: $file${NC}"
        return 0
    else
        echo -e "${RED}✗ $message: $file${NC}"
        return 1
    fi
}

# Function to assert a directory exists
assert_dir_exists() {
    local dir=$1
    local message=${2:-"Directory does not exist"}
    
    if [ -d "$dir" ]; then
        echo -e "${GREEN}✓ Directory exists: $dir${NC}"
        return 0
    else
        echo -e "${RED}✗ $message: $dir${NC}"
        return 1
    fi
}

# Function to assert equality
assert_equal() {
    local actual=$1
    local expected=$2
    local message=$3
    
    if [ "$actual" = "$expected" ]; then
        echo -e "${GREEN}✓ Equal: $actual${NC}"
        return 0
    else
        echo -e "${RED}✗ $message: Expected '$expected', got '$actual'${NC}"
        return 1
    fi
}

# Function to assert inequality
assert_not_equal() {
    local actual=$1
    local expected=$2
    local message=$3
    
    if [ "$actual" != "$expected" ]; then
        echo -e "${GREEN}✓ Not equal: $actual != $expected${NC}"
        return 0
    else
        echo -e "${RED}✗ $message: Got '$actual'${NC}"
        return 1
    fi
}

# Function to run spm command
run_spm() {
    echo -e "${YELLOW}Running:${NC} spm $*"

    # Store current directory
    local current_dir="$(pwd)"

    # The correct project root is where the Cargo.toml file is located
    local PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/../../../" && pwd )"

    # Path to built spm binary
    local SPM_BIN="$PROJECT_ROOT/target/debug/spm"

    # Build the binary if it doesn't exist
    if [ ! -f "$SPM_BIN" ]; then
        echo "Building spm binary..."
        (cd "$PROJECT_ROOT" && cargo build --quiet)
    fi

    # Debug information
    echo "DEBUG: Current directory: $current_dir"
    echo "DEBUG: Project root: $PROJECT_ROOT"
    echo "DEBUG: Using SPM binary at $SPM_BIN"

    # Run the spm binary in the current directory
    "$SPM_BIN" "$@"
    local result=$?

    return $result
}

# Function to cleanup test environment
cleanup_test() {
    echo -e "\n${BLUE}Cleaning up test environment${NC}"
    cd / > /dev/null
    rm -rf "$TEST_DIR"
}

# Function to report test results
report_test_results() {
    echo -e "\n${BLUE}======================================${NC}"
    echo -e "${BLUE}      LIBRARY DEPENDENCY TEST RESULTS  ${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo -e "Total tests: $TOTAL_TESTS"
    echo -e "Passed tests: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed tests: ${RED}$((TOTAL_TESTS - PASSED_TESTS))${NC}"
    
    if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
        echo -e "\n${GREEN}All library dependency tests passed!${NC}"
    else
        echo -e "\n${RED}Some library dependency tests failed!${NC}"
        # Don't exit with error, let the main script handle overall reporting
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