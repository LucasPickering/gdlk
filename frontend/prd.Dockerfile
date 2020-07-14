# Build the frontend static assets
FROM gcr.io/gdlkit/gdlk-frontend:latest as frontend-builder
COPY . /app
ENV NODE_ENV production
RUN npm run build

# Put all the static assets in an image
FROM alpine:latest
WORKDIR /app/static
COPY --from=frontend-builder /app/frontend/build .
