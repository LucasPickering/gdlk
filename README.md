# GDLK

## About

GDLK is a programming language, as well as a game based on solving puzzles with the language. We have a basic playable version available at [gdlk.dev](https://gdlk.dev). Feedback is appreciated!

### What does GDLK stand for?

GDLK Development Language Kit

#### Wait, but, what does GDLK stand for?

GDLK Development Language Kit!

## Development

### Setup

First, you'll need to install:

- `rustup`

Then, for development you'll probably want to install more stuff:

```sh
rustup component add rustfmt-preview clippy-preview
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

### Running the Frontend

In the repo root, run:

```sh
cd frontend
nvm install
nvm use
npm install
npm run start
```

### Tests

You can run tests with:

```sh
cargo make test # In the root, or any sub-crate
```

### Debugging

If you have a GDLK program or test failing, you can have the GDLK compiler and interpreter output additional debug information by setting `DEBUG=1`.

```sh
DEBUG=1 cargo run -p gdlk_cli -- --hardware hw.json --program prog.json -s prog.gdlk
# OR
DEBUG=1 cargo make test
```

### Nightly Rust

We use nightly Rust. Here's a list of reasons why. If this list every gets empty, we should switch to stable.

- Rustfmt
  - [imports_granularity](https://github.com/rust-lang/rustfmt/issues/4669)
  - [wrap_comments](https://github.com/rust-lang/rustfmt/issues/3347)

[Here's a helpful site for finding new nightly versions](https://rust-lang.github.io/rustup-components-history/).

## Deployment

This project is just a static site deployed through GitHub Pages. Each push to master will re-build the site and GitHub will automatically deploy that version.
