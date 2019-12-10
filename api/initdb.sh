#!/bin/sh

# "migration redo" fails on the first run, in that case fall back to doing a
# normal "migration run"
docker exec gdlk_api_1 sh -c "diesel migration redo || diesel migration run"
docker exec -i gdlk_db_1 psql gdlk < ./fixtures/init.pg
echo "Done"
