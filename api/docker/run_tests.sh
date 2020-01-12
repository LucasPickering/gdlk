#!/bin/sh

set -ex

dockerize -wait tcp://db:5432
diesel migration run
cargo test -p gdlk_api
