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

fn package(directory: &Path, edition: &str) {
    fs::create_dir_all(directory).unwrap();
    fs::write(
        directory.join("Cargo.toml"),
        format!(
            r#"[package]
name = 'example'
{edition}
"#
        ),
    )
    .unwrap();
}

#[rstest]
#[case::cargo_2015(Some("edition = '2015'"), "", &[], "legacy.rs")]
#[case::cargo_2018(Some("edition = '2018'"), "", &[], "legacy.rs")]
#[case::cargo_2021(Some("edition = '2021'"), "", &[], "legacy.rs")]
#[case::cargo_2024(Some("edition = '2024'"), "", &[], "expected.rs")]
#[case::cargo_over_rustfmt_old(Some("edition = '2021'"), "edition = '2024'", &[], "legacy.rs")]
#[case::cargo_over_rustfmt_new(Some("edition = '2024'"), "edition = '2021'", &[], "expected.rs")]
#[case::explicit_style_new(Some("edition = '2021'"), "style_edition = '2024'", &[], "expected.rs")]
#[case::explicit_style_old(Some("edition = '2024'"), "style_edition = '2021'", &[], "legacy.rs")]
#[case::rustfmt_edition(None, "edition = '2021'", &[], "legacy.rs")]
#[case::rustfmt_style(None, "style_edition = '2021'", &[], "legacy.rs")]
#[case::omitted_package_edition(Some(""), "edition = '2024'", &[], "legacy.rs")]
#[case::no_information(None, "", &[], "expected.rs")]
#[case::cli_edition(Some("edition = '2021'"), "", &["--edition", "2024"], "expected.rs")]
#[case::style_over_cli_edition(Some("edition = '2021'"), "style_edition = '2021'", &["--edition", "2024"], "legacy.rs")]
#[case::cli_style(Some("edition = '2024'"), "style_edition = '2021'", &["--style-edition", "2024"], "expected.rs")]
fn edition_sources_and_priority(
    #[case] cargo: Option<&str>,
    #[case] rustfmt: &str,
    #[case] args: &[&str],
    #[case] expected: &str,
) {
    let directory = tempdir().unwrap();
    fs::create_dir(directory.path().join(".git")).unwrap();
    if let Some(edition) = cargo {
        package(directory.path(), edition);
    }
    fs::write(directory.path().join("rustfmt.toml"), rustfmt).unwrap();
    let child = directory.path().join("src/deep");
    fs::create_dir_all(&child).unwrap();
    let mut command = vec!["stdin"];
    command.extend_from_slice(args);
    let output = execute(&child, &command, "style_order");
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", expected));
}

#[test]
fn explicit_modfmt_values_override_external_settings() {
    let directory = tempdir().unwrap();
    package(directory.path(), "edition = '2021'");
    fs::write(
        directory.path().join("rustfmt.toml"),
        "style_edition = '2021'",
    )
    .unwrap();
    fs::write(
        directory.path().join("modfmt.toml"),
        r#"[format]
edition = '2024'
style_edition = '2024'
"#,
    )
    .unwrap();
    let output = execute(directory.path(), &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", "expected.rs"));
}

#[test]
fn explicit_config_location_does_not_rebase_external_lookup() {
    let directory = tempdir().unwrap();
    package(directory.path(), "edition = '2024'");
    fs::write(directory.path().join("custom.toml"), "").unwrap();
    let other = tempdir().unwrap();
    package(other.path(), "edition = '2021'");
    let source = other.path().join("source.rs");
    fs::write(&source, fixture("cases/style_order", "input.rs")).unwrap();
    let output = execute(
        directory.path(),
        &["write", source.to_str().unwrap(), "--config", "custom.toml"],
        "style_order",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read(source).unwrap(),
        fixture("cases/style_order", "legacy.rs")
    );
}

#[test]
fn unrelated_external_settings_are_ignored() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("Cargo.toml"),
        r#"[package]
name = 'example'
edition = '2021'
[dependencies]
vendor = { version = '*', optional = true }
"#,
    )
    .unwrap();
    fs::write(
        directory.path().join("rustfmt.toml"),
        r#"comment_width = 80
imports_layout = 'Vertical'
unknown_setting = true
"#,
    )
    .unwrap();
    let output = execute(directory.path(), &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, fixture("cases/style_order", "legacy.rs"));
}

