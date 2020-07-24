# Build the frontend static assets
FROM gcr.io/gdlkit/gdlk-frontend:latest as frontend-builder
ENV NODE_ENV production

COPY Cargo.toml Cargo.lock rust-toolchain /app/

# This is a hack to remove crates from Cargo.toml that we don't want in this
# image. Cargo refuses to do anything if these crates aren't in the filesystem,
# but pulling them in causes unnecessary rebuilds.
RUN sed -i -e '/api/d' -e '/cli/d' /app/Cargo.toml

# Copy in the stuff needed for the wasm build first - do this before the
# frontend stuff so that if we just make TS changes, we can avoid a Rust rebuild
COPY core/ /app/core/
COPY wasm/ /app/wasm/
WORKDIR /app/wasm
RUN wasm-pack build

COPY api/schema.graphql /app/api/schema.graphql
COPY frontend /app/frontend/
WORKDIR /app/frontend
RUN npm install && npm run build

# Put all the static assets in an nginx image
FROM nginx:alpine
COPY --from=frontend-builder /app/frontend/build /app/static
COPY nginx/entrypoint.sh nginx/cmd.sh nginx/nginx.conf /app/
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["/app/cmd.sh"]
