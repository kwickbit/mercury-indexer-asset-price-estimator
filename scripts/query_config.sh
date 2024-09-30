#!/bin/bash

# Load environment variables
source "$(dirname "$0")/env_loader.sh"

# Check if MAINNET_JWT is set
if [ -z "$MAINNET_JWT" ]; then
    echo "Error: MAINNET_JWT is not set in .env file" >&2
    exit 1
fi

# Help message
HELP_MESSAGE="Usage: query [options] <command> [arguments]

Commands:
  all                     Get all exchange rates
  asset <asset_symbol>    Get exchange rate for a specific asset

Options:
  --raw                   Output raw JSON (don't pipe to jq)
  --help, -h              Display this help message

Examples:
  query all
  query asset BTC
  query --raw all
  query --raw asset LTC
  query --help"