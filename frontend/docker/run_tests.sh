#!/bin/sh

set -ex

npm install
npm run wasm-pack
npm run relay
npm run tsc
npm run lint
npm run lint:gql
# We don't actually have any tests yet
# CI=true npm run test
