#!/bin/sh

set -e

echo "Filling site template confs..."
envsubst '${GDLK_HOSTNAME}' < /app/nginx.conf > /etc/nginx/nginx.conf

nginx -g "daemon off;"
