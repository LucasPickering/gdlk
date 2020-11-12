# Build the frontend static assets
FROM gcr.io/gdlkit/gdlk-frontend:latest as frontend-builder
ENV NODE_ENV production

WORKDIR /app/
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock rust-toolchain ./

# Copy in the stuff needed for the wasm build first - do this before the
# frontend stuff so that if we just make TS changes, we can avoid a Rust rebuild
COPY crates/core/ ./crates/core/
COPY crates/wasm/ ./crates/wasm/
WORKDIR /app/crates/wasm
RUN wasm-pack build

# Rely on the .dockerignore file here to make sure we don't copy in too much
COPY crates/api/schema.graphql /app/crates/api/schema.graphql
COPY frontend /app/frontend/
WORKDIR /app/frontend
RUN npm install && npm run build

# Put all the static assets in an nginx image
FROM nginx:alpine
COPY --from=frontend-builder /app/frontend/build /app/static
COPY nginx/entrypoint.sh nginx/cmd.sh nginx/nginx.conf /app/
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["/app/cmd.sh"]
