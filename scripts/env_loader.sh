#!/bin/bash

load_env_vars() {
    local base_dir="$HOME/code/kwickbit"
    local env_file="$base_dir/indexer/.env"

    if [ -f "$env_file" ]; then
        source "$env_file"
    else
        echo "Error: .env file not found at $env_file" >&2
        return 1
    fi
}

load_env_vars