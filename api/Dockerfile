FROM rustlang/rust:nightly-slim

RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    libssl-dev \
    pkg-config \
    && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install cargo-watch && \
    cargo install diesel_cli --no-default-features --features=postgres
WORKDIR /app/api
