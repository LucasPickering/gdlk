#!/bin/sh

set -eu

database=$POSTGRES_DB
user=$POSTGRES_APP_USER
password=$(cat $POSTGRES_APP_PASSWORD_FILE)

# Create the new user, then, set up restricted permissions for them.
# They should have full read/write access on all tables, but can't make schema changes.
echo "Creating user '$user' on database '$database'"
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" $database <<-EOSQL
    CREATE USER $user WITH PASSWORD '$password';
    REVOKE CONNECT ON DATABASE $database FROM PUBLIC;
    GRANT CONNECT ON DATABASE $database TO $user;
    REVOKE ALL ON SCHEMA public FROM PUBLIC;
    GRANT USAGE ON SCHEMA public TO PUBLIC;
    ALTER DEFAULT PRIVILEGES
        IN SCHEMA public
        GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO $user;
EOSQL
echo "Done creating user"
