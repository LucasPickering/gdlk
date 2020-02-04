FROM rustlang/rust:nightly-slim
ARG install_cargo_watch=0

# This should be built with the repo root dir as the context

ENV DOCKERIZE_VERSION v0.6.1

RUN apt-get update && \
    apt-get install -y \
    libpq-dev \
    pkg-config \
    && \
    rm -rf /var/lib/apt/lists/*

ADD https://github.com/jwilder/dockerize/releases/download/$DOCKERIZE_VERSION/dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz .
RUN tar -C /usr/local/bin -xzvf dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz && \
    rm dockerize-alpine-linux-amd64-$DOCKERIZE_VERSION.tar.gz

# Need to do this first so we pick up the rust-toolchain file
COPY . /app
WORKDIR /app
RUN cargo install diesel_cli --no-default-features --features=postgres && \
    cargo build && \
    if [ $install_cargo_watch = 1 ]; then cargo install cargo-watch; fi

WORKDIR /app/api
