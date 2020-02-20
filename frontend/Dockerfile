# If you update this, make sure you update .nvmrc too
FROM node:12.16.0-slim

ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update && \
    apt-get install -y \
    build-essential \
    curl \
    && \
    rm -rf /var/lib/apt/lists/*

# Install rustup and wasm-pack
RUN curl -sSLf https://sh.rustup.rs | sh -s -- -y --default-toolchain none && \
    curl -sSLf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh

# Install correct rust toolchain
WORKDIR /app
COPY rust-toolchain ./
RUN sh -c "rustup toolchain install $(cat rust-toolchain)"

WORKDIR /app/frontend
