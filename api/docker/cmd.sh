#!/bin/sh

set -ex

diesel migration run
./gdlk_api
