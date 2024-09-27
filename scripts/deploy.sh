#!/bin/bash

# Load environment variables and set $BASE_DIR
source "$(dirname "$0")/env_loader.sh"

# Check if zephyr.toml specifies force mode for any tables
source "$BASE_DIR/indexer/scripts/check_force.sh"

# Function to deploy to a specific network
deploy_to_network() {
    local network=$1
    local jwt_var="${network^^}_JWT"

    if [ "$force_mode" = true ] || check_force_in_toml; then
        echo -e "\e[1;31mWarning: Force mode will destroy all data in the DB.\e[0m"
        read -p "Type 'force' to confirm: " confirmation
        if [ "$confirmation" != "force" ]; then
            echo "Deployment aborted."
            exit 1
        fi
    fi

    mercury-cli --jwt "${!jwt_var}" --local false --mainnet "$([[ $network == "mainnet" ]] && echo "true" || echo "false")" deploy $([[ $force_mode == true ]] && echo "--force true")
    return $?
}

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
deployment_success=true
if [ "$deploy_testnet" = true ]; then
    echo "Deploying to testnet..."
    if ! deploy_to_network "testnet"; then
        echo -e "\e[30;41mDeployment to testnet failed\e[0m"
        deployment_success=false
    fi
fi

if [ "$deploy_mainnet" = true ]; then
    echo "Deploying to mainnet..."
    if ! deploy_to_network "mainnet"; then
        echo -e "\e[30;41mDeployment to mainnet failed\e[0m"
        deployment_success=false
    fi
fi

cd "$BASE_DIR/indexer"
if [ "$deployment_success" = true ]; then
    echo -e "\e[30;47mDeployment completed at $(date)\e[0m"
fi
