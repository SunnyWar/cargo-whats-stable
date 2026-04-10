# cargo-whats-stable

A CLI tool to track and check the stability of Rust standard library features.

## Features
- `add <feature>`: Add a feature to the tracking list.
- `remove <feature>`: Remove a feature from the tracking list.
- `check`: Check the status (stable/nightly) of all tracked features by querying the Rust unstable book.
- `list`: List all tracked features and their current status.

## Installation

You can install the tool locally with:

```
cargo install --path .
```

## Usage

You can use the tool either as a cargo subcommand or by running the binary directly:

### As a cargo subcommand

```
cargo whats-stable <add|remove|check|list> [feature]
```

Examples:
```
cargo whats-stable add portable_simd
cargo whats-stable check
cargo whats-stable list
```

### Directly (if installed in PATH)

```
cargo-whats-stable <add|remove|check|list> [feature]
```

Examples:
```
cargo-whats-stable add portable_simd
cargo-whats-stable check
cargo-whats-stable list
```

## How it works
- Features are stored in a local `features.json` file.
- The tool checks the Rust unstable book online to determine if a feature is still nightly or has become stable.

## Requirements
- Rust (edition 2021 or later)
- Internet connection for `check` command

## License
MIT
