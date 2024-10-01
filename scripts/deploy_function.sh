#! /bin/bash

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

        if [ "$network" = "mainnet" ]; then
            spinner=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
            for i in {1..30}; do
                printf "\r(making sure you've thought this through)... ${spinner[i % 10]} "
                sleep 0.25
            done
            printf "\n"

            leeroy_confirmation="YES, I have double-checked and force-deploying to main is indeed what I want. Leeroy Jenkins!"
            read -p "Type '${leeroy_confirmation}': " final_confirmation
            if [ "$final_confirmation" != "$leeroy_confirmation" ]; then
                echo "Deployment aborted."
                exit 1
            fi
        fi
    fi

    mercury-cli --jwt "${!jwt_var}" --local false --mainnet "$([[ $network == "mainnet" ]] && echo "true" || echo "false")" deploy $([[ $force_mode == true ]] && echo "--force true")
    return $?
}