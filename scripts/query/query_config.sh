#!/bin/bash

# Load environment variables
source "$(dirname "$0")/env_loader.sh"
source "$(dirname "$0")/query_args.sh"
source "$(dirname "$0")/build_query.sh"

# Check if MAINNET_JWT is set
if [ -z "$MAINNET_JWT" ]; then
    echo "Error: MAINNET_JWT is not set in .env file" >&2
    exit 1
fi

# Help message
HELP_MESSAGE="Usage: query [options] <command> [arguments]

Commands:
  all                     Get all exchange rates
  asset <asset_symbol> [asset_issuer] [datetime]    Get exchange rate for a specific asset, optionally with issuer and/or at a specific time

Options:
  --raw                   Output raw JSON (don't pipe to jq)
  --help, -h              Display this help message

Examples:
  query asset XLM
  query asset USDC GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN
  query asset BTC 2018-01-01T14:30:00
  query --raw asset ETH GBDEVU63Y6NTHJQQZIKVTC23NWLQVP3WJ2RI2OTSJTNYOIGICST6DUXR 2020-16-09T16:09:00
  query --help"