# Wheels don't work on alpine which makes builds super slow, so we use debian
# https://discuss.python.org/t/wheels-for-musl-alpine/7084
FROM python:3.9-slim

ENV PATH="/root/.cargo/bin:${PATH}" \
    PYTHONUNBUFFERED=1

# Dependencies first, for layer caching
RUN apt-get update && \
    apt-get install -y \
    curl \
    gcc \
    git \
    openssl \
    libpq-dev \
    && \
    rm -rf /var/lib/apt/lists/*
# maturin needed for pyo3 crate
RUN pip install maturin poetry

# Install rustup/rust toolchain
WORKDIR /app/crates
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ADD ./crates/rust-toolchain.toml ./
RUN rustup show && cargo install cargo-watch

WORKDIR /app/api
ADD ./api/pyproject.toml ./api/poetry.lock ./
