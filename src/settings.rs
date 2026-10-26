use std::{fmt, path::PathBuf};

#[derive(Clone)]
pub struct Resolved<T> {
    pub value: T,
    pub origin: Origin,
}

impl<T> Resolved<T> {
    pub fn new(value: T, origin: Origin) -> Self {
        Self { value, origin }
    }

    pub fn configured(
        value: Option<T>,
        default: T,
        path: Option<&std::path::Path>,
        key: &'static str,
    ) -> Self {
        match value {
            Some(value) => Self::new(value, Origin::file(path.unwrap(), key)),
            None => Self::new(default, Origin::Default(path.map(ToOwned::to_owned))),
        }
    }
}

#[derive(Clone)]
pub enum Origin {
    CommandLine(&'static str),
    File {
        path: PathBuf,
        key: &'static str,
    },
    Default(Option<PathBuf>),
    CargoDefault(PathBuf),
    Workspace {
        package: PathBuf,
        workspace: PathBuf,
    },
    ParsingEdition,
}

impl Origin {
    pub fn file(path: &std::path::Path, key: &'static str) -> Self {
        Self::File {
            path: path.to_owned(),
            key,
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommandLine(option) => write!(formatter, "command line {option}"),
            Self::File { path, key } => write!(formatter, "from {path:?}: {key}"),
            Self::Default(Some(path)) => {
                write!(formatter, "built-in default; omitted in {path:?}")
            }
            Self::Default(None) => formatter.write_str("built-in default"),
            Self::CargoDefault(path) => write!(
                formatter,
                "Cargo default; package.edition omitted in {path:?}"
            ),
            Self::Workspace { package, workspace } => write!(
                formatter,
                "from {workspace:?}: workspace.package.edition\n  via {package:?}: package.edition.workspace = true"
            ),
            Self::ParsingEdition => formatter.write_str("fallback to resolved edition"),
        }
    }
}

pub struct FileSearch {
    pub path: Option<PathBuf>,
    pub start: PathBuf,
    pub stopped_at: PathBuf,
    pub reason: SearchStop,
}

pub enum SearchStop {
    Found,
    RepositoryRoot,
    FilesystemRoot,
}

impl fmt::Display for SearchStop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Found => "configuration found",
            Self::RepositoryRoot => "Git repository root",
            Self::FilesystemRoot => "filesystem root",
        })
    }
}
