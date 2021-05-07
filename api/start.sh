#!/bin/sh

# Start the dev server

set -ex

poetry install
./manage.py migrate
./manage.py runserver 0.0.0.0:8000
