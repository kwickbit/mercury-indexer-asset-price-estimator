#!/bin/bash

BASE_DIR="$HOME/code/kwickbit"

load_env_vars() {
    local env_file="$BASE_DIR/indexer/.env"

    if [ -f "$env_file" ]; then
        source "$env_file"
    else
        echo "Error: .env file not found at $env_file" >&2
        return 1
    fi
}

load_env_vars || exit 1