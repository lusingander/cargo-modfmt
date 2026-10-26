use std::{
    error::Error,
    fmt,
    fs::{self, File, Permissions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use tempfile::{Builder, NamedTempFile};

use crate::organize::{FormatOptions, organize};

pub enum FileMode {
    Check,
    Write,
}

#[derive(Debug)]
pub struct FileError {
    path: PathBuf,
    operation: &'static str,
    cause: Box<dyn Error>,
    cleanup: Option<(PathBuf, io::Error)>,
}

impl FileError {
    pub fn new(path: &Path, operation: &'static str, cause: impl Error + 'static) -> Self {
        Self {
            path: path.to_owned(),
            operation,
            cause: Box::new(cause),
            cleanup: None,
        }
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot {} {:?}: {}",
            self.operation, self.path, self.cause
        )?;
        if let Some((path, error)) = &self.cleanup {
            write!(f, "; cannot remove temporary file {path:?}: {error}")?;
        }
        Ok(())
    }
}

impl Error for FileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.cause.as_ref())
    }
}

struct Source<'a> {
    path: &'a Path,
    bytes: Vec<u8>,
    permissions: Permissions,
}

impl<'a> Source<'a> {
    fn read(path: &'a Path) -> Result<Self, FileError> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| FileError::new(path, "inspect file", error))?;
        if !metadata.file_type().is_file() {
            return Err(FileError::new(
                path,
                "read file",
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "expected a regular file, not a symlink or special file",
                ),
            ));
        }
        let mut file =
            File::open(path).map_err(|error| FileError::new(path, "open file", error))?;
        let permissions = file
            .metadata()
            .map_err(|error| FileError::new(path, "read permissions", error))?
            .permissions();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|error| FileError::new(path, "read file", error))?;
        Ok(Self {
            path,
            bytes,
            permissions,
        })
    }

    fn verify(&self) -> Result<(), FileError> {
        let current = Self::read(self.path)?;
        if current.bytes != self.bytes || current.permissions != self.permissions {
            return Err(FileError::new(
                self.path,
                "recheck file",
                io::Error::other("file contents or permissions changed since it was read"),
            ));
        }
        Ok(())
    }

    fn write_temporary(
        &self,
        temporary: &mut NamedTempFile,
        output: &[u8],
    ) -> Result<(), FileError> {
        temporary
            .write_all(output)
            .map_err(|error| FileError::new(self.path, "write temporary file for", error))?;
        temporary
            .as_file()
            .set_permissions(self.permissions.clone())
            .map_err(|error| FileError::new(self.path, "set temporary permissions for", error))?;
        self.sync_temporary(temporary)
    }

    fn sync_temporary(&self, temporary: &NamedTempFile) -> Result<(), FileError> {
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| FileError::new(self.path, "sync temporary file for", error))
    }

    fn replace_with(
        &self,
        prepare: impl FnOnce(&mut NamedTempFile) -> Result<(), FileError>,
    ) -> Result<(), FileError> {
        let parent = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut temporary = Builder::new()
            .prefix(".modfmt-")
            .tempfile_in(parent)
            .map_err(|error| FileError::new(self.path, "create temporary file for", error))?;
        if let Err(error) = prepare(&mut temporary).and_then(|()| self.verify()) {
            return Err(cleanup(temporary, error));
        }
        persist(temporary, self.path)
    }
}

fn cleanup(temporary: NamedTempFile, mut error: FileError) -> FileError {
    let path = temporary.path().to_owned();
    if let Err(cause) = temporary.close() {
        error.cleanup = Some((path, cause));
    }
    error
}

fn persist(temporary: NamedTempFile, path: &Path) -> Result<(), FileError> {
    match temporary.persist(path) {
        Ok(_) => Ok(()),
        Err(error) => Err(cleanup(
            error.file,
            FileError::new(path, "replace file", error.error),
        )),
    }
}

