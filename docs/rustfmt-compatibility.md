# Differences from rustfmt and current limitations

`cargo-modfmt` organizes Rust `mod` and `use` declarations. It adds fixed grouping and conservative import merging to a workflow that uses stable rustfmt for general formatting.

This guide separates [deliberate differences](#deliberate-differences) from [current coverage limits](#current-coverage-limits). Examples use illustrative module and crate names; they do not require those dependencies to be installed to try the tool's `stdin` command.

## Compatibility target

The compatibility target is **stable rustfmt with matching style editions (2015, 2018, 2021, or 2024), shared `max_width`, `tab_spaces`, and `hard_tabs` settings, and defaults for other formatting settings**. Nightly compatibility and identical output across rustfmt versions are not promised. See [comparison versions and verification](#comparison-versions-and-verification) for the versions used in this guide.

See [corresponding rustfmt settings](#supported-and-fixed-formatting-settings) for the closest options and how custom settings can affect this workflow.

Within each group, ordinary import sorting follows the selected style edition. In 2024, `alpha2` precedes `alpha10`; in 2015–2021, identifiers use the older snake_case → CamelCase → UPPER_SNAKE_CASE categories and lexical comparison, so `alpha10` comes first. Local roots follow `self`, `super`, then `crate` in both styles. See the [official version sorting guide](https://doc.rust-lang.org/edition-guide/rust-2024/rustfmt-version-sorting.html). Singleton braces, redundant ordinary aliases, and visibility spelling are also normalized. These are shared behaviors; the extra arrangement rules and coverage gaps are described below. Nightly is not required to run the tool.

See [Using rustfmt](usage.md#using-rustfmt) for the formatting workflow and CI checks.

## Deliberate differences

These rules are part of the tool's chosen behavior. They are not attempts to reproduce rustfmt's output for the same input.

### Five groups, including module visibility

Each consecutive section of external module declarations (`mod name;`) and imports is arranged in this order, with one blank line between nonempty groups:

| Order | Group | Examples |
| --- | --- | --- |
| 1 | Modules without a visibility modifier | `mod internal;` |
| 2 | Modules with any visibility modifier | `pub mod api;`, `pub(crate) mod shared;`, `pub(self) mod local;` |
| 3 | Standard library imports | Roots `alloc`, `core`, `std` |
| 4 | Other imports | Roots such as `vendor` or `utils` |
| 5 | Local imports | Roots `self`, `super`, `crate` |

Within each module group, names use case-sensitive lexical order with any `r#` prefix ignored. This order is independent of the style edition: `mod alpha10;` precedes `mod alpha2;` even in 2024. Equal names retain their input order; visibility kinds within the second group do not add another sort key.

Input:

```rust
use crate::Config;
pub mod api;
use vendor::Client;
mod zeta;
use std::io;
mod alpha;
```

Tool output:

```rust
mod alpha;
mod zeta;

pub mod api;

use std::io;

use vendor::Client;

use crate::Config;
```

Default stable rustfmt does not create these five groups. Nightly's standard/external/local import grouping covers part of this behavior, but does not separate modules by visibility. The tool does not inspect dependencies: `utils::Item` is placed in the “other imports” group even if `utils` is a local module.

### Blank lines and external modules do not stop merging

Within a consecutive section, existing blank lines are replaced by the tool's group spacing. Imports can move across external `mod` declarations and merge with imports on the other side.

Input:

```rust
use foo::Z;

mod support;

use foo::A;
```

Tool output:

```rust
mod support;

use foo::{A, Z};
```

Default stable rustfmt does not merge separate declarations. The compared nightly configuration can merge across blank lines, but the intervening `mod support;` still separates its import sections.

A function, type definition, inline module, or another kind of item ends a section. Imports never move across that boundary or into another scope.

### Aliases have an additional order

For matching paths inside a brace group, the tool orders entries as: no alias, named aliases in lexical order, then `as _`.

Input:

```rust
use foo::{Item as Z, Item as _, Item as A, Item};
```

Tool output:

```rust
use foo::{Item, Item as A, Item as Z, Item as _};
```

The compared rustfmt versions keep the input order for these aliases, but accept the tool's output. This extra alias ordering does not reorder separate declarations that remain unmerged.

There is an exception when an import declaration contains duplicate imports: aliases with otherwise equal paths keep their relative order throughout that declaration. For example, this stays unchanged:

```rust
use foo::{Item as B, Item as A, Item as B};
```

Sorting those aliases would bring the two `B` imports together, which causes the reference stable rustfmt to remove one. The tool preserves duplicates instead.

### Raw and absolute paths keep their spelling

For import classification, `r#std` is treated as `std` in every style edition, and a leading `::` does not change the import group.

Import path sorting depends on the style edition: 2024 compares names without `r#`, while 2015–2021 compare the original spelling including that prefix. For example, `use foo::{r#beta, alpha, gamma};` becomes `use foo::{alpha, r#beta, gamma};` in 2024, but `use foo::{alpha, gamma, r#beta};` in the older styles. Module-name comparisons, alias comparisons, and duplicate checks ignore `r#` in every style edition. Output retains the raw spelling.

Input:

```rust
use vendor::Item;
use r#std::fmt;
use ::std::io;
```

Tool output:

```rust
use ::std::io;
use r#std::fmt;

use vendor::Item;
```

The compared nightly grouping does not classify these raw and absolute spellings in the same way as plain `std`.

Classification does not make paths interchangeable for merging. The tool keeps `foo::A`, `::foo::B`, and `r#foo::C` as separate declarations. It also keeps `super` and `super::super` as separate merge roots, without trying to resolve their destinations.

### Merging is conservative and preserves duplicates

Imports merge only when their root spelling, absolute/relative form, and normalized visibility match. For example:

Input:

```rust
pub(in crate) use foo::B;
pub(crate) use foo::A;
```

Tool output:

```rust
pub(crate) use foo::{A, B};
```

Private imports and `pub(self)` imports remain separate, as do imports with other distinct visibility restrictions.

Even matching imports remain separate in the following cases. Each row shows a complete example that the tool leaves as separate declarations.

| Condition | Example |
| --- | --- |
| An ordinary import names the root itself | `use foo as A;` and `use foo as B;` |
| The same import occurs more than once | `use foo::A;` and `use foo::A;` |

If any of these occurs, merging is declined for **all imports with that matching root and visibility in the section**. Other roots can still merge. The declarations that remain separate are still sorted and formatted.

The compared nightly configuration can combine more of these cases and remove duplicates. The tool deliberately retains duplicates and keeps ordinary imports as siblings of their descendants (`use foo::{a, a::B};`) to avoid converting them into type-only `self` imports. It also keeps glob imports and explicit names together without removing either:

Input:

```rust
use foo::*;
use foo::Item;
```

Tool output:

```rust
use foo::{Item, *};
```

### Keeping the meaning of `self` imports

Input:

```rust
use model::entry::self as Entry;
```

Tool output:

```rust
use model::entry::{self as Entry};
```

The explicit `self` imports only from the type namespace. An ordinary `use model::entry as Entry;` can also import a function with the same name. The tool preserves this distinction, including when the `self` import is the only entry in braces.

The reference stable and nightly versions turn the trailing `::self` spelling into an ordinary import, which can change behavior when a module and function share a name. They accept the brace spelling above. If your source uses trailing `::self`, run this tool before its first rustfmt pass, as in the [formatting workflow](usage.md#using-rustfmt).

This conversion only happens in sections the tool can organize. A comment or declaration attribute protects the whole section, and function-local imports and other unsupported scopes are not visited. For example, `use model::entry::self as Entry; // explanation` stays unchanged. Convert it manually to `use model::entry::{self as Entry}; // explanation` before running rustfmt. Both formatter checks can pass after rustfmt has changed the import's meaning; neither check verifies compilation or name resolution.

## Current coverage limits

These are cases the current tool does not organize, or capabilities it does not provide. An unchanged result may therefore mean that a section was skipped.

The tool does not add missing imports, remove unused imports, expand globs, or repair name conflicts. Use the compiler and your usual analysis tools for those tasks.

### Comments and declaration attributes protect the whole section

An entire consecutive `mod`/`use` section stays unchanged if it contains a comment or if one of its declarations has an attribute or documentation comment. The tool does not move comments or infer which declaration they describe.

Unchanged:

```rust
use foo::Z;
use foo::A; // Keep this explanation with its import.
use bar::B;
```

This includes the otherwise unrelated `bar` import. An ordinary comment immediately before or after the section also protects it, even with a blank line between them. Comments are not used to split a section into smaller sortable parts.

Unchanged:

```rust
#[cfg(unix)]
use foo::Z;
use foo::A;
```

This applies to attributes on external `mod name;` declarations as well. Rustfmt can format many commented or attributed declarations; this tool leaves these sections to rustfmt or manual editing.

### Function bodies, general blocks, and macros are outside the scope

The tool organizes file-level declarations and eligible inline module bodies. It does not organize imports in functions, methods, other blocks, or macro bodies, including inline modules nested in those locations.

Unchanged:

```rust
fn example() {
    use foo::Z;
    use foo::A;
}
```

Rustfmt can sort function-local imports. Formatting expressions, function bodies, and other general Rust code also remains rustfmt's job.

### Some inline modules are not visited

An inline module with no outer attributes is visited. Outer `#[cfg(...)]`, `#[doc = ...]`, and documentation comments are also allowed; their text stays in place, and `cfg` conditions are not evaluated.

An inline module with `cfg_attr`, `macro_use`, a custom attribute, or a qualified attribute such as `custom::doc` is left entirely unchanged. Its descendant modules are not visited either.

Unchanged:

```rust
#[cfg_attr(feature = "extra", custom_attribute)]
mod protected {
    use foo::Z;
    use foo::A;
}
```

### Supported inner headers stay in place

A `mod`/`use` section starting at the first item in a file or eligible inline module can be organized after inner documentation (`//!` or `/*! ... */`), an unqualified `doc` or `cfg(...)` attribute, or an unqualified lint attribute: `allow(...)`, `warn(...)`, `deny(...)`, `forbid(...)`, or `expect(...)`. These headers can be combined. Their text, order, and surrounding whitespace stay in place; `cfg` conditions and attribute values are not evaluated.

Input:

```rust
#![allow(unused_imports)]
//! Crate documentation.

use foo::Z;
use foo::A;

fn boundary() {}

use bar::Z;
use bar::A;
```

Tool output:

```rust
#![allow(unused_imports)]
//! Crate documentation.

use foo::{A, Z};

fn boundary() {}

use bar::{A, Z};
```

If any inner header is outside this set, a section starting at the scope's first item stays unchanged. This includes `cfg_attr`, custom or qualified attributes such as `custom::doc` and `rustfmt::skip`, and other crate settings. Even `cfg_attr(test, allow(unused_imports))` protects a leading section. Sections after another item and independently eligible child modules can still be organized.

Ordinary comments before, between, or after the headers protect the first section, including comments separated by blank lines. Attributes and documentation on the declarations themselves also retain their usual protection. The outer-attribute rules for visiting an inline module remain as described above.

### Some import syntax is not supported

If any declaration in a section uses unsupported syntax, the **whole section** stays unchanged. Supported neighbors are not extracted from it.

| Unsupported form | Examples |
| --- | --- |
| A brace group without a path of its own, at any nesting level | `use {std::io, crate::Item};`, `use foo::{{A, B}, C};` |
| An empty group, at any nesting level | `use foo::{};`, `use foo::{a::{}, B};` |
| A child path beginning with `self::`, `super::`, `crate::`, or `::` | `use foo::{self::a::A, Z};` |
| A bare qualifier import without an alias | `use crate;`, `use super;`, `use crate::{self};` |
| Other unsupported keyword positions or a pathless glob | `use foo::super::Item;`, `use *;` |

These examples describe parsing and organization coverage, not whether Rust accepts every form in every context. A syntax error reported by the parser is an error for the entire file, rather than a successful skip. A syntactically valid file is not necessarily compilable: the tool does not resolve names or check import collisions.

### Existing siblings are not regrouped within a single declaration

The tool sorts this import, but keeps its import structure:

```rust
use foo::{a::A, a::B};
```

It does not turn it into `use foo::a::{A, B};` just to consolidate the shared prefix. The compared nightly `Crate` setting does. Multiple compatible declarations can be merged and share such prefixes; this limitation concerns an existing single declaration.

Singleton braces and redundant ordinary aliases are normalized, so `use foo::{Item};` and `use foo::Item as Item;` both become `use foo::Item;`. Preserving a single declaration's shared prefixes does not mean preserving every brace or space.

### An unformatted input may still fail stable fmt check

The tool preserves protected sections and code outside its scope. It also preserves duplicate imports, a UTF-8 BOM, LF or CRLF line endings, and whether the file ends with a newline. A section containing mixed LF/CRLF line endings is skipped.

For example, adjacent duplicates inside one group remain present:

```rust
use foo::{Item, Item};
```

The reference stable rustfmt removes one, so this tool's unchanged output fails stable fmt check. Separate duplicate declarations are retained by both tools in the reference stable configuration.

A file missing its final newline can also pass this tool's check while failing stable fmt check. Preserving a BOM or CRLF alone did not fail the reference rustfmt's real-file check; that should not be confused with its output when reading from standard input.

### Supported and fixed formatting settings

Edition settings are read from Cargo and rustfmt configuration. Import layout also reads rustfmt's `max_width`, `tab_spaces`, and `hard_tabs`. See [Configuration](configuration.md#editions-and-automatic-lookup) for lookup rules and edition overrides, and [import width and indentation](configuration.md#import-width-and-indentation) for layout defaults and validation.

Custom group order, alias order, merging policy, import layout strategy, and import indentation strategy are not configurable. Selecting a style edition changes import sorting, not general Rust formatting or all of rustfmt's edition-dependent behavior. A long indivisible path can exceed the configured width; the width is a layout target.

The following rustfmt settings are the closest counterparts for the [reference versions](#comparison-versions-and-verification). Grouping and merging still have the deliberate differences described earlier. These are rustfmt options; the editions and the width and indentation settings configure this tool through rustfmt configuration.

| Behavior | Related rustfmt setting | Configurable on | This tool |
| --- | --- | --- | --- |
| Import sorting | [`reorder_imports = true`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#reorder_imports) | Stable | Always sorts eligible imports, with the additional alias order described above. |
| Module sorting | [`reorder_modules = true`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#reorder_modules) | Stable | Always sorts eligible modules and separates them by visibility. |
| Import grouping | [`group_imports = "StdExternalCrate"`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#group_imports) | Nightly | Uses three import groups plus two module groups. |
| Import merging | [`imports_granularity = "Crate"`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#imports_granularity) | Nightly | Merges compatible declarations conservatively and preserves duplicates. |
| Width | [`max_width = 100`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#max_width) | Stable | Reads the configured width target; defaults to 100. |
| Indentation | [`tab_spaces = 4`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#tab_spaces), [`hard_tabs = false`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#hard_tabs) | Stable | Preserves the section's base indentation; continuation lines and tab width follow these settings. |
| Import layout | [`imports_layout = "Mixed"`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#imports_layout) | Nightly | Can place several siblings on one continuation line. |
| Import indentation | [`imports_indent = "Block"`](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md#imports_indent) | Nightly | Indents continuation lines relative to the enclosing group. |

“Nightly” means the option requires nightly to configure explicitly; stable rustfmt still uses its built-in defaults, including Mixed layout and Block import indentation. The nightly comparison changes only `imports_granularity` and `group_imports` from their defaults. Matching those two values does not make nightly output identical to this tool's output.

Unsupported settings that change declaration layout or import structure can produce competing changes. For example, nightly's `imports_layout = "Vertical"` can make rustfmt wrap imports differently, while this tool continues to use Mixed layout. Alternating the tools may then keep changing the same source. Width and indentation settings must resolve to the same values in both tools; this tool retains its documented lookup boundaries and does not read rustfmt's home or global fallback configuration. Nightly's `imports_granularity = "Module"`, `"Item"`, or `"One"` can also change the structure that this tool produces or supports.

Disabling `reorder_imports` or `reorder_modules` in rustfmt does not disable sorting in this tool; rustfmt simply leaves that resulting order in place. These switches do not by themselves imply a conflict. Check both tools with your actual configuration; convergence outside the compatibility target is not guaranteed.

## Comparison versions and verification

The comparisons and compatibility checks in this guide use these reference versions:

| Formatter | Version | Comparison settings |
| --- | --- | --- |
| Stable | `rustfmt 1.10.0-stable (58cf8f9c35 2026-10-13)` | Parsing edition 2024; all four supported style editions; default and custom layout settings described below |
| Nightly | `rustfmt 1.11.0-nightly (19833af722 2026-10-20)` | Parsing and style editions 2024; `imports_granularity = "Crate"` and `group_imports = "StdExternalCrate"`; other settings at their defaults |

Stable compatibility checks verify both rustfmt checks and byte equality when alternating the tools. They cover all four supported style editions, default layout settings, widths 80, 100, and 120, two-space indentation, and hard tabs with tab widths 2 and 4. Custom layout checks include LF and CRLF, nested imports, merged imports, and Unicode identifiers.

Compatibility fixtures use parsing edition 2024; compile/run fixtures also cover older parsing editions.
