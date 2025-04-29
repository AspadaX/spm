#!/usr/bin/env bash

# Test script for SPM library dependency features
# Tests the ability to add, install, and use library dependencies

# Wrapper function to run library dependency tests
run_library_dependency_tests() {
    echo -e "\n${BLUE}=============================================${NC}"
    echo -e "${BLUE}   Testing Library Dependency Features   ${NC}"
    echo -e "${BLUE}=============================================${NC}"
    
    # Setup test environment
    local test_dir="$(mktemp -d)"
    cd "$test_dir"
    echo -e "${BLUE}Setting up test in${NC} $test_dir"
    
    # Test 1: Create a library package
    echo -e "\n${CYAN}Test 1: Creating a library package${NC}"
    mkdir -p test-library
    cd test-library
    
    # Initialize as SPM package
    run_spm init --lib
    assert_success "Failed to initialize library package"
    
    # Mark as library by setting register_to_environment_tool to false
    sed -i.bak 's/"register_to_environment_tool": true/"register_to_environment_tool": false/' package.json
    rm -f package.json.bak
    
    # Add library content
    cat > "lib.sh" << 'EOF'
#!/usr/bin/env bash
# Test library functions

TEST_LIBRARY_VERSION="1.0.0"

test_lib_function() {
    local message="$1"
    echo "Library function called with: $message"
}

get_library_version() {
    echo "$TEST_LIBRARY_VERSION"
}
EOF
    chmod +x lib.sh
    
    cd ..
    
    # Test 2: Create a consumer package
    echo -e "\n${CYAN}Test 2: Creating a consumer package${NC}"
    mkdir -p test-consumer
    cd test-consumer
    run_spm init
    assert_success "Failed to initialize consumer package"
    
    # Test 3: Add library as a dependency
    echo -e "\n${CYAN}Test 3: Adding library as a dependency${NC}"
    
    # Update package.json manually to add library dependency with library_only flag
    pkg_json_content=$(cat package.json)
    temp_file=$(mktemp)
    echo "$pkg_json_content" | jq '.dependencies = [{"name": "test-library", "url": "../test-library", "version": "0.1.0", "library_only": true}]' > "$temp_file"
    mv "$temp_file" package.json
    
    # Verify library dependency was added correctly
    output=$(cat "package.json")
    assert_contains "$output" "test-library"
    assert_contains "$output" "\"library_only\": true"
    
    # Test 4: Install the library dependency
    echo -e "\n${CYAN}Test 4: Installing the library dependency${NC}"
    run_spm refresh
    assert_success "Failed to install library dependency"
    
    # Verify the library was installed
    assert_dir_exists "dependencies/local/test-library" "Library directory not found"
    assert_file_exists "dependencies/local/test-library/lib.sh" "Library script not found"
    
    # Test 5: Create a script that uses the library
    echo -e "\n${CYAN}Test 5: Creating a script that uses the library${NC}"
    cat > "main.sh" << 'EOF'
#!/usr/bin/env bash
# Test script that uses the library

# Source the library
source "./dependencies/local/test-library/lib.sh"

# Use functions from the library
echo "Testing library dependency:"
test_lib_function "Hello from library test"
echo "Library version: $(get_library_version)"
EOF
    chmod +x main.sh
    
    # Test 6: Run the script that uses the library
    echo -e "\n${CYAN}Test 6: Running script that uses library${NC}"
    output=$(./main.sh 2>&1)
    assert_contains "$output" "Library function called with: Hello from library test"
    assert_contains "$output" "Library version: 1.0.0"
    
    # Test 7: Test version constraints
    echo -e "\n${CYAN}Test 7: Testing version constraints on library dependency${NC}"
    # First update package.json to add version constraint
    temp_file=$(mktemp)
    jq '.dependencies["test-library"].version = "0.1.0"' package.json > "$temp_file" && mv "$temp_file" package.json
    
    # Verify version constraint was added
    output=$(cat "package.json")
    assert_contains "$output" "\"version\": \"0.1.0\""
    
    # Test 8: Export functions from library
    echo -e "\n${CYAN}Test 8: Testing function export from library${NC}"
    
    # Create a new script that demonstrates library functionality export
    cat > "export_test.sh" << 'EOF'
#!/usr/bin/env bash
# Test exporting functions from a library

# Source the library
source "./dependencies/local/test-library/lib.sh"

# Export function to be used by other scripts
export -f test_lib_function
export -f get_library_version
export TEST_LIBRARY_VERSION

# Call bash with a new script that uses the exported functions
bash -c 'echo "Exported function test: $(test_lib_function "Called from external script")"; echo "Exported library version: $TEST_LIBRARY_VERSION"'
EOF
    chmod +x export_test.sh
    
    # Run the export test
    output=$(./export_test.sh 2>&1)
    assert_contains "$output" "Exported function test"
    assert_contains "$output" "Exported library version: 1.0.0"
    
    # Test 9: Test removing a library dependency
    echo -e "\n${CYAN}Test 9: Removing library dependency${NC}"
    run_spm remove test-library
    assert_success "Failed to remove library dependency"
    
    # Verify library was removed
    output=$(cat "package.json")
    if ! echo "$output" | grep -q "test-library"; then
        echo -e "${GREEN}✓ Library dependency was successfully removed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ Failed to remove library dependency${NC}"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Clean up
    cd / > /dev/null
    rm -rf "$test_dir"
    
    # Report results
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
    fi
}