use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::Deserialize;

use crate::{
    config::{exists, find_ancestor_file, read_document},
    file::FileError,
    organize::{FormatOptions, LayoutOptions, SourceEdition, StyleEdition},
};

#[derive(Clone, Copy)]
pub struct EditionOverrides {
    pub edition: Option<SourceEdition>,
    pub style_edition: Option<StyleEdition>,
}

pub struct FormatResolver {
    overrides: EditionOverrides,
    directories: HashMap<PathBuf, FormatOptions>,
}

#[derive(Default, Deserialize)]
struct RustfmtDocument {
    edition: Option<String>,
    style_edition: Option<String>,
    max_width: Option<usize>,
    tab_spaces: Option<usize>,
    hard_tabs: Option<bool>,
}

#[derive(Deserialize)]
struct CargoDocument {
    package: Option<Package>,
    workspace: Option<Workspace>,
}

#[derive(Deserialize)]
struct Package {
    edition: Option<PackageEdition>,
    workspace: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum PackageEdition {
    Value(String),
    Inherited(EditionInheritance),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditionInheritance {
    workspace: bool,
}

#[derive(Deserialize)]
struct Workspace {
    package: Option<WorkspacePackage>,
}

#[derive(Deserialize)]
struct WorkspacePackage {
    edition: Option<String>,
}

impl FormatResolver {
    pub fn new(overrides: EditionOverrides) -> Self {
        Self {
            overrides,
            directories: HashMap::new(),
        }
    }

    pub fn for_file(&mut self, path: &Path) -> Result<FormatOptions, FileError> {
        let canonical = fs::canonicalize(path).map_err(|error| {
            FileError::new(path, "resolve source path for format settings in", error)
        })?;
        self.for_directory(canonical.parent().unwrap())
    }

    pub fn for_directory(&mut self, directory: &Path) -> Result<FormatOptions, FileError> {
        let directory = fs::canonicalize(directory).map_err(|error| {
            FileError::new(directory, "resolve format settings directory", error)
        })?;
        if let Some(options) = self.directories.get(&directory) {
            return Ok(*options);
        }
        let cargo = if self.overrides.edition.is_none() {
            match find_ancestor_file(&directory, &["Cargo.toml"])? {
                Some(path) => cargo_edition(&path)?,
                None => None,
            }
        } else {
            None
        };
        let mut rustfmt_edition = None;
        let mut rustfmt_style = None;
        let mut layout = LayoutOptions::default();
        let needs_editions = self.overrides.style_edition.is_none()
            || (self.overrides.edition.is_none() && cargo.is_none());
        if let Some(path) = find_ancestor_file(&directory, &[".rustfmt.toml", "rustfmt.toml"])? {
            let document: RustfmtDocument = read_document(&path)?;
            layout = LayoutOptions {
                max_width: document.max_width.unwrap_or(layout.max_width),
                tab_spaces: document.tab_spaces.unwrap_or(layout.tab_spaces),
                hard_tabs: document.hard_tabs.unwrap_or(layout.hard_tabs),
            };
            if layout.hard_tabs && layout.tab_spaces == 0 {
                return Err(FileError::new(
                    &path,
                    "validate format settings in",
                    io::Error::other("tab_spaces must be greater than zero when hard_tabs is true"),
                ));
            }
            if needs_editions {
                rustfmt_edition = document
                    .edition
                    .as_deref()
                    .map(|value| parse_setting(&path, "edition", value))
                    .transpose()?;
                rustfmt_style = document
                    .style_edition
                    .as_deref()
                    .map(|value| parse_setting(&path, "style_edition", value))
                    .transpose()?;
            }
        }
        let edition = self
            .overrides
            .edition
            .or(cargo)
            .or(rustfmt_edition)
            .unwrap_or(SourceEdition::E2024);
        let options = FormatOptions {
            edition,
            style_edition: self
                .overrides
                .style_edition
                .or(rustfmt_style)
                .unwrap_or_else(|| edition.into()),
            layout,
        };
        self.directories.insert(directory, options);
        Ok(options)
    }
}

fn cargo_edition(path: &Path) -> Result<Option<SourceEdition>, FileError> {
    let document: CargoDocument = read_document(path)?;
    let Some(package) = document.package else {
        return Ok(None);
    };
    match package.edition {
        Some(PackageEdition::Value(value)) => {
            parse_setting(path, "package.edition", &value).map(Some)
        }
        None => Ok(Some(SourceEdition::E2015)),
        Some(PackageEdition::Inherited(inheritance)) => {
            if !inheritance.workspace {
                return Err(edition_error(
                    path,
                    "package.edition.workspace must be true",
                ));
            }
            let directory = path.parent().unwrap();
            if let Some(workspace) = package.workspace {
                let path = directory.join(workspace).join("Cargo.toml");
                let document: CargoDocument = read_document(&path)?;
                let workspace = document
                    .workspace
                    .ok_or_else(|| edition_error(&path, "expected a workspace manifest"))?;
                return inherited_edition(&path, workspace).map(Some);
            }
            if let Some(workspace) = document.workspace {
                return inherited_edition(path, workspace).map(Some);
            }
            if !exists(&directory.join(".git"))? {
                for parent in directory.ancestors().skip(1) {
                    let manifest = parent.join("Cargo.toml");
                    if exists(&manifest)? {
                        let document: CargoDocument = read_document(&manifest)?;
                        if let Some(workspace) = document.workspace {
                            return inherited_edition(&manifest, workspace).map(Some);
                        }
                    }
                    if exists(&parent.join(".git"))? {
                        break;
                    }
                }
            }
            Err(edition_error(
                path,
                "cannot find workspace for package.edition.workspace",
            ))
        }
    }
}

fn inherited_edition(path: &Path, workspace: Workspace) -> Result<SourceEdition, FileError> {
    let value = workspace
        .package
        .and_then(|package| package.edition)
        .ok_or_else(|| edition_error(path, "workspace.package.edition is missing"))?;
    parse_setting(path, "workspace.package.edition", &value)
}

fn edition_error(path: &Path, message: &str) -> FileError {
    FileError::new(
        path,
        "resolve edition in",
        io::Error::other(message.to_owned()),
    )
}

fn parse_setting<T: FromStr<Err = String>>(
    path: &Path,
    key: &str,
    value: &str,
) -> Result<T, FileError> {
    value.parse().map_err(|error| {
        FileError::new(
            path,
            "validate edition settings in",
            io::Error::other(format!("{key}: {error}")),
        )
    })
}
