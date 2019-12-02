# GDLK

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

```
cargo run -- execute -e env.json -i prog.gdlk
```

### Running the Webserver

In the repo root:

```sh
docker-compose up
# Wait for DB to start up
./initdb.sh
```

### Tests

You can run tests with:

```sh
cargo test
```

### Debugging

If you have a program or test failing, you can run with additional debug output by setting `DEBUG=true`, like so:

```sh
DEBUG=true cargo run -- execute -e env.json -i prog.gdlk
DEBUG=true cargo test -- --nocapture # --nocapture needed to make stdout visible
```
