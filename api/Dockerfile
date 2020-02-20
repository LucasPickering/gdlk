# Version doesn't matter here, we're just going to install our own toolchain
FROM rust:1.41.1-slim

ENV DOCKERIZE_VERSION v0.6.1

RUN apt-get update && \
    apt-get install -y \
    curl \
    libpq-dev \
    pkg-config \
    && \
    rm -rf /var/lib/apt/lists/*

RUN curl -sSL https://github.com/jwilder/dockerize/releases/download/${DOCKERIZE_VERSION}/dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz \
    | tar xzv -C /usr/local/bin

WORKDIR /app
COPY rust-toolchain ./
RUN cargo install diesel_cli --no-default-features --features=postgres && \
    cargo install cargo-watch

WORKDIR /app/api
