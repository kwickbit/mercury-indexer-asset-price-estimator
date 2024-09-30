#!/bin/bash

parse_arguments() {
    local raw_output=false
    local command=""
    local asset=""

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
                        asset=${1^^}
                    else
                        asset=$1
                    fi
                    shift
                else
                    echo "Error: Asset symbol required for 'asset' command" >&2
                    return 1
                fi
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

    echo "$raw_output:$command:$asset"
    return 0
}