#!/bin/bash

source "$(dirname "$0")/query_config.sh"

# Parse arguments
result=$(parse_arguments "$@")
exit_code=$?

# The arguments might be invalid or the user might have called --help
if [ $exit_code -ne 0 ]; then
    [ "$1" = "--help" ] || [ "$1" = "-h" ] && echo "$HELP_MESSAGE" && exit 0
    exit $exit_code
fi

# Send the parsed arguments to build_query
IFS='@' read -r raw_output command asset_code asset_issuer date cat_text <<< "$result"
QUERY=$(build_query "$command" "$asset_code" "$asset_issuer" "$date" "$cat_text")

# Call the API, suppressing progress output from `curl`
curl -s -X POST https://mainnet.mercurydata.app/zephyr/execute \
     -H "Authorization: Bearer $MAINNET_JWT" \
     -H 'Content-Type: application/json' \
     -d "$QUERY" | if [ "$raw_output" = true ]; then cat; else jq; fi