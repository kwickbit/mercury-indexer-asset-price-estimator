#!/bin/bash

# By default, deployments don't change the DB schema. When they do, all data is
# dropped. The script takes a --force argument that forces everything, but we
# can be more granular via the zephyr.toml file.
check_force_in_toml() {
    local toml_file="zephyr.toml"
    if [ -f "$toml_file" ]; then
        grep -qE '^\s*force\s*=\s*true\s*$' "$toml_file"
        return $?
    fi
    return 1
}