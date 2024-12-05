#! /bin/bash

# Back up the code before deploying, so we know what code is currently running.
backup_and_deploy() {
    local network=$1
    local deploy_dir="$BASE_DIR/indexer/deploy/$network"

    mkdir -p "$deploy_dir"
    rm -rf "$deploy_dir"/*
    cp -R "$BASE_DIR/indexer/src"/* "$deploy_dir"

    if [ "$network" = "test" ]; then
        deploy_to_network "testnet"
    else
        deploy_to_network "mainnet"
    fi

    return $?
}

# Function to deploy to a specific network
deploy_to_network() {
    local network=$1
    local jwt_var="${network^^}_JWT"

    # Warn and confirm in case of force mode
    if [ "$force_mode" = true ] || check_force_in_toml; then
        echo -e "\e[1;31mWarning: Force mode will destroy all data in the DB.\e[0m"
        read -p "Type 'force' to confirm: " confirmation
        if [ "$confirmation" != "force" ]; then
            echo "Deployment aborted."
            exit 1
        fi

        # Double warn and confirm when forcing to the mainnet
        if [ "$network" = "mainnet" ]; then
            spinner=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
            for i in {1..30}; do
                printf "\r(making sure you've thought this through)... ${spinner[i % 10]} "
                sleep 0.25
            done
            printf "\n"

            confirmation_text="YES, I have double-checked and force-deploying to main is indeed what I want."
            read -p "Type '${confirmation_text}': " final_confirmation
            if [ "$final_confirmation" != "$confirmation_text" ]; then
                echo "Deployment aborted."
                exit 1
            fi
        fi
    fi

    # Call the CLI to actually deploy
    mercury-cli --jwt "${!jwt_var}" --local false --mainnet "$([[ $network == "mainnet" ]] && echo "true" || echo "false")" deploy $([[ $force_mode == true ]] && echo "--force true")
    return $?
}