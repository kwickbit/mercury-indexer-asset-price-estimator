#!/bin/bash

# Base directory for the project
BASE_DIR="$HOME/code/kwickbit"

# Function to deploy to a specific network
deploy_to_network() {
    local network=$1
    local jwt_var="${network^^}_JWT"  # Convert to uppercase for variable name
    mercury-cli --jwt "${!jwt_var}" --local false --mainnet "$([[ $network == "mainnet" ]] && echo "true" || echo "false")" deploy $([[ $force_mode == true ]] && echo "--force true")
}

# Load environment variables
ENV_FILE="$BASE_DIR/indexer/.env"
if [ -f "$ENV_FILE" ]; then
    source "$ENV_FILE"
else
    echo "Error: .env file not found at $ENV_FILE"
    exit 1
fi

# Default to deploying to both networks
deploy_testnet=true
deploy_mainnet=true
reset_mode=false
force_mode=false

# Parse command line arguments
for arg in "$@"
do
    case $arg in
        --test)
        deploy_testnet=true
        deploy_mainnet=false
        ;;
        --main)
        deploy_testnet=false
        deploy_mainnet=true
        ;;
        --reset)
        reset_mode=true
        ;;
        --force)
        force_mode=true
        ;;
    esac
done

# Set the working directory based on reset mode
if [ "$reset_mode" = true ]; then
    cd "$BASE_DIR/basic" || exit 1
    echo -e "\e[1;93mWarning: Running in reset mode from the 'basic' directory.\e[0m"
else
    cd "$BASE_DIR/indexer" || exit 1
fi

# Warn user if deploying to both networks
if [ "$deploy_testnet" = true ] && [ "$deploy_mainnet" = true ]; then
    echo -e "\e[1;31mWarning: Deploying to both testnet and mainnet.\e[0m"
fi

# Deploy to selected networks
if [ "$deploy_testnet" = true ]; then
    echo "Deploying to testnet..."
    deploy_to_network "testnet"
fi

if [ "$deploy_mainnet" = true ]; then
    echo "Deploying to mainnet..."
    deploy_to_network "mainnet"
fi

cd "$BASE_DIR/indexer"
echo "Deployment completed."