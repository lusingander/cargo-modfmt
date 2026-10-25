use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use assert_cmd::{Command, cargo::cargo_bin_cmd};

pub fn fixture_path(path: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path)
}

pub fn fixture(directory: &str, name: &str) -> Vec<u8> {
    fs::read(fixture_path(directory).join(name)).unwrap()
}

pub fn command(cwd: &Path) -> Command {
    let mut command = cargo_bin_cmd!();
    command
        .arg("modfmt")
        .current_dir(cwd)
        .timeout(Duration::from_secs(5));
    command
}

pub fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
