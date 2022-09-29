#!/bin/sh

set -ex

export VIRTUAL_ENV=$(poetry env info --path)

# Compile rust crate in the backgrouns
# Make sure container is run w/ --init so this dies correctly
(cd ../crates/pyo3 && cargo watch -s "maturin develop") &

poetry install
./m.sh runserver 0.0.0.0:8000
