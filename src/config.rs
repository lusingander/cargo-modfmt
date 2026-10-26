use std::{
    fs, io,
    path::{Path, PathBuf},
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::{Deserialize, de::DeserializeOwned};

use crate::{
    file::FileError,
    format::EditionOverrides,
    organize::{SourceEdition, StyleEdition},
    settings::{FileSearch, Origin, Resolved, SearchStop},
};

pub struct Config {
    pub path: Option<PathBuf>,
    pub search: Option<FileSearch>,
    pub editions: EditionOverrides,
    pub respect_gitignore: Resolved<bool>,
    pub exclude: Resolved<Vec<String>>,
    pub additional_exclude: Vec<String>,
    pub exclusions: Exclusions,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Document {
    files: Files,
    format: Format,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Files {
    respect_gitignore: Option<bool>,
    exclude: Option<Vec<String>>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Format {
    edition: Option<String>,
    style_edition: Option<String>,
}

impl Config {
    pub fn load(
        cwd: &Path,
        explicit: Option<&Path>,
        overrides: EditionOverrides,
        exclude: &[String],
    ) -> Result<Self, FileError> {
        let search = match explicit {
            Some(_) => None,
            None => Some(find_ancestor_file(cwd, &["modfmt.toml"])?),
        };
        let path = match explicit {
            Some(path) => Some(cwd.join(path)),
            None => search.as_ref().unwrap().path.clone(),
        };
        let document = match &path {
            Some(path) => read_document(path)?,
            None => Document::default(),
        };
        let context = path.as_deref().unwrap_or(cwd);
        let configured_edition = document
            .format
            .edition
            .as_deref()
            .map(str::parse::<SourceEdition>)
            .transpose()
            .map_err(|error| {
                FileError::new(
                    context,
                    "validate format.edition in",
                    io::Error::other(error),
                )
            })?;
        let configured_style = document
            .format
            .style_edition
            .as_deref()
            .map(str::parse::<StyleEdition>)
            .transpose()
            .map_err(|error| {
                FileError::new(
                    context,
                    "validate format.style_edition in",
                    io::Error::other(error),
                )
            })?;
        let base = match &path {
            Some(path) => path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new(".")),
            None => cwd,
        };
        let base = fs::canonicalize(base)
            .map_err(|error| FileError::new(base, "resolve configuration directory", error))?;
        let respect_gitignore = Resolved::configured(
            document.files.respect_gitignore,
            true,
            path.as_deref(),
            "files.respect_gitignore",
        );
        let configured_exclude = Resolved::configured(
            document.files.exclude,
            Vec::new(),
            path.as_deref(),
            "files.exclude",
        );
        let patterns = configured_exclude.value.iter().chain(exclude).cloned();
        let exclusions = Exclusions::new(base, patterns).map_err(|error| {
            FileError::new(context, "validate exclusions in", io::Error::other(error))
        })?;
        Ok(Self {
            editions: EditionOverrides {
                edition: overrides.edition.or_else(|| {
                    configured_edition.map(|value| {
                        Resolved::new(
                            value,
                            Origin::file(path.as_ref().unwrap(), "format.edition"),
                        )
                    })
                }),
                style_edition: overrides.style_edition.or_else(|| {
                    configured_style.map(|value| {
                        Resolved::new(
                            value,
                            Origin::file(path.as_ref().unwrap(), "format.style_edition"),
                        )
                    })
                }),
            },
            path,
            search,
            respect_gitignore,
            exclude: configured_exclude,
            additional_exclude: exclude.to_vec(),
            exclusions,
        })
    }
}

pub fn find_ancestor_file(start: &Path, names: &[&str]) -> Result<FileSearch, FileError> {
    for directory in start.ancestors() {
        for name in names {
            let path = directory.join(name);
            if exists(&path)? {
                return Ok(FileSearch {
                    path: Some(path),
                    start: start.to_owned(),
                    stopped_at: directory.to_owned(),
                    reason: SearchStop::Found,
                });
            }
        }
        let repository_root = exists(&directory.join(".git"))?;
        if repository_root || directory.parent().is_none() {
            return Ok(FileSearch {
                path: None,
                start: start.to_owned(),
                stopped_at: directory.to_owned(),
                reason: if repository_root {
                    SearchStop::RepositoryRoot
                } else {
                    SearchStop::FilesystemRoot
                },
            });
        }
    }
    unreachable!("ancestor search always reaches a root")
}

pub fn read_document<T: DeserializeOwned>(path: &Path) -> Result<T, FileError> {
    let metadata =
        fs::metadata(path).map_err(|error| FileError::new(path, "inspect configuration", error))?;
    if !metadata.is_file() {
        return Err(FileError::new(
            path,
            "read configuration",
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected a regular configuration file",
            ),
        ));
    }
    let text = fs::read_to_string(path)
        .map_err(|error| FileError::new(path, "read configuration", error))?;
    toml::from_str(&text).map_err(|error| FileError::new(path, "parse configuration", error))
}

pub fn exists(path: &Path) -> Result<bool, FileError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(FileError::new(path, "inspect path", error)),
    }
}

pub struct Exclusions {
    pub base: PathBuf,
    matcher: Gitignore,
}

impl Exclusions {
    pub fn new(base: PathBuf, patterns: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut builder = GitignoreBuilder::new(".");
        builder.allow_unclosed_class(false);
        for pattern in patterns {
            validate_pattern(&pattern)?;
            builder
                .add_line(None, &pattern)
                .map_err(|error| error.to_string())?;
        }
        let matcher = builder.build().map_err(|error| error.to_string())?;
        Ok(Self { base, matcher })
    }

    pub fn matches(&self, canonical: &Path, is_dir: bool) -> bool {
        // Both paths are absolute. Preserve non-UTF-8 names while computing ../ paths.
        let relative = self
            .base
            .ancestors()
            .enumerate()
            .find_map(|(depth, ancestor)| {
                canonical.strip_prefix(ancestor).ok().map(|suffix| {
                    let mut path = PathBuf::new();
                    for _ in 0..depth {
                        path.push("..");
                    }
                    path.push(suffix);
                    path
                })
            })
            .unwrap_or_else(|| canonical.to_owned());
        // Using a relative candidate also supports targets outside the configuration directory.
        self.matcher
            .matched_path_or_any_parents(relative, is_dir)
            .is_ignore()
    }
}

fn validate_pattern(pattern: &str) -> Result<(), String> {
    if pattern.starts_with('!') {
        return Err(
            "exclusions cannot re-include paths with '!'; use \\! for a literal '!'".into(),
        );
    }
    if pattern.contains(['\n', '\r']) {
        return Err("an exclusion pattern must be a single line".into());
    }
    Ok(())
}

pub fn parse_exclude(pattern: &str) -> Result<String, String> {
    Exclusions::new(PathBuf::from("."), [pattern.to_owned()])?;
    Ok(pattern.to_owned())
}
