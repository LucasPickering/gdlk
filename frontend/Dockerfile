FROM node:12.6.0-slim

# This should be built with the repo root dir as the context
RUN apt-get update && \
    apt-get install -y \
    curl build-essential
# RUN apk update && apk upgrade && \
#     apk add --no-cache curl

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

COPY rust-toolchain /app/
COPY ./wasm /app/wasm/
WORKDIR /app/wasm
# RUN cargo build
RUN wasm-pack build

COPY ./frontend /app/frontend

WORKDIR /app/frontend
RUN npm install