pub fn process(path: &Path, options: FormatOptions, mode: FileMode) -> Result<bool, FileError> {
    let source = Source::read(path)?;
    let text = std::str::from_utf8(&source.bytes)
        .map_err(|error| FileError::new(path, "decode UTF-8 in", error))?;
    let output =
        organize(text, options).map_err(|error| FileError::new(path, "parse source in", error))?;
    let changed = output.as_bytes() != source.bytes;
    if changed && matches!(mode, FileMode::Write) {
        if source.permissions.readonly() {
            return Err(FileError::new(
                path,
                "update file",
                io::Error::new(io::ErrorKind::PermissionDenied, "file is read-only"),
            ));
        }
        source.replace_with(|temporary| source.write_temporary(temporary, output.as_bytes()))?;
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use rstest::rstest;
    use tempfile::{TempDir, tempdir};

    fn original() -> (TempDir, PathBuf, Vec<u8>) {
        let directory = tempdir().unwrap();
        let path = directory.path().join("main.rs");
        let bytes = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/cases/mod_use_groups/input.rs"),
        )
        .unwrap();
        fs::write(&path, &bytes).unwrap();
        (directory, path, bytes)
    }

    fn assert_no_temporary(directory: &Path) {
        assert!(fs::read_dir(directory).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".modfmt-")
        }));
    }

    #[test]
    fn partial_preparation_failure_preserves_original() {
        let (directory, path, bytes) = original();
        let source = Source::read(&path).unwrap();
        let error = source
            .replace_with(|temporary| {
                temporary.write_all(&bytes[..8]).unwrap();
                Err(FileError::new(
                    &path,
                    "prepare temporary file for",
                    io::Error::other("injected failure after partial write"),
                ))
            })
            .unwrap_err();
        assert!(error.to_string().contains("injected failure"));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_no_temporary(directory.path());
    }

    #[test]
    fn temporary_write_error_preserves_original() {
        let (directory, path, bytes) = original();
        let source = Source::read(&path).unwrap();
        let error = source
            .replace_with(|temporary| {
                let read_only = File::open(temporary.path()).unwrap();
                *temporary.as_file_mut() = read_only;
                source.write_temporary(temporary, &bytes)
            })
            .unwrap_err();
        assert!(error.to_string().contains("write temporary file"));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_no_temporary(directory.path());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn temporary_permissions_error_preserves_original() {
        use std::os::fd::OwnedFd;
        use std::os::unix::net::UnixStream;

        let (directory, path, bytes) = original();
        let source = Source::read(&path).unwrap();
        let (_reader, writer) = UnixStream::pair().unwrap();
        let error = source
            .replace_with(|temporary| {
                // macOS rejects fchmod on a socket after accepting the write.
                *temporary.as_file_mut() = File::from(OwnedFd::from(writer));
                source.write_temporary(temporary, &bytes)
            })
            .unwrap_err();
        assert!(
            error.to_string().contains("set temporary permissions"),
            "{error}"
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_no_temporary(directory.path());
    }

    #[cfg(unix)]
    #[test]
    fn temporary_sync_error_preserves_original() {
        use std::os::fd::OwnedFd;
        use std::os::unix::net::UnixStream;

        let (directory, path, bytes) = original();
        let source = Source::read(&path).unwrap();
        let (_reader, writer) = UnixStream::pair().unwrap();
        let error = source
            .replace_with(|temporary| {
                *temporary.as_file_mut() = File::from(OwnedFd::from(writer));
                source.sync_temporary(temporary)
            })
            .unwrap_err();
        assert!(error.to_string().contains("sync temporary file"), "{error}");
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_no_temporary(directory.path());
    }

    #[test]
    fn failed_replace_keeps_destination_and_removes_temporary() {
        let directory = tempdir().unwrap();
        let destination = directory.path().join("destination");
        fs::create_dir(&destination).unwrap();
        let marker = destination.join("keep");
        fs::write(&marker, b"preserved").unwrap();
        let temporary = Builder::new()
            .prefix(".modfmt-")
            .tempfile_in(directory.path())
            .unwrap();
        let error = persist(temporary, &destination).unwrap_err();
        assert!(error.to_string().contains("replace file"));
        assert_eq!(fs::read(marker).unwrap(), b"preserved");
        assert_no_temporary(directory.path());
    }

    #[test]
    fn failed_cleanup_reports_remaining_path_and_original_error() {
        let (directory, path, bytes) = original();
        let source = Source::read(&path).unwrap();
        let mut remaining = PathBuf::new();
        let error = source
            .replace_with(|temporary| {
                remaining = temporary.path().to_owned();
                fs::remove_file(&remaining).unwrap();
                fs::create_dir(&remaining).unwrap();
                Err(FileError::new(
                    &path,
                    "prepare temporary file for",
                    io::Error::other("preparation failed"),
                ))
            })
            .unwrap_err();
        let diagnostic = error.to_string();
        assert!(diagnostic.contains("preparation failed"));
        assert!(diagnostic.contains("cannot remove temporary file"));
        assert!(diagnostic.contains(&format!("{remaining:?}")));
        assert!(remaining.is_dir());
        assert_eq!(fs::read(&path).unwrap(), bytes);
        fs::remove_dir(remaining).unwrap();
        assert_no_temporary(directory.path());
    }

    #[cfg(unix)]
    #[rstest]
    #[case::contents("contents")]
    #[case::removed("removed")]
    #[case::symlink("symlink")]
    #[case::permissions("permissions")]
    fn changed_source_is_not_replaced(#[case] change: &str) {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let (directory, path, bytes) = original();
        let source = Source::read(&path).unwrap();
        let output = organize(
            std::str::from_utf8(&bytes).unwrap(),
            FormatOptions {
                edition: crate::organize::SourceEdition::E2024,
                style_edition: crate::organize::StyleEdition::E2024,
                layout: Default::default(),
            },
        )
        .unwrap();
        let other = directory.path().join("other.rs");
        fs::write(&other, &bytes).unwrap();
        let error = source
            .replace_with(|temporary| {
                source.write_temporary(temporary, output.as_bytes())?;
                match change {
                    "contents" => fs::write(&path, [&bytes[..], b"\n"].concat()).unwrap(),
                    "removed" => fs::remove_file(&path).unwrap(),
                    "symlink" => {
                        fs::remove_file(&path).unwrap();
                        symlink(&other, &path).unwrap();
                    }
                    "permissions" => {
                        fs::set_permissions(&path, Permissions::from_mode(0o444)).unwrap()
                    }
                    _ => unreachable!(),
                }
                Ok(())
            })
            .unwrap_err();
        assert!(error.cleanup.is_none());
        match change {
            "contents" => assert_eq!(fs::read(&path).unwrap(), [&bytes[..], b"\n"].concat()),
            "removed" => assert!(!path.exists()),
            "symlink" => assert_eq!(fs::read_link(&path).unwrap(), other),
            "permissions" => assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o444
            ),
            _ => unreachable!(),
        }
        assert_eq!(fs::read(other).unwrap(), bytes);
        assert_no_temporary(directory.path());
    }
}
