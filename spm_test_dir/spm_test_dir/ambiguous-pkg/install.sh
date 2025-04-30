#!/usr/bin/env sh

echo "Setting up the package..."

# Install dependencies if any
if [ -d "./dependencies" ]; then
    for dep in ./dependencies/*; do
        if [ -f "$dep/install.sh" ]; then
            echo "Installing dependency: $(basename "$dep")"
            (cd "$dep" && ./install.sh)
        fi
    done
fi