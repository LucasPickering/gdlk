#!/bin/bash

set -e

# Env variables we expect as input:
# - GDLK_DB_HOST
# - GDLK_DB_NAME
# - GDLK_ROOT_DB_USER
# - GDLK_ROOT_DB_PASSWORD_FILE
# - GDLK_APP_DB_USER
# - GDLK_APP_DB_PASSWORD_FILE
# - GDLK_SECRET_KEY_FILE
# - GDLK_GOOGLE_CLIENT_ID_FILE
# - GDLK_GOOGLE_CLIENT_SECRET_FILE

# We use the DB superuser for migrations so that we can do extension and table alertations
DATABASE_URL="postgres://${GDLK_ROOT_DB_USER}:$(cat $GDLK_ROOT_DB_PASSWORD_FILE)@${GDLK_DB_HOST}/${GDLK_DB_NAME}?connect_timeout=10" \
    diesel migration run

# We use an unprivileged user for the app cause security
GDLK_DATABASE_URL="postgres://${GDLK_APP_DB_USER}:$(cat $GDLK_APP_DB_PASSWORD_FILE)@${GDLK_DB_HOST}/${GDLK_DB_NAME}?connect_timeout=10" \
GDLK_SECRET_KEY=$(cat $GDLK_SECRET_KEY_FILE) \
GDLK_OPEN_ID__PROVIDERS__GOOGLE__CLIENT_ID=$(cat $GDLK_GOOGLE_CLIENT_ID_FILE) \
GDLK_OPEN_ID__PROVIDERS__GOOGLE__CLIENT_SECRET=$(cat $GDLK_GOOGLE_CLIENT_SECRET_FILE) \
    ./gdlk_api
