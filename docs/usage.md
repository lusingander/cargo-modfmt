# Usage

With `cargo-modfmt` installed, run:

```text
cargo modfmt <COMMAND> [OPTIONS]
```

| Command | Behavior |
| --- | --- |
| `write <PATH>...` | Update files in place; report updated paths to stderr. |
| `check <PATH>...` | Report paths needing changes to stdout without modifying files. |
| `files <PATH>...` | List selected paths without reading their source. |
| `stdin` | Read UTF-8 Rust source from stdin and write the organized source to stdout. |

File commands require one or more files or directories. Directories are searched recursively for `.rs` files; Cargo workspaces and external module declarations are not used to discover files.

See [file selection](configuration.md#file-selection) for traversal filters, `.gitignore` scope, and explicit-path rules, including symlink handling.

```sh
cargo modfmt write src tests
cargo modfmt check src/main.rs src/lib.rs
cargo modfmt files . --exclude 'vendor/**'
cargo modfmt stdin < src/main.rs > /tmp/main-organized.rs
```

## Using rustfmt

Use stable rustfmt with matching style editions and otherwise default formatting settings. See the [compatibility target](rustfmt-compatibility.md#compatibility-target) for supported settings and limitations.

The tool automatically reads edition settings from Cargo and rustfmt configuration. To use 2024 sorting with an older parsing edition, make the style explicit in `rustfmt.toml`:

```toml
style_edition = "2024"
```

Rustfmt's parsing edition and style edition are separate settings. See the [official style edition documentation](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#style_edition).

From the Cargo project directory, format with:

```sh
cargo modfmt write .
cargo +stable fmt --all
cargo modfmt write .
```

The first pass handles the [trailing `::self` case](rustfmt-compatibility.md#keeping-the-meaning-of-self-imports) in eligible sections before rustfmt sees it. In skipped sections or scopes outside this tool's coverage, convert trailing `::self` to the brace spelling manually before running rustfmt. Rustfmt formats the rest of the source; the second pass organizes that formatted source.

For CI, run both checks:

```sh
cargo +stable fmt --all -- --check
cargo modfmt check .
```

For supported input under the target settings, both checks should pass after the formatting workflow, and further formatting should produce no changes.

**Passing one check does not imply passing the other.** Stable rustfmt can accept separate imports that this tool would merge. Conversely, this tool can leave an unsupported or protected section unchanged even when rustfmt would format it.

## Options

| Option | Behavior |
| --- | --- |
| `--config <FILE>` | Use an explicit configuration file instead of searching for `modfmt.toml`. |
| `--edition <EDITION>` | Override the parsing edition; also supplies the style fallback when no style edition is configured. |
| `--style-edition <EDITION>` | Override the edition used to sort imports, independently of parsing. |
| `--exclude <PATTERN>` | Add an exclusion pattern. Repeatable; available for file commands. |

For accepted editions and configuration defaults, see [config.schema.json](../config.schema.json). Unspecified editions are resolved from Cargo and rustfmt settings for each source file. Configuration lookup, precedence, and exclusion rules are described in [Configuration](configuration.md).

Run `cargo help modfmt` or `cargo modfmt <COMMAND> --help` for the full command help.

## Output format

File commands print one line per reported path, ending with LF. Paths are double-quoted and escaped; for example, a newline in a filename is displayed as `\n`.

| Command | Stream | Example line |
| --- | --- | --- |
| `files` | stdout | `"src/main.rs"` |
| `check` | stdout | `would reformat: "src/main.rs"` |
| `write` | stderr | `reformatted: "src/main.rs"` |

`files` lists all selected paths. `check` reports only files needing changes, and `write` reports only successfully updated files; neither reports unchanged files. The `stdin` command writes the Rust source itself to stdout, without quoting it as a path.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success; for `check`, no changes are needed. |
| `1` | Configuration, file selection, or processing error. |
| `2` | Invalid command line arguments. |
| `3` | `check` found files needing changes, with no processing errors. |

## File updates and failures

Source must be valid UTF-8 and parse successfully. A failed file is not partially rewritten.

Edition settings for all selected files are resolved before any processing or listing. A configuration error leaves all sources unchanged. After an individual source fails during processing, other selected files are still processed. Successful writes are retained even if the command eventually exits with an error.

Read-only sources are protected. Avoid concurrent editing. Ownership, ACLs, extended attributes, and hard-link relationships are not guaranteed to survive an update.

Unsupported or protected sections can remain unchanged without an error, and skip reasons are not reported. See [rustfmt compatibility and limitations](rustfmt-compatibility.md) for transformation rules and skipped sections.
