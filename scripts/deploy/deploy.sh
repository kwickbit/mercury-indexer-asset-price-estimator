#!/bin/bash

# Load environment variables and set $BASE_DIR
source "$(dirname "$0")/deploy_config.sh"

# Default to deploying to both networks
deploy_testnet=true
deploy_mainnet=true
reset_mode=false
force_mode=false

# Parse command line arguments
for arg in "$@"
do
    case $arg in
        --test) deploy_mainnet=false ;;
        --main) deploy_testnet=false ;;
        --reset) reset_mode=true ;;
        --force) force_mode=true ;;
    esac
done

# Set the working directory based on reset mode - useful during development
# The basic version of the code does nothing but log the current sequence number.
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
deployment_success=true
if [ "$deploy_testnet" = true ]; then
    echo "Deploying to testnet..."
    if ! backup_and_deploy "test"; then
        echo -e "\e[30;41mDeployment to testnet failed\e[0m"
        deployment_success=false
    fi
fi

if [ "$deploy_mainnet" = true ]; then
    echo "Deploying to mainnet..."
    if ! backup_and_deploy "main"; then
        echo -e "\e[30;41mDeployment to mainnet failed\e[0m"
        deployment_success=false
    fi
fi

# There have been instances of false positives on `deployment_success`.
# The logs from the CLI itself are reliable.
cd "$BASE_DIR/indexer"
if [ "$deployment_success" = true ]; then
echo -e "\e[30;47mDeployment completed at $(date)\e[0m"
fi
