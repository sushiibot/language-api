# language-api

HTTP API server to detect language used in provided text.

Uses [lingua-rs](https://github.com/pemistahl/lingua-rs).

## Usage

Loading language models take a long time, so be sure to use `--release` when
building.

```bash
cargo run --features=build-binary --release
./target/release/language-api
```

This also includes a Rust API wrapper to not have to manually create HTTP
requests.

```toml
language-api = { git = "https://github.com/sushiibot/language-api" }
```

## Configuration

Configuration options can be set in the environment or in an `.env` file.

The `languages` option is required, to determine which languages to detect. This
must be a comma separated list of languages enclosed in quotes. If no quotes are
provided, then they must not have spaces in between.

Example `.env` file shown below:

```bash
# Optional, if you want it to listen on a different interface or port
LISTEN_ADDR=0.0.0.0:8080

LANGUAGES="english, japanese, chinese, french, turkish"
# Or without quotes, must have no spaces
LANGUAGES=english,japanese,chinese,french,turkish
```
