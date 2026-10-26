# Configuration

Project settings use a TOML file named `modfmt.toml`. All settings are optional. [config.schema.json](../config.schema.json) is the authoritative reference for `modfmt.toml` setting names, types, defaults, and allowed values. Exclusion glob syntax is additionally validated by the application.

For example, to exclude fixtures and vendored code:

```toml
[files]
exclude = ["fixtures/**", "vendor/**"]
```

## Settings at a glance

| Key | Purpose | When omitted |
| --- | --- | --- |
| `files.respect_gitignore` | Apply `.gitignore` rules during directory traversal. | `true` |
| `files.exclude` | Exclude paths using gitignore-style patterns, including explicitly supplied files. | `[]` (no additional exclusions) |
| `format.edition` | Select the Rust edition used to parse source. | Automatic lookup; see [edition precedence](#editions-and-automatic-lookup). |
| `format.style_edition` | Select the style edition used to sort imports. | Automatic lookup; see [edition precedence](#editions-and-automatic-lookup). |

Import width and continuation indentation are read separately from rustfmt configuration; see [import width and indentation](#import-width-and-indentation). See [file selection](#file-selection) for exclusion syntax and traversal rules. CLI overrides are described below.

## Loading settings

- The tool searches from the working directory upward and uses the nearest `modfmt.toml`. Search stops at the Git repository root, or the filesystem root outside a repository.
- `--config <FILE>` replaces automatic lookup. Relative paths are resolved from the working directory.
- One `modfmt.toml` applies to the entire invocation. Parent and child configurations are not merged, and input directories do not trigger separate `modfmt.toml` lookups. Unspecified edition settings are resolved per source as described below.
- If no configuration is found, file selection uses built-in defaults and editions can still be inferred. Invalid settings, including unknown keys, cause an error before files are processed.

`--edition` overrides `format.edition`, and `--style-edition` overrides `format.style_edition`. `--exclude` adds patterns to `files.exclude`. CLI overrides do not bypass validation of the configuration file.

## Editions and automatic lookup

`edition` controls parsing; `style_edition` controls import sorting. Both accept `2015`, `2018`, `2021`, and `2024`. Style editions 2015–2021 use rustfmt's older name categories and lexical order; 2024 uses version sorting. For example, `alpha10` precedes `alpha2` in the older style and follows it in 2024.

| Value | Precedence, highest first |
| --- | --- |
| Parsing edition | CLI `--edition` → `format.edition` → Cargo package edition → rustfmt `edition` → `2024` |
| Style edition | CLI `--style-edition` → `format.style_edition` → rustfmt `style_edition` → resolved parsing edition |

Cargo takes precedence over rustfmt's parsing edition to match `cargo fmt`. An explicit style edition remains independent of parsing.

To override automatic lookup and use 2024 for both editions in every selected file:

```toml
[format]
edition = "2024"
style_edition = "2024"
```

For each selected source, lookup starts at its canonical parent directory and walks upward to the nearest Git repository root, inclusive, or the filesystem root outside a repository. Cargo.toml and rustfmt configuration are searched independently. The nearest `rustfmt.toml` or `.rustfmt.toml` is used without merging parent settings; `.rustfmt.toml` wins when both names exist in the same directory. No additional fallback to home or global rustfmt configuration directories is performed; normal ancestor lookup still applies. `stdin` uses the working directory as its lookup starting point. `--config` changes only the modfmt configuration, not these lookup roots.

The nearest Cargo.toml supplies `[package].edition`. An omitted package edition means `2015`, matching Cargo. `edition.workspace = true` uses `[workspace.package].edition` from the nearest ancestor workspace, or from the workspace explicitly referenced by `package.workspace`. An explicit reference can point outside the normal search boundary. A virtual workspace manifest has no package edition; its shared value is used only by packages that opt into inheritance.

Cargo settings are interpreted only for edition lookup. From rustfmt configuration, the tool reads edition settings plus `max_width`, `tab_spaces`, and `hard_tabs`. Other rustfmt and Cargo keys are ignored; dependencies are not resolved and neither Cargo nor rustfmt is invoked. Unusual target paths outside a package, or one source shared by targets with different editions, are not inferred from Cargo's target graph.

Rustfmt configuration is searched for every selected source, even when both editions are explicit, because it also supplies layout settings. Cargo settings are read only when the parsing edition is not explicit. A required file that is unreadable, malformed, not a regular file, or has invalid supported settings causes an error rather than a fallback. Missing workspace inheritance values also cause an error. All selected files' format settings are resolved before listing or writing, including in `files` mode; source contents are not read during this step.

When no style edition is configured, changing the parsing edition also changes the sorting style. To use 2024 sorting independently of the parsing edition, use `--style-edition 2024` or set `format.style_edition = "2024"`.

## Import width and indentation

The nearest rustfmt configuration supplies these settings independently for each source file, using the same lookup rules described above. Missing settings use the defaults below; parent configurations are not merged.

| Rustfmt key | Purpose | Default |
| --- | --- | --- |
| `max_width` | Target line width for import layout, including the declaration prefix and indentation. | `100` |
| `tab_spaces` | Spaces per continuation level and columns per tab when calculating line width. | `4` |
| `hard_tabs` | Use a tab per continuation level instead of spaces. | `false` |

For example, in `rustfmt.toml`:

```toml
max_width = 80
tab_spaces = 2
hard_tabs = false
```

These keys are read only from rustfmt configuration, including for `stdin`, which looks up settings from the working directory. They have no CLI or `modfmt.toml` overrides. `--config` still selects only the modfmt configuration and does not change rustfmt lookup.

`max_width` and `tab_spaces` accept nonnegative integers; `hard_tabs` accepts a boolean. `tab_spaces = 0` adds no continuation spaces and is rejected when combined with `hard_tabs = true`.

The tool preserves each section's existing base indentation and uses these settings for newly generated continuation lines and width calculations. It does not reindent surrounding Rust code or normalize line endings. A long indivisible path can exceed `max_width`; the width is a layout target.

## File selection

Directory traversal respects `.gitignore` when `files.respect_gitignore` is enabled. It always skips hidden paths and `target` directories, and does not follow symlink entries. Generated code has no special detection; exclude it explicitly.

Explicit regular files bypass `.gitignore` and the hidden-path and `target` traversal exclusions, but still obey configuration and CLI exclusions. A path whose final component is a symlink is rejected, even when supplied explicitly; symlinks in parent directory components are resolved normally. Explicit directories remain subject to traversal filters.

Inside a Git repository, traversal loads `.gitignore` rules from ancestors up to the nearest repository root, as well as from directories it visits. Outside a repository, only the supplied traversal root and its descendants contribute `.gitignore` rules; parent directories above that root are not consulted. Deeper rules and later patterns take precedence. Negation patterns in `.gitignore` can re-include entries, but traversal does not enter an excluded directory to read rules inside it.

The tool does not read the Git index: tracked files are also excluded when they match `.gitignore`. Global Git ignore files, `.git/info/exclude`, and `.ignore` are not read. Setting `files.respect_gitignore = false` disables `.gitignore` filtering; hidden-path, `target`, configuration, and CLI exclusions still apply.

Exclusion patterns are relative to the configuration file's directory, or the working directory when no configuration is loaded. Input paths always remain relative to the working directory.

Patterns use gitignore syntax, including anchored patterns such as `/fixtures/`. Re-inclusion with a leading `!` is not supported in `files.exclude` or `--exclude`. Each pattern must fit on a single line.

```sh
cargo modfmt files . --exclude 'generated/**'
cargo modfmt check src --config settings/modfmt.toml
```

Use `files` to preview the selection before writing. The `stdin` command validates the configuration and resolves editions and layout settings, but does not apply file selection rules.

Group order, alias order, merging policy, Mixed import layout, and Block import indentation remain fixed. Rustfmt options other than the editions and the three layout settings above are not applied. See [rustfmt compatibility](rustfmt-compatibility.md) for the supported formatting settings and workflow.
