#!/bin/sh

set -ex

cargo make -p docker seed
cargo make -p docker test
