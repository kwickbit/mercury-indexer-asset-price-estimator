#!/bin/bash

# Force mode can be triggered either by an argument to the script,
# which applies to all tables, or by a setting in a specific table
# in the zephyr.toml file. Here we check for the latter.
check_force_in_toml() {
    local toml_file="zephyr.toml"
    if [ -f "$toml_file" ]; then
        grep -qE '^\s*force\s*=\s*true\s*$' "$toml_file"
        return $?
    fi
    return 1
}