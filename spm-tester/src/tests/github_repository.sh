#!/usr/bin/env bash

# GitHub repository test module

# Test set 5: GitHub repository support
run_github_repository_tests() {
    echo -e "\n${BLUE}=== Testing GitHub repository support ===${NC}"

    # eck if temp directory exists before run
    if check_temp_directory_exists "$TEST_GITHUB_REPO"; then
        echo -e "${YELLOW}Warning:${NC} Temporary directory for $TEST_GITHUB_REPO already exists. Cleaning up first."
        rm -rf "$HOME/.spm/temp/$TEST_GITHUB_REPO"
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
}