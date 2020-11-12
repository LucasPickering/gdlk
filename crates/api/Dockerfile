# Version doesn't matter here, we're just going to install our own toolchain
FROM rust:1.41.1-slim

RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    postgresql \
    && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app/crates/api
COPY rust-toolchain ../
RUN cargo install cargo-make --version 0.32.0 && \
    cargo install diesel_cli --version 1.4.1 --no-default-features --features=postgres && \
    cargo install cargo-watch --version 7.5.0
