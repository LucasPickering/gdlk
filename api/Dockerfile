# We use slim instead of alpine so we can install cryptography from a wheel
# instead of needing the rust compiler
FROM python:3.9-slim

ENV PYTHONUNBUFFERED=1

RUN apt-get update && apt-get install -y \
    gcc \
    openssl \
    libpq-dev
# RUN apk add --no-cache \
#     gcc \
# libmagic \
# libffi-dev \
# linux-headers \
# musl-dev \
# openssl \
# postgresql-dev \
# zlib-dev
RUN pip install poetry && \
    # Disable venv since we're isolated in a container anyway
    poetry config virtualenvs.create false

# Don't install project dependencies yet, we'll do that on container start.
# That way we don't have to re-build the image when changing dependencies in dev.

WORKDIR /app/api
