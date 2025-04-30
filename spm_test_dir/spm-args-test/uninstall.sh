#!/usr/bin/env sh

echo "Uninstalling the package..."

# Run uninstall scripts for dependencies if any
if [ -d "./dependencies" ]; then
    for dep in ./dependencies/*; do
        if [ -f "$dep/uninstall.sh" ]; then
            echo "Uninstalling dependency: $(basename "$dep")"
            (cd "$dep" && ./uninstall.sh)
        fi
    done
fi