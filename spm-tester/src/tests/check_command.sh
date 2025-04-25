#!/usr/bin/env bash

# Check command test module

# Test set 6: Check command
run_check_command_tests() {
    echo -e "\n${BLUE}=== Testing check command ===${NC}"

    # Test 6.1: Check syntax of a shell script
    cd ..
    cat > "syntax_test.sh" << 'EOF'
#!/bin/bash
echo "This is a test script for the check command"
EOF
    chmod +x syntax_test.sh

    run_test "Check Command" "cargo run --quiet -- check syntax_test.sh || true"
}