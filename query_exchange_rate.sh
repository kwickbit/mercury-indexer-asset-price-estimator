#!/bin/bash

# Load environment variables
if [ -f .env ]; then
    source .env
else
    echo "Error: .env file not found"
    exit 1
fi

# Check if MAINNET_JWT is set
if [ -z "$MAINNET_JWT" ]; then
    echo "Error: MAINNET_JWT is not set in .env file"
    exit 1
fi

# Check if an argument is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <asset>"
    exit 1
fi

# Make the argument uppercase if it is all-lowercase
if [[ "$1" =~ ^[a-z]+$ ]]; then
    asset=${1^^}
else
    asset=$1
fi

# Set the QUERY variable with the provided asset
QUERY="{\"project_name\": \"indexer\", \"mode\": {\"Function\": {\"fname\": \"get_exchange_rate\", \"arguments\": \"{\\\"asset\\\": \\\"$asset\\\"}\"}}}"

# Execute the curl command
curl -X POST https://mainnet.mercurydata.app/zephyr/execute \
     -H "Authorization: Bearer $MAINNET_JWT" \
     -H 'Content-Type: application/json' \
     -d "$QUERY" | jq