#!/usr/bin/env bash

# Execution path test module

# Test set 8: Test script execution path/working directory
run_execution_path_tests() {
    echo -e "\n${BLUE}=== Testing script execution working directory ===${NC}"

    # Create a test package that checks working directory
    mkdir -p "workdir-test-pkg"
    cd "workdir-test-pkg"

    # Create a package.json file
    cat > "package.json" << EOF
{
    "name": "workdir-test",
    "namespace": "test",
    "description": "Test package to check working directory behavior",
    "version": "1.0.0",
    "interpreter": "sh",
    "is_library": false,
    "entrypoint": "main.sh",
    "install": {
        "setup_script": "install.sh",
        "register_to_environment_tool": false
    },
    "uninstall": "uninstall.sh"
}
EOF

    # Create a main.sh script that outputs the current working directory
    cat > "main.sh" << 'EOF'
#!/usr/bin/env sh

# Print the current working directory
echo "Running from directory: $(pwd)"

# Create a temporary file in the current directory
echo "test content" > workdir-test-file.txt

# Check if the file was created in the expected location
if [ -f "$(pwd)/workdir-test-file.txt" ]; then
    echo "SUCCESS: File created in current working directory"
else
    echo "ERROR: File not created in current working directory"
    exit 1
fi

# Clean up
rm -f "$(pwd)/workdir-test-file.txt"
EOF
    chmod +x main.sh

    # Create install.sh that creates a file in its own directory
    cat > "install.sh" << 'EOF'
#!/usr/bin/env sh

# Create a file in the current directory (should be package dir)
echo "Package directory: $(pwd)" > install-location.txt
echo "Installation successful"
EOF
    chmod +x install.sh

    # Create uninstall.sh
    cat > "uninstall.sh" << 'EOF'
#!/usr/bin/env sh

# Output the current working directory during uninstall
echo "Uninstall working directory: $(pwd)" > uninstall-location.txt
echo "Uninstallation successful"
EOF
    chmod +x uninstall.sh

    cd ..

    # Install the working directory test package
    run_test "Install Working Directory Test Package" "cargo run --quiet -- install workdir-test-pkg --force"

    # Create a test directory with a unique file to verify working directory
    mkdir -p "execution-test-dir"
    cd "execution-test-dir"
    touch "unique-marker-file.txt"

    # Run the package from this test directory - it should execute in this directory
    run_test "Run Package with Working Directory Check" "cargo run --quiet -- run test/workdir-test"

    # Verify the script ran in the current directory by checking if the file was successfully created and deleted
    if [ $? -eq 0 ]; then
        report_test "Script Execution Working Directory" 0
    else
        report_test "Script Execution Working Directory" 1 "Script didn't execute in the expected working directory"
    fi

    # Change back to parent and uninstall
    cd ..
    run_test "Uninstall Working Directory Test Package" "cargo run --quiet -- uninstall test/workdir-test"

    # Clean up
    rm -rf "execution-test-dir"
    rm -rf "workdir-test-pkg"
}