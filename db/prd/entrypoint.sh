#!/bin/sh

# We expect these env variables:
# - POSTGRES_DB
# - POSTGRES_USER
# - POSTGRES_ROOT_PASSWORD_FILE
# - POSTGRES_APP_USER
# - POSTGRES_APP_PASSWORD_FILE

# This var gets read by the built-in postgres entrypoing script
export POSTGRES_PASSWORD=$(cat $POSTGRES_ROOT_PASSWORD_FILE)
exec /usr/local/bin/docker-entrypoint.sh "$@"
