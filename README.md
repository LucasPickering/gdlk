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

### Running

In the repo root:

```sh
docker-compose up
# Wait for DB to start up
docker exec gdlk_api_1 diesel migration run
```

### Tests

You can run tests with:

```sh
cargo test
```

#### Debugging

If you have a test failing, you can run just that test with:

```sh
cargo test test_name # Run just one test
cargo test -- --nocapture  # Print stdout from the program
DEBUG=true cargo test -- --nocapture # Run in debug mode (includes more useful output)
```
