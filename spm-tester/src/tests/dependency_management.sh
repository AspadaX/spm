#!/usr/bin/env bash

# SPM Dependency Management Test Suite
# Tests: add, remove, refresh, and edge cases for dependencies

run_dependency_management_tests() {
    echo -e "\n${BLUE}=============================================${NC}"
    echo -e "${BLUE}   Testing Dependency Management Features   ${NC}"
    echo -e "${BLUE}=============================================${NC}"

    local test_dir
    test_dir="$(mktemp -d)"
    cd "$test_dir" || exit 1
    echo -e "${BLUE}Test workspace:${NC} $test_dir"

    # 1. Create a new SPM package
    run_spm init --interpreter bash
    assert_file_exists "package.json" "package.json not created by init"

    # 2. Add a local dependency (simulate a local package)
    mkdir local-lib
    cd local-lib
    run_spm init --lib --interpreter bash
    echo 'echo "Hello from local-lib"' > lib.sh
    chmod +x lib.sh
    cd ..
    run_spm add "./local-lib"
    assert_contains "$(cat package.json)" "local-lib"

    # 3. Add a remote (GitHub) dependency (should succeed)
    run_spm add "AspadaX/test-spm-library" --version main
    assert_contains "$(cat package.json)" "test-spm-library"

    # 3b. Add a remote (GitHub) non-library dependency (should fail)
    if run_spm add "AspadaX/quick-git" --version main; then
        echo -e "${GREEN}✓ quick-git (non-library) add succeeded as expected${NC}"
    else
        echo -e "${RED}✗ quick-git (non-library) should have been added successfully${NC}"
    fi

    # 4. Refresh dependencies (should fetch both)
    run_spm refresh
    assert_dir_exists "dependencies/local/local-lib" "Local dependency not installed"
    assert_dir_exists "dependencies/AspadaX/test-spm-library" "GitHub dependency not installed"

    # 5. Remove a dependency and verify
    run_spm remove local-lib
    ! grep -q "local-lib" package.json && echo -e "${GREEN}✓ local-lib removed from package.json${NC}" || echo -e "${RED}✗ local-lib still present in package.json${NC}"

    # 6. Remove a non-existent dependency (should fail gracefully)
    run_spm remove does-not-exist && echo -e "${RED}✗ Removing non-existent dependency should fail${NC}" || echo -e "${GREEN}✓ Removing non-existent dependency failed as expected${NC}"

    # 7. Add the same dependency twice (should not duplicate)
    run_spm add "AspadaX/test-spm-library" --version main
    run_spm add "AspadaX/test-spm-library" --version main
    count=$(grep -o 'test-spm-library' package.json | wc -l)
    if [ "$count" -le 2 ]; then
        echo -e "${GREEN}✓ No duplicate test-spm-library entries${NC}"
    else
        echo -e "${RED}✗ Duplicate test-spm-library entries found${NC}"
    fi

    # 8. Remove all dependencies and refresh (should handle empty gracefully)
    run_spm remove test-spm-library
    run_spm refresh && echo -e "${GREEN}✓ Refresh with no dependencies succeeded${NC}" || echo -e "${RED}✗ Refresh with no dependencies failed${NC}"

    # Clean up
    cd / && rm -rf "$test_dir"
    echo -e "\n${BLUE}Dependency management tests completed.${NC}"
}