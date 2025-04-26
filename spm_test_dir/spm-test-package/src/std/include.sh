#!/usr/bin/env sh

# Shell Package Manager include function
# This function allows importing dependencies and other shell scripts

# Global variable to track already included files to avoid duplicate inclusions
if [ -z "$SPM_INCLUDED_FILES" ]; then
    export SPM_INCLUDED_FILES=""
fi

# Include a dependency or local file
# Usage: include "dependency_name" [file_path]
# Examples:
#   include "logger"                # Include main entry point of logger dependency
#   include "logger" "src/utils.sh" # Include specific file from logger dependency
#   include "./src/helpers.sh"      # Include local file
include() {
    local dependency=""
    local file_path=""
    
    # Check if this is a dependency include or a local file include
    if [ $# -eq 1 ]; then
        # Single argument could be a dependency name or a local file
        if [ -d "./dependencies/$1" ]; then
            # It's a dependency - use its main entry point
            dependency="$1"
            local package_json="./dependencies/$dependency/package.json"
            if [ -f "$package_json" ]; then
                # Extract the entrypoint from package.json
                if command -v jq >/dev/null 2>&1; then
                    file_path="./dependencies/$dependency/$(jq -r '.entrypoint' "$package_json")"
                else
                    # Fallback if jq is not available
                    file_path="./dependencies/$dependency/main.sh"
                    if [ ! -f "$file_path" ]; then
                        file_path="./dependencies/$dependency/lib.sh"
                    fi
                fi
            else
                file_path="./dependencies/$dependency/main.sh"
                if [ ! -f "$file_path" ]; then
                    file_path="./dependencies/$dependency/lib.sh"
                fi
            fi
        else
            # It's a local file
            file_path="$1"
        fi
    elif [ $# -eq 2 ]; then
        # Two arguments: dependency and specific file
        dependency="$1"
        file_path="./dependencies/$dependency/$2"
    else
        echo "Error: include requires 1 or 2 arguments" >&2
        return 1
    fi
    
    # Convert to absolute path to ensure uniqueness check works
    local abs_path="$(cd "$(dirname "$file_path")" 2>/dev/null && pwd)/$(basename "$file_path")"
    
    # Check if the file has already been included
    if echo "$SPM_INCLUDED_FILES" | grep -q "$abs_path"; then
        return 0
    fi
    
    # Check if the file exists
    if [ ! -f "$file_path" ]; then
        echo "Error: Cannot include '$file_path' - file not found" >&2
        return 1
    fi
    
    # Add to the list of included files
    SPM_INCLUDED_FILES="$SPM_INCLUDED_FILES:$abs_path"
    
    # Source the file
    . "$file_path"
    return $?
}