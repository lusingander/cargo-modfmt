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
    settings::{FileSearch, Origin, Resolved},
};

#[derive(Clone)]
pub struct EditionOverrides {
    pub edition: Option<Resolved<SourceEdition>>,
    pub style_edition: Option<Resolved<StyleEdition>>,
}

pub struct FormatResolver {
    overrides: EditionOverrides,
    directories: HashMap<PathBuf, ResolvedFormat>,
}

pub struct ResolvedFormat {
    pub edition: Resolved<SourceEdition>,
    pub style_edition: Resolved<StyleEdition>,
    pub max_width: Resolved<usize>,
    pub tab_spaces: Resolved<usize>,
    pub hard_tabs: Resolved<bool>,
    pub cargo: Option<FileSearch>,
    pub rustfmt: FileSearch,
}

impl ResolvedFormat {
    pub fn options(&self) -> FormatOptions {
        FormatOptions {
            edition: self.edition.value,
            style_edition: self.style_edition.value,
            layout: LayoutOptions {
                max_width: self.max_width.value,
                tab_spaces: self.tab_spaces.value,
                hard_tabs: self.hard_tabs.value,
            },
        }
    }
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

    pub fn for_file(&mut self, path: &Path) -> Result<&ResolvedFormat, FileError> {
        let canonical = fs::canonicalize(path).map_err(|error| {
            FileError::new(path, "resolve source path for format settings in", error)
        })?;
        self.for_directory(canonical.parent().unwrap())
    }

    pub fn for_directory(&mut self, directory: &Path) -> Result<&ResolvedFormat, FileError> {
        let directory = fs::canonicalize(directory).map_err(|error| {
            FileError::new(directory, "resolve format settings directory", error)
        })?;
        if !self.directories.contains_key(&directory) {
            let resolved = self.resolve_directory(&directory)?;
            self.directories.insert(directory.clone(), resolved);
        }
        Ok(self.directories.get(&directory).unwrap())
    }

    fn resolve_directory(&self, directory: &Path) -> Result<ResolvedFormat, FileError> {
        let cargo = if self.overrides.edition.is_none() {
            Some(find_ancestor_file(directory, &["Cargo.toml"])?)
        } else {
            None
        };
        let cargo_edition = match cargo.as_ref().and_then(|search| search.path.as_deref()) {
            Some(path) => cargo_edition(path)?,
            None => None,
        };
        let needs_editions = self.overrides.style_edition.is_none()
            || (self.overrides.edition.is_none() && cargo_edition.is_none());
        let rustfmt = find_ancestor_file(directory, &[".rustfmt.toml", "rustfmt.toml"])?;
        let path = rustfmt.path.as_deref();
        let document: RustfmtDocument = match path {
            Some(path) => read_document(path)?,
            None => RustfmtDocument::default(),
        };
        let defaults = LayoutOptions::default();
        let max_width =
            Resolved::configured(document.max_width, defaults.max_width, path, "max_width");
        let tab_spaces =
            Resolved::configured(document.tab_spaces, defaults.tab_spaces, path, "tab_spaces");
        let hard_tabs =
            Resolved::configured(document.hard_tabs, defaults.hard_tabs, path, "hard_tabs");
        if hard_tabs.value && tab_spaces.value == 0 {
            return Err(FileError::new(
                path.unwrap(),
                "validate format settings in",
                io::Error::other("tab_spaces must be greater than zero when hard_tabs is true"),
            ));
        }
        let mut rustfmt_edition = None;
        let mut rustfmt_style = None;
        if needs_editions {
            if let Some(value) = document.edition {
                let path = path.unwrap();
                rustfmt_edition = Some(Resolved::new(
                    parse_setting(path, "edition", &value)?,
                    Origin::file(path, "edition"),
                ));
            }
            if let Some(value) = document.style_edition {
                let path = path.unwrap();
                rustfmt_style = Some(Resolved::new(
                    parse_setting(path, "style_edition", &value)?,
                    Origin::file(path, "style_edition"),
                ));
            }
        }
        let edition = self
            .overrides
            .edition
            .clone()
            .or(cargo_edition)
            .or(rustfmt_edition)
            .unwrap_or_else(|| Resolved::new(SourceEdition::E2024, Origin::Default(None)));
        let style_edition = self
            .overrides
            .style_edition
            .clone()
            .or(rustfmt_style)
            .unwrap_or_else(|| Resolved::new(edition.value.into(), Origin::ParsingEdition));
        Ok(ResolvedFormat {
            edition,
            style_edition,
            max_width,
            tab_spaces,
            hard_tabs,
            cargo,
            rustfmt,
        })
    }
}

fn cargo_edition(path: &Path) -> Result<Option<Resolved<SourceEdition>>, FileError> {
    let document: CargoDocument = read_document(path)?;
    let Some(package) = document.package else {
        return Ok(None);
    };
    match package.edition {
        Some(PackageEdition::Value(value)) => parse_setting(path, "package.edition", &value)
            .map(|value| Some(Resolved::new(value, Origin::file(path, "package.edition")))),
        None => Ok(Some(Resolved::new(
            SourceEdition::E2015,
            Origin::CargoDefault(path.to_owned()),
        ))),
        Some(PackageEdition::Inherited(inheritance)) => {
            if !inheritance.workspace {
                return Err(edition_error(
                    path,
                    "package.edition.workspace must be true",
                ));
            }
            let directory = path.parent().unwrap();
            if let Some(workspace) = package.workspace {
                let workspace_path = directory.join(workspace).join("Cargo.toml");
                let document: CargoDocument = read_document(&workspace_path)?;
                let workspace = document.workspace.ok_or_else(|| {
                    edition_error(&workspace_path, "expected a workspace manifest")
                })?;
                return inherited_edition(&workspace_path, workspace, path).map(Some);
            }
            if let Some(workspace) = document.workspace {
                return inherited_edition(path, workspace, path).map(Some);
            }
            if !exists(&directory.join(".git"))? {
                for parent in directory.ancestors().skip(1) {
                    let manifest = parent.join("Cargo.toml");
                    if exists(&manifest)? {
                        let document: CargoDocument = read_document(&manifest)?;
                        if let Some(workspace) = document.workspace {
                            return inherited_edition(&manifest, workspace, path).map(Some);
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

fn inherited_edition(
    path: &Path,
    workspace: Workspace,
    package: &Path,
) -> Result<Resolved<SourceEdition>, FileError> {
    let value = workspace
        .package
        .and_then(|package| package.edition)
        .ok_or_else(|| edition_error(path, "workspace.package.edition is missing"))?;
    let value = parse_setting(path, "workspace.package.edition", &value)?;
    Ok(Resolved::new(
        value,
        Origin::Workspace {
            package: package.to_owned(),
            workspace: path.to_owned(),
        },
    ))
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
