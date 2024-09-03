#!/bin/bash

ENV_FILE="$HOME/code/kwickbit/indexer/.env"

# Load environment variables
if [ -f "$ENV_FILE" ]; then
    source "$ENV_FILE"
else
    echo "Error: .env file not found at $ENV_FILE"
    exit 1
fi

# Default to testnet
MAINNET="false"
JWT="$TESTNET_JWT"

# Check for --prod flag
if [ "$1" = "--prod" ]; then
    MAINNET="true"
    JWT="$MAINNET_JWT"
fi

# Execute the deployment command
mercury-cli --jwt "$JWT" --local false --mainnet "$MAINNET" deploy
