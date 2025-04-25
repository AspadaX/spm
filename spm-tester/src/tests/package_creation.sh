#!/usr/bin/env bash

# Package creation test module

# Test set 2: Package Creation
run_package_creation_tests() {
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
}