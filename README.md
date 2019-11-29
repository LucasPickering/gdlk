# GDLK

## Development

### Setup

First, you'll need to install:

- `docker-compose`
- `rustup`

Then, for development you'll probably want to install:

```
rustup component add rustfmt-preview clippy-preview
```

If you're using VSCode and have the RLS extension installed (you should), it'll ask to install more components, just say yes.

### Running

In the repo root:

```
docker-compose up
# Wait for DB to start up
docker exec gdlk_api_1 diesel migration run
```
