#![cfg(unix)]

pub mod common;

use std::{
    fs,
    os::{fd::OwnedFd, unix::net::UnixStream},
    path::Path,
    process::{Command, Output, Stdio},
};

use rstest::rstest;
use tempfile::tempdir;

use crate::common::fixture;

#[test]
fn closed_config_output_reports_failure_without_writing_files() {
    let project = tempdir().unwrap();
    std::fs::create_dir(project.path().join(".git")).unwrap();
    let output = execute(project.path(), &["config"], Stream::Stdout, Stdio::null());
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("error: cannot report configuration for")
    );
    assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 1);
}

#[derive(Clone, Copy, Debug)]
enum Stream {
    Stdout,
    Stderr,
}

fn execute(cwd: &Path, args: &[&str], closed: Stream, stdin: Stdio) -> Output {
    let (reader, writer) = UnixStream::pair().unwrap();
    drop(reader);
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!("cargo-modfmt"));
    command
        .arg("modfmt")
        .current_dir(cwd)
        .args(args)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let broken = Stdio::from(OwnedFd::from(writer));
    match closed {
        Stream::Stdout => command.stdout(broken),
        Stream::Stderr => command.stderr(broken),
    };
    // Keep the configured descriptors; assert_cmd replaces them when capturing output.
    command.output().unwrap()
}

#[rustfmt::skip]
#[rstest]
#[case::stdin_stdout("stdin", Stream::Stdout, 1, 0, "error:")]
#[case::files_stdout("files", Stream::Stdout, 1, 0, "error: cannot list file \"first.rs\":")]
#[case::check_stdout("check", Stream::Stdout, 1, 0, "error: cannot report changes for \"first.rs\":")]
#[case::write_stdout("write", Stream::Stdout, 0, 2, "reformatted: \"first.rs\"\nreformatted: \"last.rs\"\n")]
#[case::write_stderr("write", Stream::Stderr, 1, 1, "")]
fn closed_output_preserves_completed_updates_and_stops_on_failure(
    #[case] mode: &str,
    #[case] closed: Stream,
    #[case] code: i32,
    #[case] updated: usize,
    #[case] diagnostic: &str,
) {
    let project = tempdir().unwrap();
    fs::create_dir(project.path().join(".git")).unwrap();
    let input = fixture("cases/mod_use_groups", "input.rs");
    let expected = fixture("cases/mod_use_groups", "expected.rs");
    let paths = ["first.rs", "last.rs"].map(|name| project.path().join(name));
    for path in &paths {
        fs::write(path, &input).unwrap();
    }
    let modified = paths
        .each_ref()
        .map(|path| fs::metadata(path).unwrap().modified().unwrap());
    let (args, stdin) = if mode == "stdin" {
        (vec![mode], Stdio::from(fs::File::open(&paths[0]).unwrap()))
    } else {
        (vec![mode, "first.rs", "last.rs"], Stdio::null())
    };
    let output = execute(project.path(), &args, closed, stdin);
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    if code == 0 || matches!(closed, Stream::Stderr) {
        assert_eq!(stderr, diagnostic);
    } else {
        assert!(stderr.starts_with(diagnostic), "{stderr}");
    }
    for (index, path) in paths.iter().enumerate() {
        let bytes = if index < updated { &expected } else { &input };
        assert_eq!(fs::read(path).unwrap(), *bytes);
        if index >= updated {
            assert_eq!(
                fs::metadata(path).unwrap().modified().unwrap(),
                modified[index]
            );
        }
    }
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 3);
}

#[rstest]
fn failed_error_report_stops_later_files(#[values("check", "write")] mode: &str) {
    let project = tempdir().unwrap();
    fs::create_dir(project.path().join(".git")).unwrap();
    let sources = [
        ("first.rs", fixture("cases/mod_use_groups", "expected.rs")),
        ("broken.rs", fixture("errors/invalid_utf8", "input.rs")),
        ("last.rs", fixture("cases/mod_use_groups", "input.rs")),
    ];
    let modified = sources.each_ref().map(|(name, bytes)| {
        let path = project.path().join(name);
        fs::write(&path, bytes).unwrap();
        fs::metadata(path).unwrap().modified().unwrap()
    });
    let output = execute(
        project.path(),
        &[mode, "first.rs", "broken.rs", "last.rs"],
        Stream::Stderr,
        Stdio::null(),
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(
        output.stdout.is_empty() && output.stderr.is_empty(),
        "{output:?}"
    );
    for (index, (name, bytes)) in sources.iter().enumerate() {
        let path = project.path().join(name);
        assert_eq!(fs::read(&path).unwrap(), *bytes);
        assert_eq!(
            fs::metadata(path).unwrap().modified().unwrap(),
            modified[index]
        );
    }
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 4);
}
