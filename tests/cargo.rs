pub mod common;

use std::{fs, path::PathBuf, process::Output, time::Duration};

use assert_cmd::Command;
use rstest::rstest;
use tempfile::{TempDir, tempdir};

use crate::common::fixture;

struct Project {
    _directory: TempDir,
    cwd: PathBuf,
    cargo_home: PathBuf,
}

impl Project {
    fn new() -> Self {
        let directory = tempdir().unwrap();
        let cwd = directory.path().join("project");
        let cargo_home = directory.path().join("cargo-home");
        fs::create_dir_all(cwd.join(".git")).unwrap();
        let bin = cargo_home.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::copy(
            assert_cmd::cargo::cargo_bin!("cargo-modfmt"),
            bin.join(format!("cargo-modfmt{}", std::env::consts::EXE_SUFFIX)),
        )
        .unwrap();
        Self {
            _directory: directory,
            cwd,
            cargo_home,
        }
    }

    fn cargo(&self) -> Command {
        let mut command = Command::new(env!("CARGO"));
        command
            .current_dir(&self.cwd)
            .env("CARGO_HOME", &self.cargo_home)
            .env("PATH", self.cargo_home.join("bin"))
            .env("CARGO_TERM_COLOR", "never")
            .timeout(Duration::from_secs(5));
        command
    }

    fn execute(&self, args: &[&str], input: &[u8]) -> Output {
        self.cargo()
            .args(args)
            .write_stdin(input.to_vec())
            .output()
            .unwrap()
    }
}

fn assert_success(output: &Output) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn stdin_matches_expected_and_is_idempotent() {
    let project = Project::new();
    let input = fixture("cases/mod_use_groups", "input.rs");
    let expected = fixture("cases/mod_use_groups", "expected.rs");
    let output = project.execute(&["modfmt", "stdin"], &input);
    assert_success(&output);
    assert_eq!(output.stdout, expected);
    let repeated = project.execute(&["modfmt", "stdin"], &output.stdout);
    assert_success(&repeated);
    assert_eq!(repeated.stdout, output.stdout);
}

#[rustfmt::skip]
#[rstest]
#[case::automatic(&["modfmt", "stdin"], "legacy.rs")]
#[case::explicit_before(&["modfmt", "--style-edition", "2024", "stdin"], "expected.rs")]
#[case::explicit_after(&["modfmt", "stdin", "--style-edition", "2024"], "expected.rs")]
fn cargo_invocation_resolves_editions_and_accepts_style_override(#[case] args: &[&str], #[case] expected: &str) {
    let project = Project::new();
    fs::write(project.cwd.join("Cargo.toml"), r#"[package]
name = 'example'
edition = '2021'
"#).unwrap();
    let output = project.execute(args, &fixture("cases/style_order", "input.rs"));
    assert_success(&output);
    assert_eq!(output.stdout, fixture("cases/style_order", expected));
}

#[rstest]
#[case::command_name("modfmt")]
#[case::spaces("path with spaces.rs")]
#[case::leading_hyphen("-source.rs")]
fn file_workflow_preserves_paths(#[case] name: &str) {
    let project = Project::new();
    let path = project.cwd.join(name);
    let input = fixture("cases/mod_use_groups", "input.rs");
    let expected = fixture("cases/mod_use_groups", "expected.rs");
    fs::write(&path, &input).unwrap();

    let output = project.execute(&["modfmt", "files", "--", name], b"\xff");
    assert_success(&output);
    assert_eq!(output.stdout, format!("{name:?}\n").as_bytes());

    let output = project.execute(&["modfmt", "check", "--", name], b"\xff");
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert_eq!(
        output.stdout,
        format!("would reformat: {name:?}\n").as_bytes()
    );
    assert!(output.stderr.is_empty());
    assert_eq!(fs::read(&path).unwrap(), input);

    let output = project.execute(&["modfmt", "write", "--", name], b"\xff");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, format!("reformatted: {name:?}\n").as_bytes());
    assert_eq!(fs::read(&path).unwrap(), expected);

    let output = project.execute(&["modfmt", "check", "--", name], b"\xff");
    assert_success(&output);
    assert!(output.stdout.is_empty());
}

