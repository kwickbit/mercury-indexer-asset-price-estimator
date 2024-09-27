#!/bin/bash

source "$(dirname "$0")/env_loader.sh"

# Check if MAINNET_JWT is set
if [ -z "$MAINNET_JWT" ]; then
    echo "Error: MAINNET_JWT is not set in .env file" >&2
    exit 1
fi

# Initialize raw_output flag
raw_output=false
command=""

# Parse command line arguments
for arg in "$@"; do
    case $arg in
        --raw) raw_output=true ;;
        all) command="all" ;;
        *) command=$arg ;;
    esac
done

if [ "$command" = "all" ]; then
    fname="get_all_exchange_rates"
    arguments="{}"
elif [ -z "$command" ]; then
    echo "Usage: \`query <asset>\` or \`query all\` or \`query --raw <asset>\` or \`query --raw all\`" >&2
    exit 1
else
    fname="get_exchange_rate"
    # Make the argument uppercase if it is all-lowercase
    if [[ "$command" =~ ^[a-z]+$ ]]; then
        asset=${command^^}
    else
        asset=$command
    fi
    arguments="{\\\"asset\\\": \\\"$asset\\\"}"
fi

# Set the QUERY variable using a template
QUERY="{\"project_name\": \"indexer\", \"mode\": {\"Function\": {\"fname\": \"$fname\", \"arguments\": \"$arguments\"}}}"

# Call the API, suppressing progress output from `curl`
curl -s -X POST https://mainnet.mercurydata.app/zephyr/execute \
     -H "Authorization: Bearer $MAINNET_JWT" \
     -H 'Content-Type: application/json' \
     -d "$QUERY" | if [ "$raw_output" = true ]; then cat; else jq; fi