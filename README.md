# GDLK

## About

GDLK is a programming language, as well as a game based on solving puzzles with the language.

### What does GDLK stand for?

GDLK Development Language Kit

#### Wait, but, what does GDLK stand for?

GDLK Development Language Kit!

## Development

### Setup

First, you'll need to install:

- `docker-compose`
- `rustup`

Then, for development you'll probably want to install:

```sh
rustup component add rustfmt-preview clippy-preview
```

If you're using VSCode and have the RLS extension installed (you should), it'll ask to install more components, just say yes.

### Compiling and Executing Locally

If you just want to compile and run a program without starting up the webserver, you can use the CLI for that. First, you'll need the environment to execute under saved in a JSON file, e.g. `env.json`. Then you need your program source in a file, e.g. `prog.gdlk`. Then run:

```sh
cargo run -- execute -e env.json -i prog.gdlk
```

### Running the Webserver

In the repo root:

```sh
docker-compose up
```

Then, see the next section for initialize the DB.

### Migrations & Seed Data

Migrations are managed by Diesel. Seed data is defined in code, in `seed.rs`.

```sh
./x.py migrate # Run initial migrations
./x.py seed # Insert seed data

# If you need to re-run migrations (this will wipe out your DB!)
./x.py migrate --redo
```

### Tests

You can run tests with:

```sh
./x.py test
```

### Debugging

If you have a program or test failing, you can run with additional debug output by setting `DEBUG=1`. There is also the `--debug` (or `-d`) flag on the `x.py` command that does the same thing.

```sh
DEBUG=1 cargo run -- execute -e env.json -i prog.gdlk
./x.py test --debug core
```
