#!/bin/bash

# Load environment variables and set $BASE_DIR
source "$(dirname "$0")/env_loader.sh"

# Check if zephyr.toml specifies force mode for any tables
source "$BASE_DIR/indexer/scripts/check_force.sh"

# Function to deploy to a specific network
source "$BASE_DIR/indexer/scripts/deploy_function.sh"