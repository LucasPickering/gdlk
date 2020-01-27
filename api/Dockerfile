FROM rustlang/rust:nightly-slim

# This should be built with the repo root dir as the context

ENV DOCKERIZE_VERSION v0.6.1

# Need to do this first so we pick up the rust-toolchain file
COPY . /app
WORKDIR /app

RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    wget \
    && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-watch && \
    cargo install diesel_cli --no-default-features --features=postgres && \
    wget https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz && \
    tar -C /usr/local/bin -xzvf dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz && \
    rm dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz

WORKDIR /app/api
