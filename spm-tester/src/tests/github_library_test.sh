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
    run_spm init
    assert_success "Failed to initialize test package"
    
    # Test 2: Add the GitHub library as a dependency
    echo -e "\n${CYAN}Test 2: Adding GitHub library as a dependency${NC}"
    
    # Update package.json manually to add the GitHub library as a dependency with library_only flag
    pkg_json_content=$(cat package.json)
    temp_file=$(mktemp)
    echo "$pkg_json_content" | jq '.dependencies = {"test-spm-library": {"url": "https://github.com/AspadaX/test-spm-library", "version": "0.1.0", "library_only": true}}' > "$temp_file"
    mv "$temp_file" package.json
    
    # Verify the library was added correctly
    echo -e "\n${CYAN}Checking if library was added correctly${NC}"
    output=$(cat "package.json")
    assert_contains "$output" "test-spm-library"
    assert_contains "$output" "\"library_only\": true"
    
    # Test 3: Install the library dependency
    echo -e "\n${CYAN}Test 3: Installing the library dependency${NC}"
    run_spm install
    assert_success "Failed to install dependencies"
    
    # Verify the library was installed
    echo -e "\n${CYAN}Verifying library installation${NC}"
    if [ -d "dependencies/test-spm-library" ]; then
        echo -e "${GREEN}✓ GitHub library was installed correctly${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ GitHub library installation failed${NC}"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Test 4: Create a script that uses the library
    echo -e "\n${CYAN}Test 4: Creating a script that uses the GitHub library${NC}"
    cat > "main.sh" << 'EOF'
#!/usr/bin/env bash
# Test script that uses the GitHub library

# Source the library
source "./dependencies/test-spm-library/lib.sh"

# Use functions from the library
echo "Testing GitHub library dependency:"
test_library_function "Hello from GitHub library test"
test_library_version
EOF
    chmod +x main.sh
    
    # Test 5: Run the script that uses the GitHub library
    echo -e "\n${CYAN}Test 5: Running script that uses GitHub library${NC}"
    output=$(./main.sh 2>&1)
    if echo "$output" | grep -q "This is a function from test-spm-library"; then
        echo -e "${GREEN}✓ Successfully used functions from GitHub library${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ Failed to use functions from GitHub library${NC}"
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Test 6: Test refreshing the library
    echo -e "\n${CYAN}Test 6: Testing refresh of GitHub library${NC}"
    # For now, refresh command doesn't support the --name flag
    # Use update command instead which should refresh dependencies
    run_spm update
    assert_success "Failed to refresh GitHub library"
    
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