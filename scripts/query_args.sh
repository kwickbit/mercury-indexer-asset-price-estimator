#!/bin/bash

parse_arguments() {
    local raw_output=false
    local command=""
    local asset_code=""
    local asset_issuer=""
    local date=""
    local cat_text=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            --raw)
                raw_output=true
                shift
                ;;
            --help|-h)
                echo "$HELP_MESSAGE"
                return 1
                ;;
            all)
                command="all"
                shift
                ;;
            asset)
                command="asset"
                shift
                if [[ $# -gt 0 ]]; then
                    if [[ "$1" =~ ^[a-z]+$ ]]; then
                        asset_code=${1^^}
                    else
                        asset_code=$1
                    fi
                    shift
                    # Check for optional asset_issuer argument
                    if [[ $# -gt 0 && "$1" =~ ^G[A-Z0-9]{55}$ ]]; then
                        asset_issuer=$1
                        shift
                    fi
                    # Check for optional date argument
                    if [[ $# -gt 0 && "$1" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}$ ]]; then
                        date=$1
                        shift
                    fi
                else
                    echo "Error: Asset symbol required for 'asset' command" >&2
                    return 1
                fi
                ;;
            cat)
                command="cat"
                shift
                if [[ $# -gt 0 ]]; then
                    cat_text=$1
                    shift
                fi
                ;;
            savepoint)
                command="savepoint"
                shift
                ;;
            *)
                echo "Error: Unknown argument '$1'" >&2
                echo "$HELP_MESSAGE" >&2
                return 1
                ;;
        esac
    done

    if [[ -z $command ]]; then
        echo "$HELP_MESSAGE" >&2
        return 1
    fi

    echo "$raw_output@$command@$asset_code@$asset_issuer@$date@$cat_text"
    return 0
}