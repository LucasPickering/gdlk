#!/bin/sh

set -ex

dockerize -wait tcp://db:5432

# Run seed data, just to make sure it's working
diesel database reset
for seed_file in ./seeds/*.sql; do
    echo "Executing $seed_file..."
    psql "$DATABASE_URL" < $seed_file
done

# Run tests
diesel database reset
cargo test -- --test-threads=1
