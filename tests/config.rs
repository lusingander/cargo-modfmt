pub mod common;

use std::{fs, path::Path, process::Output};

use rstest::rstest;
use tempfile::tempdir;

use crate::common::{command, fixture};

fn execute(cwd: &Path, args: &[&str], case: &str) -> Output {
    command(cwd)
        .args(args)
        .write_stdin(fixture(&format!("cases/{case}"), "input.rs"))
        .output()
        .unwrap()
}

#[rustfmt::skip]
#[rstest]
#[case::configured_old("2018", &["stdin"], 0)]
#[case::configured_new("2024", &["stdin"], 1)]
#[case::override_before("2024", &["--edition", "2021", "stdin"], 0)]
#[case::override_after("2024", &["stdin", "--edition", "2021"], 0)]
fn ancestor_config_and_cli_edition(
    #[case] edition: &str,
    #[case] args: &[&str],
    #[case] code: i32,
) {
    let project = tempdir().unwrap();
    fs::write(
        project.path().join("modfmt.toml"),
        format!(r#"[format]
edition = '{edition}'
"#),
    )
    .unwrap();
    let child = project.path().join("src/deep");
    fs::create_dir_all(&child).unwrap();
    let output = execute(&child, args, "edition_keyword");
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    if code == 0 {
        assert_eq!(output.stdout, fixture("cases/edition_keyword", "expected.rs"));
        assert!(output.stderr.is_empty());
    } else {
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("syntax error"));
    }
}

#[rstest]
#[case::empty("")]
#[case::files_only(
    r#"[files]
exclude = ['**']
"#
)]
fn nearest_config_does_not_merge(#[case] nearest: &str) {
    let project = tempdir().unwrap();
    fs::write(
        project.path().join("modfmt.toml"),
        r#"[format]
edition = '2018'
"#,
    )
    .unwrap();
    let child = project.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("modfmt.toml"), nearest).unwrap();
    let output = execute(&child, &["stdin"], "edition_keyword");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("syntax error"));
}

#[rstest]
#[case::directory(false)]
#[case::worktree_file(true)]
fn configuration_search_stops_at_git_root(#[case] worktree: bool) {
    let outer = tempdir().unwrap();
    fs::write(
        outer.path().join("modfmt.toml"),
        r#"[format]
edition = '2018'
"#,
    )
    .unwrap();
    let project = outer.path().join("project");
    fs::create_dir(&project).unwrap();
    if worktree {
        fs::write(project.join(".git"), "gitdir: ../git-data\n").unwrap();
    } else {
        fs::create_dir(project.join(".git")).unwrap();
    }
    let child = project.join("src");
    fs::create_dir(&child).unwrap();
    let output = execute(&child, &["stdin"], "edition_keyword");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("syntax error"));
}

#[rstest]
#[case::before(&["--config", "settings/custom.toml", "check", "source.rs"])]
#[case::after(&["check", "source.rs", "--config", "settings/custom.toml"])]
fn explicit_config_replaces_search_without_rebasing_inputs(#[case] args: &[&str]) {
    let project = tempdir().unwrap();
    fs::create_dir(project.path().join("settings")).unwrap();
    fs::write(project.path().join("modfmt.toml"), "invalid TOML").unwrap();
    fs::write(
        project.path().join("settings/custom.toml"),
        r#"[format]
edition = '2018'
"#,
    )
    .unwrap();
    let input = fixture("cases/edition_keyword", "input.rs");
    fs::write(project.path().join("source.rs"), &input).unwrap();
    let output = execute(project.path(), args, "edition_keyword");
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert_eq!(output.stdout, b"would reformat: \"source.rs\"\n");
    assert!(output.stderr.is_empty());
    assert_eq!(fs::read(project.path().join("source.rs")).unwrap(), input);
}

#[rstest]
#[case::syntax(b"not a TOML document")]
#[case::utf8(b"\xff")]
#[case::unknown_top(b"unknown = true")]
#[case::unknown_file_key(
    br#"[files]
excludes = []"#
)]
#[case::unknown_format_key(
    br#"[format]
max_width = 80"#
)]
#[case::wrong_type(
    br#"[files]
respect_gitignore = 'yes'"#
)]
#[case::wrong_array(
    br#"[files]
exclude = 'fixtures/**'"#
)]
#[case::wrong_edition_type(
    br#"[format]
edition = 2021"#
)]
#[case::invalid_edition(
    br#"[format]
edition = '2025'"#
)]
#[case::invalid_style(
    br#"[format]
style_edition = '2027'"#
)]
#[case::wrong_style_type(
    br#"[format]
style_edition = 2021"#
)]
#[case::invalid_glob(
    br#"[files]
exclude = ['[z-a]']"#
)]
#[case::negated_exclusion(
    br#"[files]
