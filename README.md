# cargo-whats-stable

A CLI tool to track and check the stability of Rust standard library features.

## Features
- `add <feature>`: Add a feature to the tracking list.
- `remove <feature>`: Remove a feature from the tracking list.
- `check`: Check the status (stable/nightly) of all tracked features by querying the Rust unstable book.
- `list`: List all tracked features and their current status.

## Usage

```
cargo run -- <command> [feature]
```

Examples:
```
cargo run -- add async_closure
cargo run -- check
cargo run -- list
```

## How it works
- Features are stored in a local `features.json` file.
- The tool checks the Rust unstable book online to determine if a feature is still nightly or has become stable.

## Requirements
- Rust (edition 2021 or later)
- Internet connection for `check` command

## License
MIT
