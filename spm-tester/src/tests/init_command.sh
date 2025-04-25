#!/usr/bin/env bash

# Init command test module

# Test set 4: Init command
run_init_command_tests() {
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
}