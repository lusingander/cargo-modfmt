use std::{
    fmt::Display,
    fs,
    io::{self, Write},
    path::Path,
};

use crate::{
    config::Config,
    file::FileError,
    format::{FormatResolver, ResolvedFormat},
    settings::{FileSearch, Origin, Resolved},
};

pub fn run(
    cwd: &Path,
    path: Option<&Path>,
    config: &Config,
    resolver: &mut FormatResolver,
) -> Result<(), FileError> {
    let path = path.unwrap_or(cwd);
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| FileError::new(path, "inspect configuration target", error))?;
    if !metadata.is_file() && !metadata.is_dir() {
        return Err(FileError::new(
            path,
            "inspect configuration target",
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a regular file or directory",
            ),
        ));
    }
    let target = fs::canonicalize(path)
        .map_err(|error| FileError::new(path, "resolve configuration target", error))?;
    let directory = if metadata.is_dir() {
        target.as_path()
    } else {
        target.parent().unwrap()
    };
    let resolved = resolver.for_directory(directory)?;
    let mut output = io::stdout().lock();
    write_report(&mut output, cwd, &target, config, resolved)
        .and_then(|()| output.flush())
        .map_err(|error| FileError::new(path, "report configuration for", error))
}

fn write_report(
    output: &mut impl Write,
    cwd: &Path,
    target: &Path,
    config: &Config,
    resolved: &ResolvedFormat,
) -> io::Result<()> {
    writeln!(output, "target: {target:?}")?;
    writeln!(output, "working directory: {cwd:?}")?;
    writeln!(
        output,
        "format lookup directory: {:?}",
        resolved.rustfmt.start
    )?;
    writeln!(output)?;

    match &config.search {
        Some(search) => write_search(output, "modfmt config", search)?,
        None => {
            writeln!(
                output,
                "modfmt config: {:?} (explicit)",
                config.path.as_ref().unwrap()
            )?;
            writeln!(output, "  lookup not performed (--config)")?;
        }
    }
    match &resolved.cargo {
        Some(search) => write_search(output, "Cargo manifest", search)?,
        None => writeln!(output, "Cargo manifest: not searched (explicit edition)")?,
    }
    write_search(output, "rustfmt config", &resolved.rustfmt)?;
    writeln!(output)?;

    write_setting(output, "edition", &resolved.edition)?;
    write_setting(output, "style_edition", &resolved.style_edition)?;
    write_setting(output, "max_width", &resolved.max_width)?;
    write_setting(output, "tab_spaces", &resolved.tab_spaces)?;
    write_setting(output, "hard_tabs", &resolved.hard_tabs)?;
    write_setting(output, "files.respect_gitignore", &config.respect_gitignore)?;
    let patterns: Vec<_> = config
        .exclude
        .value
        .iter()
        .chain(&config.additional_exclude)
        .collect();
    writeln!(output, "files.exclude: {patterns:?}")?;
    if patterns.is_empty() {
        writeln!(output, "  {}", config.exclude.origin)?;
    } else {
        for pattern in &config.exclude.value {
            writeln!(output, "  {pattern:?}: {}", config.exclude.origin)?;
        }
        for pattern in &config.additional_exclude {
            writeln!(
                output,
                "  {pattern:?}: {}",
                Origin::CommandLine("--exclude")
            )?;
        }
    }
    writeln!(
        output,
        "exclusion base directory: {:?}",
        config.exclusions.base
    )?;
    writeln!(output, "file selection: not evaluated")
}

fn write_setting<T: Display>(
    output: &mut impl Write,
    name: &str,
    setting: &Resolved<T>,
) -> io::Result<()> {
    writeln!(output, "{name}: {}", setting.value)?;
    writeln!(output, "  {}", setting.origin)?;
    writeln!(output)
}

fn write_search(output: &mut impl Write, name: &str, search: &FileSearch) -> io::Result<()> {
    match &search.path {
        Some(path) => writeln!(output, "{name}: {path:?} (automatic)")?,
        None => writeln!(output, "{name}: none")?,
    }
    writeln!(
        output,
        "  lookup from {:?}; stopped at {:?} ({})",
        search.start, search.stopped_at, search.reason
    )
}