#[test]
fn automatic_parsing_edition_is_independent_of_explicit_style() {
    let directory = tempdir().unwrap();
    package(directory.path(), "edition = '2021'");
    let output = execute(
        directory.path(),
        &["stdin", "--style-edition", "2024"],
        "edition_keyword",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        fixture("cases/edition_keyword", "expected.rs")
    );
    let output = execute(
        directory.path(),
        &["stdin", "--edition", "2024", "--style-edition", "2021"],
        "edition_keyword",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("syntax error"));
}

#[test]
fn closest_rustfmt_file_is_not_merged_and_hidden_name_wins() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("rustfmt.toml"),
        "style_edition = '2021'",
    )
    .unwrap();
    let child = directory.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("rustfmt.toml"), "").unwrap();
    let output = execute(&child, &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", "expected.rs"));
    fs::write(child.join(".rustfmt.toml"), "style_edition = '2021'").unwrap();
    let output = execute(&child, &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", "legacy.rs"));
}

#[rstest]
#[case::git_directory(false)]
#[case::git_file(true)]
fn external_search_stops_at_repository_root(#[case] worktree: bool) {
    let directory = tempdir().unwrap();
    package(directory.path(), "edition = '2021'");
    fs::write(
        directory.path().join("rustfmt.toml"),
        "style_edition = '2021'",
    )
    .unwrap();
    let child = directory.path().join("project");
    fs::create_dir(&child).unwrap();
    if worktree {
        fs::write(child.join(".git"), "gitdir: ../git").unwrap();
    } else {
        fs::create_dir(child.join(".git")).unwrap();
    }
    let output = execute(&child, &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", "expected.rs"));
}

#[test]
fn each_input_uses_its_own_package_from_another_working_directory() {
    let directory = tempdir().unwrap();
    package(directory.path(), "edition = '2021'");
    let input = fixture("cases/style_order", "input.rs");
    let mut paths = Vec::new();
    for (name, edition) in [("old", "2021"), ("new", "2024")] {
        let child = directory.path().join(name);
        package(&child, &format!("edition = '{edition}'"));
        let path = child.join("source.rs");
        fs::write(&path, &input).unwrap();
        paths.push(path);
    }
    let unrelated = tempdir().unwrap();
    package(unrelated.path(), "edition = '2015'");
    let paths: Vec<_> = paths.iter().map(|path| path.to_str().unwrap()).collect();
    let output = execute(
        unrelated.path(),
        &["write", paths[0], paths[1]],
        "style_order",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read(paths[0]).unwrap(),
        fixture("cases/style_order", "legacy.rs")
    );
    assert_eq!(
        fs::read(paths[1]).unwrap(),
        fixture("cases/style_order", "expected.rs")
    );
    let output = execute(
        unrelated.path(),
        &["check", paths[0], paths[1]],
        "style_order",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

#[rstest]
#[case::rustfmt_syntax("rustfmt.toml", "invalid TOML")]
#[case::rustfmt_type("rustfmt.toml", "edition = 2021")]
#[case::rustfmt_value("rustfmt.toml", "style_edition = '2027'")]
#[case::cargo_syntax("Cargo.toml", "invalid TOML")]
#[case::cargo_value(
    "Cargo.toml",
    r#"[package]
edition = '2025'"#
)]
fn external_errors_are_checked_for_all_files_before_processing(
    #[case] name: &str,
    #[case] contents: &str,
    #[values("files", "check", "write")] mode: &str,
) {
    let directory = tempdir().unwrap();
    let input = fixture("cases/style_order", "input.rs");
    for child in ["first", "second"] {
        fs::create_dir(directory.path().join(child)).unwrap();
        fs::write(directory.path().join(child).join("source.rs"), &input).unwrap();
    }
    fs::write(directory.path().join("second").join(name), contents).unwrap();
    let output = execute(directory.path(), &[mode, "first", "second"], "style_order");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains(name));
    for child in ["first", "second"] {
        assert_eq!(
            fs::read(directory.path().join(child).join("source.rs")).unwrap(),
            input
        );
    }
}

#[test]
fn explicit_editions_do_not_read_cargo_settings() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("Cargo.toml"), "invalid TOML").unwrap();
    fs::write(directory.path().join(".rustfmt.toml"), "max_width = 80").unwrap();
    let output = execute(
        directory.path(),
        &["stdin", "--edition", "2024", "--style-edition", "2021"],
        "style_order",
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", "legacy.rs"));
}

