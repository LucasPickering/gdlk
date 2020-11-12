# Build the server distributable
FROM gcr.io/gdlkit/gdlk-api:latest as rust-builder
# We need core/, api/, and a bunch of other files, so just copy everything in
COPY . /app
RUN cargo build --release

# Build our actual image
FROM debian:buster-slim
WORKDIR /app/api
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    && \
    rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /usr/local/cargo/bin/diesel /usr/local/bin/
COPY --from=rust-builder /app/target/release/gdlk_api .

ADD ./crates/api/migrations ./migrations/
ADD ./crates/api/config/default.json ./config/
ADD ./crates/api/docker/ /app/

CMD ["/app/cmd.sh"]
