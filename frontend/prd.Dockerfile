# Build the frontend static assets
FROM gcr.io/gdlkit/gdlk-frontend:latest as frontend-builder
ENV NODE_ENV production
COPY . /app
WORKDIR /app/wasm
RUN wasm-pack build
WORKDIR /app/frontend
RUN npm install && npm run build

# Put all the static assets in an image
FROM alpine:latest
WORKDIR /app/static
COPY --from=frontend-builder /app/frontend/build .