#[rstest]
#[case::old("2021", "legacy.rs")]
#[case::new("2024", "expected.rs")]
fn workspace_edition_is_inherited_only_by_opted_in_packages(
    #[case] edition: &str,
    #[case] expected: &str,
) {
    let directory = tempdir().unwrap();
    fs::create_dir(directory.path().join(".git")).unwrap();
    fs::write(
        directory.path().join("Cargo.toml"),
        format!(
            r#"[workspace]
members = ['member', 'explicit']
[workspace.package]
edition = '{edition}'
"#
        ),
    )
    .unwrap();
    let member = directory.path().join("member");
    package(&member, "edition.workspace = true");
    let child = member.join("src/deep");
    fs::create_dir_all(&child).unwrap();
    let output = execute(&child, &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", expected));

    let explicit = directory.path().join("explicit");
    package(&explicit, "edition = '2024'");
    let output = execute(&explicit, &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", "expected.rs"));

    let output = execute(directory.path(), &["stdin"], "style_order");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fixture("cases/style_order", "expected.rs"));
}

#[rstest]
#[case::same_manifest(false)]
#[case::explicit_workspace(true)]
fn workspace_edition_can_be_defined_in_own_or_explicit_manifest(#[case] explicit: bool) {
    let directory = tempdir().unwrap();
    if explicit {
        let workspace = directory.path().join("workspace");
        fs::create_dir(&workspace).unwrap();
        fs::write(
            workspace.join("Cargo.toml"),
            r#"[workspace]
[workspace.package]
edition = '2021'
"#,
        )
        .unwrap();
        let member = directory.path().join("member");
        package(
            &member,
            r#"edition = { workspace = true }
workspace = '../workspace'
"#,
        );
        fs::create_dir(member.join(".git")).unwrap();
        let output = execute(&member, &["stdin"], "style_order");
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, fixture("cases/style_order", "legacy.rs"));
    } else {
        fs::write(
            directory.path().join("Cargo.toml"),
            r#"[package]
name = 'root'
edition.workspace = true
[workspace]
[workspace.package]
edition = '2021'
"#,
        )
        .unwrap();
        let output = execute(directory.path(), &["stdin"], "style_order");
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, fixture("cases/style_order", "legacy.rs"));
    }
}

#[rstest]
#[case::no_workspace("edition.workspace = true", "", "cannot find workspace")]
#[case::false_inheritance("edition.workspace = false", "", "must be true")]
#[case::missing_value(
    "edition.workspace = true",
    "[workspace]",
    "workspace.package.edition is missing"
)]
#[case::invalid_value(
    "edition.workspace = true",
    r#"[workspace.package]
edition = '2025'
"#,
    "workspace.package.edition"
)]
#[case::explicit_not_workspace(
    r#"edition.workspace = true
workspace = '..'
"#,
    "[package]",
    "expected a workspace manifest"
)]
#[case::explicit_missing(
    r#"edition.workspace = true
workspace = 'missing'
"#,
    "",
    "Cargo.toml"
)]
fn invalid_workspace_inheritance_does_not_fall_back(
    #[case] edition: &str,
    #[case] root: &str,
    #[case] diagnostic: &str,
) {
    let directory = tempdir().unwrap();
    fs::create_dir(directory.path().join(".git")).unwrap();
    fs::write(directory.path().join("Cargo.toml"), root).unwrap();
    let member = directory.path().join("member");
    package(&member, edition);
    fs::write(member.join("rustfmt.toml"), "edition = '2024'").unwrap();
    let output = execute(&member, &["stdin"], "style_order");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
        "{output:?}"
    );
}

#[test]
fn automatic_workspace_search_does_not_escape_repository_root() {
    let directory = tempdir().unwrap();
    fs::write(
        directory.path().join("Cargo.toml"),
        r#"[workspace.package]
edition = '2021'
"#,
    )
    .unwrap();
    let member = directory.path().join("member");
    package(&member, "edition.workspace = true");
    fs::create_dir(member.join(".git")).unwrap();
    let output = execute(&member, &["stdin"], "style_order");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot find workspace"));
}

#[cfg(unix)]
#[rstest]
#[case::directory(false)]
#[case::fifo(true)]
fn non_regular_external_settings_are_not_opened(
    #[case] fifo: bool,
    #[values("Cargo.toml", "rustfmt.toml")] name: &str,
) {
    let directory = tempdir().unwrap();
    let path = directory.path().join(name);
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
    let output = execute(directory.path(), &["stdin"], "style_order");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected a regular configuration file")
    );
}
