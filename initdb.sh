#!/bin/sh

docker exec gdlk_api_1 diesel migration redo
docker exec gdlk_db_1 psql gdlk --file=/app/fixtures/env.sql
