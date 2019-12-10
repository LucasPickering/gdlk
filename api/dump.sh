#!/bin/sh

docker exec -it gdlk_db_1 pg_dump gdlk --data-only --exclude-table=__diesel_schema_migrations > ./fixtures/init.pg
