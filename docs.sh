#!/bin/bash

# In most cases you can just use `cargo watch -x doc --open`
# This is useful for remote development
# Requires the http-server npm package (npm i -g http-server)

trap 'echo killing cargo $CARGO_PID; kill $CARGO_PID; exit' INT
cargo watch -x doc &
CARGO_PID=$!
echo "Cargo PID: $CARGO_PID"
http-server target/doc/
