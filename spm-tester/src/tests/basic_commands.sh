#!/usr/bin/env bash

# Basic commands test module

# Test set 1: Basic commands
run_basic_commands() {
    echo -e "\n${BLUE}=== Testing basic commands ===${NC}"

    # Test 1.1: Version command
    run_test "Version Command" "cargo run --quiet -- version"
}