exclude = ['!keep.rs']"#
)]
fn invalid_config_is_rejected_before_writing(#[case] config: &[u8]) {
    let project = tempdir().unwrap();
    fs::write(project.path().join("modfmt.toml"), config).unwrap();
    let input = fixture("cases/mod_use_groups", "input.rs");
    let path = project.path().join("source.rs");
    fs::write(&path, &input).unwrap();
    let metadata = fs::metadata(&path).unwrap();
    let output = execute(
        project.path(),
        &[
            "write",
            "source.rs",
            "--edition",
            "2021",
            "--style-edition",
            "2024",
        ],
        "mod_use_groups",
    );
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("modfmt.toml"));
    assert_eq!(fs::read(&path).unwrap(), input);
    assert_eq!(
        fs::metadata(&path).unwrap().modified().unwrap(),
        metadata.modified().unwrap()
    );
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 2);
}

#[test]
fn missing_explicit_config_does_not_fall_back() {
    let project = tempdir().unwrap();
    fs::write(project.path().join("modfmt.toml"), "").unwrap();
    let output = execute(
        project.path(),
        &["stdin", "--config", "missing.toml"],
        "mod_basic",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing.toml"));
}

#[rstest]
#[case::help(&["--config", "missing.toml", "--help"], 0)]
#[case::subcommand_help(&["write", "--config", "missing.toml", "--help"], 0)]
#[case::config_help(&["config", "--config", "missing.toml", "--help"], 0)]
#[case::config_extra_path(&["--config", "missing.toml", "config", "first.rs", "second.rs"], 2)]
#[case::config_invalid_exclude(&["--config", "missing.toml", "config", "--exclude", "[z-a]"], 2)]
#[case::invalid_edition(&["--config", "missing.toml", "stdin", "--edition", "2025"], 2)]
#[case::missing_input(&["--config", "missing.toml", "check"], 2)]
#[case::invalid_exclude(&["--config", "missing.toml", "files", "missing.rs", "--exclude", "[z-a]"], 2)]
fn help_and_argument_errors_do_not_load_config(#[case] args: &[&str], #[case] code: i32) {
    let project = tempdir().unwrap();
    let output = execute(project.path(), args, "mod_basic");
    assert_eq!(output.status.code(), Some(code));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("read configuration"));
}

#[test]
fn stdin_does_not_apply_file_exclusions() {
    let project = tempdir().unwrap();
    fs::write(
        project.path().join("modfmt.toml"),
        r#"[files]
exclude = ['**']
"#,
    )
    .unwrap();
    let output = execute(project.path(), &["stdin"], "mod_basic");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/mod_basic", "expected.rs"));
    assert!(output.stderr.is_empty());
}

#[test]
fn multiple_projects_use_the_single_working_directory_configuration() {
    let project = tempdir().unwrap();
    fs::write(
        project.path().join("modfmt.toml"),
        r#"[format]
edition = '2018'
"#,
    )
    .unwrap();
    for (directory, config) in [
        (
            "first",
            r#"[format]
edition = '2024'
"#,
        ),
        ("second", "invalid TOML"),
    ] {
        let child = project.path().join(directory);
        fs::create_dir(&child).unwrap();
        fs::write(child.join("modfmt.toml"), config).unwrap();
        fs::write(
            child.join("source.rs"),
            fixture("cases/edition_keyword", "input.rs"),
        )
        .unwrap();
    }
    let output = execute(
        project.path(),
        &["check", "first/source.rs", "second/source.rs"],
        "mod_basic",
    );
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert_eq!(
        output.stdout,
        br#"would reformat: "first/source.rs"
would reformat: "second/source.rs"
"#
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn listing_validates_format_settings_before_selecting_paths() {
    let project = tempdir().unwrap();
    fs::write(
        project.path().join("modfmt.toml"),
        r#"[format]
edition = '2025'
"#,
    )
    .unwrap();
    let output = execute(project.path(), &["files", "missing.rs"], "mod_basic");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("format.edition"));
    assert!(!stderr.contains("missing.rs"));
}

#[cfg(unix)]
#[rstest]
#[case::directory(false)]
#[case::fifo(true)]
fn non_regular_configuration_is_rejected_without_opening_it(#[case] fifo: bool) {
    let project = tempdir().unwrap();
    let path = project.path().join("modfmt.toml");
    if fifo {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
    } else {
        fs::create_dir(&path).unwrap();
    }
    let output = execute(project.path(), &["stdin"], "mod_basic");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected a regular configuration file")
    );
}

#[rstest]
#[case::edition_fallback("edition = '2021'", &[], "legacy.rs")]
#[case::configured_style(r#"edition = '2024'
style_edition = '2021'"#, &[], "legacy.rs")]
#[case::style_wins_over_edition("style_edition = '2021'", &["--edition", "2024"], "legacy.rs")]
#[case::cli_style("style_edition = '2021'", &["--style-edition", "2024"], "expected.rs")]
fn configured_style_and_cli_overrides(
    #[case] format: &str,
    #[case] args: &[&str],
    #[case] expected: &str,
) {
    let project = tempdir().unwrap();
    fs::write(
        project.path().join("modfmt.toml"),
        format!(
            r#"[format]
{format}
"#
        ),
    )
    .unwrap();
    let mut command = vec!["stdin"];
    command.extend_from_slice(args);
    let output = execute(project.path(), &command, "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", expected));
}
