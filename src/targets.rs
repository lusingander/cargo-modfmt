use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::{
    config::{Config, exists},
    file::FileError,
};

pub fn collect(paths: &[PathBuf], config: &Config) -> Result<Vec<PathBuf>, FileError> {
    let mut seen = HashSet::new();
    let mut targets = Vec::new();
    for path in paths {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| FileError::new(path, "inspect file", error))?;
        if !metadata.is_file() && !metadata.is_dir() {
            return Err(invalid_type(path));
        }
        let canonical =
            fs::canonicalize(path).map_err(|error| FileError::new(path, "resolve file", error))?;
        if metadata.is_dir() {
            if default_excluded(&canonical) || config.exclusions.matches(&canonical, true) {
                continue;
            }
            let Some(mut rules) = parent_rules(&canonical, config.respect_gitignore)? else {
                continue;
            };
            let mut discovered = Vec::new();
            walk(path, &canonical, config, &mut rules, &mut discovered)?;
            discovered.sort_by(|left, right| left.0.cmp(&right.0));
            for (display, canonical) in discovered {
                if seen.insert(canonical) {
                    targets.push(display);
                }
            }
        } else if !config.exclusions.matches(&canonical, false) && seen.insert(canonical) {
            targets.push(path.clone());
        }
    }
    Ok(targets)
}

fn walk(
    display: &Path,
    canonical: &Path,
    config: &Config,
    rules: &mut Vec<Gitignore>,
    targets: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), FileError> {
    if default_excluded(canonical)
        || config.exclusions.matches(canonical, true)
        || ignored(canonical, true, rules)
    {
        return Ok(());
    }
    let added = if config.respect_gitignore {
        load_rules(canonical, rules)?
    } else {
        false
    };
    let entries =
        fs::read_dir(display).map_err(|error| FileError::new(display, "read directory", error))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| FileError::new(display, "read directory entry in", error))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| FileError::new(&path, "inspect entry", error))?;
        if !file_type.is_dir() && !file_type.is_file() {
            continue;
        }
        if default_excluded(&path) {
            continue;
        }
        if file_type.is_file() && path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let canonical = fs::canonicalize(&path)
            .map_err(|error| FileError::new(&path, "resolve entry", error))?;
        if file_type.is_dir() {
            walk(&path, &canonical, config, rules, targets)?;
        } else if !config.exclusions.matches(&canonical, false)
            && !ignored(&canonical, false, rules)
        {
            targets.push((path, canonical));
        }
    }
    if added {
        rules.pop();
    }
    Ok(())
}

fn default_excluded(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name == "target" || name.as_encoded_bytes().starts_with(b"."))
}

fn ignored(path: &Path, is_dir: bool, rules: &[Gitignore]) -> bool {
    rules
        .iter()
        .rev()
        .map(|rules| rules.matched(path, is_dir))
        .find(|matched| !matched.is_none())
        .is_some_and(|matched| matched.is_ignore())
}

fn parent_rules(root: &Path, enabled: bool) -> Result<Option<Vec<Gitignore>>, FileError> {
    let mut rules = Vec::new();
    if !enabled {
        return Ok(Some(rules));
    }
    let mut ancestors = Vec::new();
    let mut repository = false;
    for directory in root.ancestors() {
        ancestors.push(directory);
        if exists(&directory.join(".git"))? {
            repository = true;
            break;
        }
    }
    if repository {
        for directory in ancestors.iter().rev().take(ancestors.len() - 1) {
            // An excluded ancestor cannot be re-included by rules inside it.
            if ignored(directory, true, &rules) {
                return Ok(None);
            }
            load_rules(directory, &mut rules)?;
        }
    }
    Ok(Some(rules))
}

fn load_rules(directory: &Path, rules: &mut Vec<Gitignore>) -> Result<bool, FileError> {
    let path = directory.join(".gitignore");
    if !exists(&path)? {
        return Ok(false);
    }
    let metadata =
        fs::metadata(&path).map_err(|error| FileError::new(&path, "inspect ignore file", error))?;
    if !metadata.is_file() {
        return Err(FileError::new(
            &path,
            "read ignore rules in",
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a regular ignore file",
            ),
        ));
    }
    let mut builder = GitignoreBuilder::new(directory);
    if let Some(error) = builder.add(&path) {
        return Err(FileError::new(&path, "read ignore rules in", error));
    }
    let matcher = builder
        .build()
        .map_err(|error| FileError::new(&path, "compile ignore rules in", error))?;
    rules.push(matcher);
    Ok(true)
}

fn invalid_type(path: &Path) -> FileError {
    FileError::new(
        path,
        "select input",
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file or directory, not a symlink or special file",
        ),
    )
}
