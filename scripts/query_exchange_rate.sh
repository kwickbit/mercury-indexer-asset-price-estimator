#!/bin/bash

source "$(dirname "$0")/query_config.sh"

# Parse arguments
result=$(parse_arguments "$@")
exit_code=$?

if [ $exit_code -ne 0 ]; then
    exit $exit_code
fi

IFS=':' read -r raw_output command asset date <<< "$result"

if [ "$command" = "all" ]; then
    fname="get_all_exchange_rates"
    arguments="{}"
elif [ "$command" = "asset" ]; then
    fname="get_exchange_rate"
    arguments="{\\\"asset\\\": \\\"$asset\\\""
    [ -n "$date" ] && arguments="$arguments, \\\"date\\\": \\\"$date\\\""
    arguments="$arguments}"
fi

# Set the QUERY variable using a template
QUERY="{\"project_name\": \"indexer\", \"mode\": {\"Function\": {\"fname\": \"$fname\", \"arguments\": \"$arguments\"}}}"

# Call the API, suppressing progress output from `curl`
curl -s -X POST https://mainnet.mercurydata.app/zephyr/execute \
     -H "Authorization: Bearer $MAINNET_JWT" \
     -H 'Content-Type: application/json' \
     -d "$QUERY" | if [ "$raw_output" = true ]; then cat; else jq; fi