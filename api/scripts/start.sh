#!/bin/sh

# Start the dev server
# If running this from docker-compose, make sure you include `init: true`
# on the container, otherwise it won't die gracefully because of background processes

set -ex

poetry install
./manage.py migrate
./manage.py graphql_schema --watch &
./manage.py runserver 0.0.0.0:8000
