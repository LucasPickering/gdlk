#!/bin/sh

set -ex

export VIRTUAL_ENV=$(poetry env info --path)

(cd ../crates/pyo3 && maturin develop)
poetry install
./m.sh runserver 0.0.0.0:8000
