#!/bin/bash

PREFIX=keskne_ source /app/load_secrets.sh
export DATABASE_URL="postgres://${GDLK_DB_USER}:${GDLK_DB_PASSWORD}@${GDLK_DB_HOST}/${GDLK_DB_NAME}?connect_timeout=10"
exec $@
