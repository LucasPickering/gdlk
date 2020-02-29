#!/bin/bash
pushd ../wasm
wasm-pack build
popd
npm install
npm start | cat # pipe into cat so console does not get reset
