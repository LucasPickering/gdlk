#!/bin/sh

psql template1 -U root -c 'CREATE EXTENSION IF NOT EXISTS "uuid-ossp";'