#[rustfmt::skip]
#[rstest]
#[case::root_help(&["modfmt", "--config", "missing.toml", "--help"], 0, "")]
#[case::subcommand_help(&["modfmt", "check", "--help"], 0, " check")]
#[case::help_command(&["modfmt", "help", "check"], 0, " check")]
#[case::cargo_help(&["help", "modfmt"], 0, "")]
#[case::missing_mode(&["modfmt"], 2, "")]
#[case::missing_path(&["modfmt", "check"], 2, " check")]
#[case::unknown_argument(&["modfmt", "stdin", "--unknown"], 2, " stdin")]
#[case::duplicate_prefix(&["modfmt", "modfmt", "check", "."], 2, "")]
fn help_and_argument_errors_precede_io(
    #[case] args: &[&str],
    #[case] code: i32,
    #[case] usage_suffix: &str,
) {
    let project = Project::new();
    fs::write(project.cwd.join("modfmt.toml"), "invalid TOML").unwrap();
    let output = project.execute(args, b"\xff");
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    let message = if code == 0 {
        assert!(output.stderr.is_empty(), "{output:?}");
        String::from_utf8(output.stdout).unwrap()
    } else {
        assert!(output.stdout.is_empty(), "{output:?}");
        String::from_utf8(output.stderr).unwrap()
    };
    assert!(
        message.contains(&format!("Usage: cargo modfmt{usage_suffix} ")),
        "{message}"
    );
    assert!(!message.contains("read configuration"), "{message}");
}

#[rstest]
#[case::utf8("invalid_utf8")]
#[case::syntax("broken_item")]
fn source_errors_keep_exit_code_one(#[case] case: &str) {
    let project = Project::new();
    let input = fixture(&format!("errors/{case}"), "input.rs");
    let output = project.execute(&["modfmt", "stdin"], &input);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[rustfmt::skip]
#[rstest]
#[case::before(&["modfmt", "--config", "modfmt", "--edition", "2021", "check", "source.rs"])]
#[case::after(&["modfmt", "check", "source.rs", "--config", "modfmt", "--edition", "2021"])]
fn explicit_configuration_and_cli_edition_keep_their_values(#[case] args: &[&str]) {
    let project = Project::new();
    fs::write(project.cwd.join("modfmt.toml"), "invalid TOML").unwrap();
    fs::write(
        project.cwd.join("modfmt"),
        r#"[format]
edition = '2024'
"#,
    )
    .unwrap();
    let input = fixture("cases/edition_keyword", "input.rs");
    fs::write(project.cwd.join("source.rs"), &input).unwrap();
    let output = project.execute(args, b"\xff");
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert_eq!(output.stdout, b"would reformat: \"source.rs\"\n");
    assert!(output.stderr.is_empty());
    assert_eq!(fs::read(project.cwd.join("source.rs")).unwrap(), input);
}

#[test]
fn configuration_search_starts_at_the_invocation_directory_without_a_manifest() {
    let mut project = Project::new();
    fs::write(
        project.cwd.join("modfmt.toml"),
        r#"[format]
edition = '2018'
"#,
    )
    .unwrap();
    project.cwd = project.cwd.join("nested");
    fs::create_dir(&project.cwd).unwrap();
    fs::write(
        project.cwd.join("source.rs"),
        fixture("cases/edition_keyword", "input.rs"),
    )
    .unwrap();
    let output = project.execute(&["modfmt", "write", "source.rs"], b"\xff");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"reformatted: \"source.rs\"\n");
    assert_eq!(
        fs::read(project.cwd.join("source.rs")).unwrap(),
        fixture("cases/edition_keyword", "expected.rs")
    );
}

#[cfg(unix)]
#[test]
fn non_utf8_path_reaches_file_processing() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let project = Project::new();
    let path = OsString::from_vec(b"source-\xff.rs".to_vec());
    let output = project
        .cargo()
        .args(["modfmt", "write", "--"])
        .arg(&path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains(&format!("{path:?}"))
    );
}
