#!/bin/sh

set -ex

export POSTGRES_USER=root
export POSTGRES_PASSWORD=root

# Start the pg server
nohup sh -c "docker-entrypoint.sh postgres &"
sleep 10 # Highly technical

# This user is referenced in the prd DB, but we don't need it in dev. Create it
# just so the restore passes, then immediately delete it
echo 'Restoring DB...'
psql ${POSTGRES_DB} -c "CREATE USER gdlk;"
psql ${POSTGRES_DB} < ${BACKUP_DB_FILE}
psql ${POSTGRES_DB} -c "DROP OWNED BY gdlk; DROP USER gdlk;"

# VERY IMPORTANT PART - sanitize all sensitive data
# We assume that all sensitive data is related to users, so if we truncate the
# users table and cascade, we'll delete everything that directly or indirectly
# relates to users.
# It's possible we add non-user sensitive tables in the future. If so, we'll
# need to make sure we clean those up here too.
echo 'Sanitizing DB...'
psql ${POSTGRES_DB} -c "TRUNCATE users, user_providers RESTART IDENTITY CASCADE;"

echo 'Dumping sanitized DB...'
pg_dump ${POSTGRES_DB} > sanitized.sql
