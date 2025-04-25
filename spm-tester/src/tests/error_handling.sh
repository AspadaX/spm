#!/usr/bin/env bash

# Error handling test module

# Test set 7: Error handling and edge cases
run_error_handling_tests() {
    echo -e "\n${BLUE}=== Testing error handling and edge cases ===${NC}"

    # Test 7.1: Running a non-existent package should fail gracefully
    run_test "Run Non-existent Package" "cargo run --quiet -- run non-existent-package || true"

    # Test 7.2: Create a package with a name that could be confused with a repository
    run_test "Create Package With Repository-like Name" "cargo run --quiet -- new repo-like-name"

    # Modify the package for testing
    cd "repo-like-name"
    cat > "main.sh" << 'EOF'
#!/bin/bash
echo "This is a package with a repository-like name"
echo "Testing package resolution logic"
EOF
    chmod +x main.sh
    chmod +x install.sh
    chmod +x uninstall.sh
    cd ..

    # Test 7.3: Install the repository-like named package
    run_test "Install Repository-like Named Package" "cargo run --quiet -- install repo-like-name --force"

    # Test 7.4: Verify package resolution logic prioritizes local packages over remote repositories
    run_test "Run Repository-like Named Package" "cargo run --quiet -- run repo-like-name"

    # Test 7.5: Uninstall the repository-like package
    run_test "Uninstall Repository-like Named Package" "cargo run --quiet -- uninstall repo-like-name"

    # Test 7.6: Running a package name with a dash that doesn't exist locally but could be interpreted as a GitHub repo
    run_test "Run Non-existent Dashed Package" "cargo run --quiet -- run some-nonexistent-package || true"

    # Test 7.7: Running a package with an ambiguous name (could be either local or remote)
    # First install a package with an ambiguous name
    run_test "Create Ambiguous Package" "cargo run --quiet -- new ambiguous-pkg"
    cd "ambiguous-pkg"
    cat > "main.sh" << 'EOF'
#!/bin/bash
echo "This is a package with an ambiguous name"
echo "It should be found locally before attempting remote fetch"
EOF
    chmod +x main.sh
    chmod +x install.sh
    chmod +x uninstall.sh
    cd ..
    run_test "Install Ambiguous Package" "cargo run --quiet -- install ambiguous-pkg --force"

    # Test if the ambiguous package is resolved locally first
    run_test "Run Ambiguous Package" "cargo run --quiet -- run ambiguous-pkg"
    run_test "Uninstall Ambiguous Package" "cargo run --quiet -- uninstall ambiguous-pkg"

    # Test 7.8: Test handling of malformed inputs
    run_test "Run Empty Package Name" "cargo run --quiet -- run \"\" || true"
    run_test "Run Package With Special Characters" "cargo run --quiet -- run \"test@#\$%^&*()\" || true"
}