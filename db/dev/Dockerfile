# Create a sanitized DB image for use in development. "Sanitized" means it
# contains no sensitive data, so nothing pertaining to any real users.
# We do this in 2 stages:
# 1. Load in production data and sanitize it
# 2. Copy the sanitized data into a clean image
#
# IMPORTANT - before building this, you'll need to download the backup file
# yourself. Try:
# gsutil cp gs://<backup bucket>/backups.tar.gz db/dev

FROM postgres:12-alpine AS builder

ARG backup_archive=backups.tar.gz
ARG backup_db_file=backups/gdlk.sql
ARG db_name=gdlk

WORKDIR /app
# This will extract the archive to ${backup_db_file}
ADD sanitize.sh ${backup_archive} ./

RUN POSTGRES_DB=${db_name} \
    BACKUP_DB_FILE=${backup_db_file} \
    ./sanitize.sh

# Stage 2 - starting from a clean image, copy in the sanitary data. We need to
# make sure the sensitive data never touches this image, otherwise it will get
# captured in one of the layers.
FROM postgres:12-alpine

ARG db_name=gdlk
ENV POSTGRES_DB=${db_name}
ENV POSTGRES_USER=root
ENV POSTGRES_PASSWORD=root

# The pg image will automatically load the file from this dir on first startup
COPY --from=builder /app/sanitized.sql /docker-entrypoint-initdb.d/
