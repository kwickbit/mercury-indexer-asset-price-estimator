#!/bin/bash

source "$(dirname "$0")/env_loader.sh"

# Check if MAINNET_JWT is set
if [ -z "$MAINNET_JWT" ]; then
    echo "Error: MAINNET_JWT is not set in .env file" >&2
    exit 1
fi

# Parse command line arguments
if [ "$1" = "--all" ]; then
    fname="get_all_exchange_rates"
    arguments="{}"
elif [ $# -eq 0 ]; then
    echo "Usage: \`query <asset>\` or \`query --all"\` >&2
    exit 1
else
    fname="get_exchange_rate"
    # Make the argument uppercase if it is all-lowercase
    if [[ "$1" =~ ^[a-z]+$ ]]; then
        asset=${1^^}
    else
        asset=$1
    fi
    arguments="{\\\"asset\\\": \\\"$asset\\\"}"
fi

# Set the QUERY variable using a template
QUERY="{\"project_name\": \"indexer\", \"mode\": {\"Function\": {\"fname\": \"$fname\", \"arguments\": \"$arguments\"}}}"

# Call the API, suppressing progress output from `curl`
curl -s -X POST https://mainnet.mercurydata.app/zephyr/execute \
     -H "Authorization: Bearer $MAINNET_JWT" \
     -H 'Content-Type: application/json' \
     -d "$QUERY" | jq