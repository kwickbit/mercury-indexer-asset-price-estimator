#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# Go up one level to get the project root
export BASE_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

load_env_vars() {
    local env_file="$BASE_DIR/.env"

    if [ -f "$env_file" ]; then
        source "$env_file"
    else
        echo "Error: .env file not found at $env_file" >&2
        return 1
    fi
}

load_env_vars || exit 1