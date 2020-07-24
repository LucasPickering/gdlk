# GDLK

## About

GDLK is a programming language, as well as a game based on solving puzzles with the language. There is no usable product _yet_, but it's coming soon™!

### What does GDLK stand for?

GDLK Development Language Kit

#### Wait, but, what does GDLK stand for?

GDLK Development Language Kit!

## Development

### Setup

First, you'll need to install:

- `docker-compose`
- `rustup`

Then, for development you'll probably want to install more stuff:

```sh
rustup component add rustfmt-preview clippy-preview
cargo install diesel_cli --no-default-features --features postgres
cargo install cargo-make
```

### Compiling and Executing Locally

If you just want to compile and run a program without starting up the webserver, you can use the CLI for that. To execute a program, you will need:

- A hardware spec, which defines what hardware the program can access (JSON)
- A program spec, which defines the input and expected output (JSON)
- A source file (GDLK)

See `core/src/models.rs` for a list of fields that the hardware and program specs need. Then, you can run the program with:

```sh
cargo run -p gdlk_cli -- run --hardware hw.json --program prog.json -s prog.gdlk
```

### Running the Web Stack

In the repo root, run:

```sh
cd api && cargo make secrets && cd.. # Only needed on first execution
# Enter your username and the new token as your password
docker-compose up
```

Then, see the next section to initialize the DB.

### Migrations & Seed Data

Migrations are managed by Diesel. Seed data is defined via SQL scripts, in `api/seeds`.

```sh
cd api
# Migrations are automatically run on server startup, but if you need to run them manually:
cargo make diesel migration run

# You can load seed data with:
cargo make seed

# If you need to re-run migrations (this will wipe out your DB!)
cargo make diesel db reset
```

### Tests

You can run tests with:

```sh
cargo make test # In the root, or any sub-crate
```

Note: for API tests, you will need the database running. You can do this with either of these commands:

```sh
docker-compose up
# OR
docker-compose run --service-ports db
```

### Debugging

If you have a GDLK program or test failing, you can have the GDLK compiler and interpreter output additional debug information by setting `DEBUG=1`.

```sh
DEBUG=1 cargo run -p gdlk_cli -- --hardware hw.json --program prog.json -s prog.gdlk
DEBUG=1 cargo make test
```

### Nightly Rust

We use nightly Rust. Here's a list of reasons why. If this list every gets empty, we should switch to stable.

- Rust features
  - [or_patterns](https://github.com/rust-lang/rust/issues/54883)
- Rustfmt
  - [merge_imports](https://github.com/rust-lang/rustfmt/issues/3362)
  - [wrap_comments](https://github.com/rust-lang/rustfmt/issues/3347)

[Here's a helpful site for finding new nightly versions](https://rust-lang.github.io/rustup-components-history/).

## Deployment

This project is deployed through docker-machine on Google Cloud. Production images are built automatically on each merge to master, then those get deployed by manually running a command.

### Testing deployment

You can test the production stack locally, or initialize a new one, like so:

- Make sure the values in `deploy/dev.env` are correct for you
  - Change `GDLK_HOSTNAME` to another address (preferably one that is listed under the OpenID clients)
    - You may need to add this to `/etc/hosts` to make it point at your development machine
  - Change `GDLK_DOCKER_TAG` if you're going to be pushing different versions of images.
- Run this:

```sh
cd deploy
cargo make deploy
# Fill in the secrets as necessary
```

### Production deployment

Set the production machine as your docker-machine target, then:

```sh
docker-machine ssh <machine name>
mkdir -p /var/log/gdlk/nginx
exit

# On your development machine
cd deploy
cargo make -p production deploy
# Fill in the secrets as necessary
```

### DB Backup/Restore

The database is backed up automatically. To restore from a backup, exec into the `db-backup` container, then run:

```sh
gsutil cp gs://<bucket>/<backup file> .
tar xzvf <backup file>
PGPASSWORD=$(cat $POSTGRES_PASSWORD_FILE) psql -h $POSTGRES_HOST -U $POSTGRES_USER -c "CREATE DATABASE gdlk;" # If necessary
PGPASSWORD=$(cat $POSTGRES_PASSWORD_FILE) psql -h $POSTGRES_HOST -U $POSTGRES_USER gdlk < backups/gdlk.bak
```
