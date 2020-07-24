#!/bin/bash

set -e

# Read the secret value directly from the prompt
function read_value {
    name=$1
    echo $(read -p "${name} = " value)
}

# Read a file name from the prompt, then read the secret from that file
function read_value_from_file {
    name=$1
    read -p "${name} FILE = " file
    cat $file
}

function generate_hex_secret {
    echo $(head -c 16 /dev/urandom | hexdump -e '"02%x"')
}

function generate_b64_secret {
    echo $(head -c 64 /dev/urandom | base64 -w0)
}

function set_secret {
    name=$1
    cmd=$2

    if docker secret inspect $1 > /dev/null 2>&1; then
        echo "$name already set, skipping"
        return
    fi

    # Run the given command to get our value
    value=$($cmd $name)

    echo $value | docker secret create $name -
    echo -e "\e[32m${name} = ${value}\e[0m"
    echo
}

set_secret gdlk_db_root_password generate_hex_secret
set_secret gdlk_db_app_password generate_hex_secret
set_secret gdlk_cloud_storage_bucket read_value
set_secret gdlk_cloud_storage_key read_value_from_file
set_secret gdlk_api_secret_key generate_b64_secret
set_secret gdlk_api_open_id__providers__google__client_id read_value
set_secret gdlk_api_open_id__providers__google__client_secret read_value
