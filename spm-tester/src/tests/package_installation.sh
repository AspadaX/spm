#!/usr/bin/env bash

# Package installation test module

# Test set 3: Package Installation and Management
run_package_installation_tests() {
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
}