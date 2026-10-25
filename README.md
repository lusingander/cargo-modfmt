# cargo-modfmt

[![Crate Status](https://img.shields.io/crates/v/cargo-modfmt.svg)](https://crates.io/crates/cargo-modfmt)

Organize Rust `mod` and `use` declarations alongside stable rustfmt.

## About

A Cargo subcommand that sorts and groups Rust `mod` and `use` declarations, and merges compatible imports.

Before:

```rust
use crate::config::Config;
use std::io;
pub mod api;
use serde::Serialize;
mod parser;
use std::fmt;
mod config;
```

After `cargo modfmt write example.rs`:

```rust
mod config;
mod parser;

pub mod api;

use std::{fmt, io};

use serde::Serialize;

use crate::config::Config;
```

The tool organizes file-level declarations and eligible inline module bodies. Imports inside functions are outside its scope. A comment or declaration attribute protects the entire consecutive `mod`/`use` section from changes. See [coverage limits](docs/rustfmt-compatibility.md#current-coverage-limits) for details and exceptions.

General Rust formatting is handled by rustfmt. See [Using rustfmt](docs/usage.md#using-rustfmt) for the formatting workflow and CI checks.

### Goals

- Sort `use` and `mod` declarations without requiring nightly rustfmt.
- Support some grouping rules rustfmt does not provide, such as separating modules by visibility.

### Non-goals

- Match every rustfmt behavior or setting.
- Replace rustfmt for general Rust formatting.

This tool may become unnecessary as rustfmt improves, and that is fine.

## Documentation

- [Usage and command line options](docs/usage.md)
- [Configuration](docs/configuration.md)
- [Differences from rustfmt and current limitations](docs/rustfmt-compatibility.md)

## Installation

Requires Rust 1.98.0 or later. Linux and macOS are the current verification targets; Windows is untested.

```sh
cargo install --locked cargo-modfmt
```

## Usage

```sh
# Organize Rust files in the current directory
cargo modfmt write .

# Check without modifying files
cargo modfmt check .

# Preview selected files
cargo modfmt files .

# Read source from stdin and write the result to stdout
cargo modfmt stdin < src/main.rs
```

File commands accept one or more files or directories. `check` exits with code `3` when changes are needed.

Run `cargo help modfmt` for all modes and options. See [Usage](docs/usage.md) for details.

### Configuration

Parsing and style editions are inferred from each source file’s Cargo and rustfmt settings. Use `--edition` / `--style-edition` to override them.

Place project settings in `modfmt.toml`. See [Configuration](docs/configuration.md) for examples and the [configuration schema](config.schema.json) for all settings and defaults.

## License

[MIT](LICENSE)
