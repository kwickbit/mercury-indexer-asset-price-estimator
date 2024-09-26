#!/bin/bash

check_force_in_toml() {
    local toml_file="zephyr.toml"
    if [ -f "$toml_file" ]; then
        grep -qE '^\s*force\s*=\s*true\s*$' "$toml_file"
        return $?
    fi
    return 1
}