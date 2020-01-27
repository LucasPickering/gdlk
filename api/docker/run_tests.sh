#!/bin/sh

set -ex

dockerize -wait tcp://db:5432
diesel migration run --locked-schema
cargo test -p gdlk_api
