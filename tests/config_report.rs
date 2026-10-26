pub mod common;

use std::{fs, path::Path};

use rstest::rstest;
use tempfile::{TempDir, tempdir};

use crate::common::{command, fixture};

fn new_project() -> TempDir {
    let project = tempdir().unwrap();
    fs::create_dir(project.path().join(".git")).unwrap();
    project
}

fn report(cwd: &Path, args: &[&str]) -> String {
    let output = command(cwd).arg("config").args(args).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

fn assert_setting(report: &str, name: &str, value: &str, origin: &str) {
    let expected = format!("{name}: {value}\n  {origin}\n");
    assert!(
        report.contains(&expected),
        "missing {expected:?} in:\n{report}"
    );
}

#[rstest]
#[case::cwd(&[])]
#[case::directory(&["."])]
#[case::file(&["invalid.rs"])]
fn defaults_need_neither_source_contents_nor_stdin(#[case] args: &[&str]) {
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    fs::write(root.join("invalid.rs"), [0xff]).unwrap();
    let output = report(&root, args);
    let target = if args == ["invalid.rs"] {
        root.join("invalid.rs")
    } else {
        root.clone()
    };
    assert!(
        output.contains(&format!("target: {target:?}\n")),
        "{output}"
    );
    assert!(
        output.contains(&format!("format lookup directory: {root:?}\n")),
        "{output}"
    );
    for (name, value) in [
        ("edition", "2024"),
        ("max_width", "100"),
        ("tab_spaces", "4"),
        ("hard_tabs", "false"),
        ("files.respect_gitignore", "true"),
        ("files.exclude", "[]"),
    ] {
        assert_setting(&output, name, value, "built-in default");
    }
    assert_setting(
        &output,
        "style_edition",
        "2024",
        "fallback to resolved edition",
    );
    for name in ["modfmt config", "Cargo manifest", "rustfmt config"] {
        assert!(output.contains(&format!("{name}: none\n")), "{output}");
    }
    assert!(
        output.contains(&format!("stopped at {root:?} (Git repository root)")),
        "{output}"
    );
    assert!(output.contains("file selection: not evaluated\n"));
    assert_eq!(fs::read(root.join("invalid.rs")).unwrap(), [0xff]);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
}

#[rstest]
#[case::cargo(Some("edition = '2021'"), "edition = '2018'\nstyle_edition = '2015'", "", &[], ("2021", "Cargo.toml", "package.edition"), ("2015", "rustfmt.toml", "style_edition"))]
#[case::rustfmt(None, "edition = '2021'", "", &[], ("2021", "rustfmt.toml", "edition"), ("2021", "", ""))]
#[case::modfmt(Some("edition = '2024'"), "style_edition = '2021'", "[format]\nedition = '2018'", &[], ("2018", "modfmt.toml", "format.edition"), ("2021", "rustfmt.toml", "style_edition"))]
#[case::cli_edition(Some("edition = '2015'"), "edition = '2018'", "[format]\nedition = '2018'\nstyle_edition = '2024'", &["--edition", "2021"], ("2021", "", "--edition"), ("2024", "modfmt.toml", "format.style_edition"))]
#[case::cli_style(Some("edition = '2024'"), "style_edition = '2021'", "[format]\nstyle_edition = '2015'", &["--style-edition", "2018"], ("2024", "Cargo.toml", "package.edition"), ("2018", "", "--style-edition"))]
fn edition_origins_follow_the_same_priority_as_formatting(
    #[case] cargo: Option<&str>,
    #[case] rustfmt: &str,
    #[case] modfmt: &str,
    #[case] args: &[&str],
    #[case] expected_edition: (&str, &str, &str),
    #[case] expected_style: (&str, &str, &str),
) {
    let (edition, edition_file, edition_key) = expected_edition;
    let (style, style_file, style_key) = expected_style;
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    if let Some(cargo) = cargo {
        fs::write(
            root.join("Cargo.toml"),
            format!("[package]\nname = 'example'\n{cargo}"),
        )
        .unwrap();
    }
    fs::write(root.join("rustfmt.toml"), rustfmt).unwrap();
    fs::write(root.join("modfmt.toml"), modfmt).unwrap();
    let origin = |file: &str, key: &str| {
        if !file.is_empty() {
            format!("from {:?}: {key}", root.join(file))
        } else if key.is_empty() {
            "fallback to resolved edition".into()
        } else {
            format!("command line {key}")
        }
    };
    let output = report(&root, args);
    assert_setting(
        &output,
        "edition",
        edition,
        &origin(edition_file, edition_key),
    );
    assert_setting(
        &output,
        "style_edition",
        style,
        &origin(style_file, style_key),
    );
    if edition_file == "modfmt.toml" || edition_key == "--edition" {
        assert!(
            output.contains("Cargo manifest: not searched (explicit edition)"),
            "{output}"
        );
    }
    let formatted = command(&root)
        .arg("stdin")
        .args(args)
        .write_stdin(fixture("cases/style_order", "input.rs"))
        .output()
        .unwrap();
    assert!(formatted.status.success(), "{formatted:?}");
    let expected = if style == "2024" {
        "expected.rs"
    } else {
        "legacy.rs"
    };
    assert_eq!(formatted.stdout, fixture("cases/style_order", expected));
}

#[rstest]
#[case::omitted(false)]
#[case::explicit(true)]
fn explicit_default_values_are_distinguished_from_omitted_keys(#[case] explicit: bool) {
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    fs::write(
        root.join("rustfmt.toml"),
        if explicit {
            "max_width = 100\ntab_spaces = 4\nhard_tabs = false"
        } else {
            ""
        },
    )
    .unwrap();
    fs::write(
        root.join("modfmt.toml"),
        if explicit {
            "[files]\nrespect_gitignore = true\nexclude = []"
        } else {
            ""
        },
    )
    .unwrap();
    let output = report(&root, &[]);
    for (key, value, file, source_key) in [
        ("max_width", "100", "rustfmt.toml", "max_width"),
        ("tab_spaces", "4", "rustfmt.toml", "tab_spaces"),
        ("hard_tabs", "false", "rustfmt.toml", "hard_tabs"),
        (
            "files.respect_gitignore",
            "true",
            "modfmt.toml",
            "files.respect_gitignore",
        ),
        ("files.exclude", "[]", "modfmt.toml", "files.exclude"),
    ] {
        let source = if explicit {
            format!("from {:?}: {source_key}", root.join(file))
        } else {
            format!("built-in default; omitted in {:?}", root.join(file))
        };
        assert_setting(&output, key, value, &source);
    }
    assert_setting(
        &output,
        "edition",
        "2024",
        &format!(
            "built-in default; omitted in {:?}",
            root.join("rustfmt.toml")
        ),
    );
}

#[rstest]
#[case::omitted("[package]\nname = 'example'", "2015", true)]
#[case::virtual_workspace("[workspace.package]\nedition = '2015'", "2024", false)]
fn cargo_defaults_and_virtual_workspaces_have_distinct_origins(
    #[case] cargo: &str,
    #[case] edition: &str,
    #[case] omitted: bool,
) {
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    fs::write(root.join("Cargo.toml"), cargo).unwrap();
    let output = report(&root, &[]);
    let origin = if omitted {
        format!(
            "Cargo default; package.edition omitted in {:?}",
            root.join("Cargo.toml")
        )
    } else {
        "built-in default".into()
    };
    assert_setting(&output, "edition", edition, &origin);
}

#[rstest]
#[case::ancestor(false, false)]
#[case::same_manifest(false, true)]
#[case::explicit_outside_search_boundary(true, false)]
fn workspace_inheritance_shows_both_manifests(#[case] explicit: bool, #[case] same: bool) {
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    let member = if same {
        root.clone()
    } else {
        root.join("member")
    };
    fs::create_dir_all(&member).unwrap();
    let workspace = if explicit {
        root.join("workspace")
    } else {
        root.clone()
    };
    fs::create_dir_all(&workspace).unwrap();
    let mut cargo = "[package]\nname = 'member'\nedition.workspace = true\n".to_owned();
    if explicit {
        cargo.push_str("workspace = '../workspace'\n");
        fs::create_dir(member.join(".git")).unwrap();
    }
    let workspace_settings = "[workspace.package]\nedition = '2021'\n";
    if same {
        cargo.push_str(workspace_settings);
    } else {
        fs::write(workspace.join("Cargo.toml"), workspace_settings).unwrap();
    }
    fs::write(member.join("Cargo.toml"), cargo).unwrap();
    let output = report(&member, &[]);
    let source = if explicit {
        member.join("../workspace/Cargo.toml")
    } else {
        workspace.join("Cargo.toml")
    };
    assert_setting(
        &output,
        "edition",
        "2021",
        &format!("from {source:?}: workspace.package.edition"),
    );
    assert!(
        output.contains(&format!(
            "via {:?}: package.edition.workspace = true",
            member.join("Cargo.toml")
        )),
        "{output}"
    );
    assert_setting(
        &output,
        "style_edition",
        "2021",
        "fallback to resolved edition",
    );
}

#[test]
fn explicit_config_does_not_rebase_per_target_lookup_or_filter_the_target() {
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    let other = new_project();
    let target_root = other.path().canonicalize().unwrap();
    fs::create_dir(root.join("settings")).unwrap();
    let config = root.join("settings/custom.toml");
    fs::write(&config, "[files]\nrespect_gitignore = false\nexclude = ['**/*.rs']\n[format]\nstyle_edition = '2024'").unwrap();
    fs::write(
        target_root.join("Cargo.toml"),
        "[package]\nname = 'target'\nedition = '2018'",
    )
    .unwrap();
    fs::write(
        target_root.join("modfmt.toml"),
        "[format]\nedition = '2015'",
    )
    .unwrap();
    fs::write(
        target_root.join("rustfmt.toml"),
        "max_width = 80\ntab_spaces = 2\nhard_tabs = true",
    )
    .unwrap();
    let target = target_root.join("hidden.rs");
    fs::write(&target, [0xff]).unwrap();
    let output = report(
        &root,
        &[
            target.to_str().unwrap(),
            "--config",
            "settings/custom.toml",
            "--exclude",
            "vendor/**",
        ],
    );
    assert!(
        output.contains(&format!("modfmt config: {config:?} (explicit)")),
        "{output}"
    );
    assert_setting(
        &output,
        "edition",
        "2018",
        &format!("from {:?}: package.edition", target_root.join("Cargo.toml")),
    );
    assert_setting(
        &output,
        "style_edition",
        "2024",
        &format!("from {config:?}: format.style_edition"),
    );
    assert_setting(
        &output,
        "max_width",
        "80",
        &format!("from {:?}: max_width", target_root.join("rustfmt.toml")),
    );
    assert_setting(
        &output,
        "files.respect_gitignore",
        "false",
        &format!("from {config:?}: files.respect_gitignore"),
    );
    assert!(
        output.contains("files.exclude: [\"**/*.rs\", \"vendor/**\"]"),
        "{output}"
    );
    assert!(
        output.contains(&format!("\"**/*.rs\": from {config:?}: files.exclude")),
        "{output}"
    );
    assert!(
        output.contains("\"vendor/**\": command line --exclude"),
        "{output}"
    );
    assert!(
        output.contains(&format!(
            "exclusion base directory: {:?}",
            root.join("settings")
        )),
        "{output}"
    );
    assert_eq!(fs::read(&target).unwrap(), [0xff]);
    let automatic = report(&root, &[target.to_str().unwrap()]);
    assert!(automatic.contains("modfmt config: none"), "{automatic}");
    assert_setting(
        &automatic,
        "style_edition",
        "2018",
        "fallback to resolved edition",
    );
}

#[test]
fn reported_layout_values_are_used_by_formatting() {
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    let config = root.join("rustfmt.toml");
    fs::write(&config, "max_width = 80\ntab_spaces = 2\nhard_tabs = true").unwrap();
    let output = report(&root, &[]);
    for (key, value) in [
        ("max_width", "80"),
        ("tab_spaces", "2"),
        ("hard_tabs", "true"),
    ] {
        assert_setting(&output, key, value, &format!("from {config:?}: {key}"));
    }
    let formatted = command(&root).arg("stdin").write_stdin("use vendor::{AlphaLongName, BetaLongName, DeltaLongName, EpsilonLongName, GammaLongName};\n").output().unwrap();
    assert!(formatted.status.success(), "{formatted:?}");
    assert_eq!(formatted.stdout, b"use vendor::{\n\tAlphaLongName, BetaLongName, DeltaLongName, EpsilonLongName, GammaLongName,\n};\n");
}

#[test]
fn nearest_hidden_rustfmt_config_is_not_merged_with_other_configs() {
    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    fs::write(root.join("rustfmt.toml"), "max_width = 80").unwrap();
    let child = root.join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("rustfmt.toml"), "max_width = 120").unwrap();
    let config = child.join(".rustfmt.toml");
    fs::write(&config, "tab_spaces = 2").unwrap();
    let output = report(&root, &["child"]);
    assert!(
        output.contains(&format!("rustfmt config: {config:?} (automatic)")),
        "{output}"
    );
    assert_setting(
        &output,
        "max_width",
        "100",
        &format!("built-in default; omitted in {config:?}"),
    );
    assert_setting(
        &output,
        "tab_spaces",
        "2",
        &format!("from {config:?}: tab_spaces"),
    );
}

#[rstest]
#[case::cli(&["--edition", "2021", "--style-edition", "2024"], "")]
#[case::modfmt(&[], "[format]\nedition = '2021'\nstyle_edition = '2024'")]
fn inspection_does_not_read_settings_unneeded_by_normal_resolution(
    #[case] args: &[&str],
    #[case] modfmt: &str,
) {
    let project = new_project();
    fs::write(project.path().join("Cargo.toml"), "invalid TOML").unwrap();
    fs::write(
        project.path().join("rustfmt.toml"),
        "edition = 'invalid'\nstyle_edition = 'invalid'\nmax_width = 80",
    )
    .unwrap();
    fs::write(project.path().join("modfmt.toml"), modfmt).unwrap();
    let output = report(project.path(), args);
    assert!(
        output.contains("Cargo manifest: not searched (explicit edition)"),
        "{output}"
    );
    assert!(output.contains("edition: 2021\n"));
    assert!(output.contains("style_edition: 2024\n"));
    assert!(output.contains("max_width: 80\n"));
    let normal = command(project.path())
        .arg("stdin")
        .args(args)
        .write_stdin("")
        .output()
        .unwrap();
    assert!(normal.status.success(), "{normal:?}");
}

#[rstest]
#[case::modfmt("modfmt.toml", "invalid TOML")]
#[case::modfmt_unknown("modfmt.toml", "unknown = true")]
#[case::cargo("Cargo.toml", "invalid TOML")]
#[case::workspace("Cargo.toml", "[package]\nname = 'example'\nedition.workspace = true")]
#[case::rustfmt("rustfmt.toml", "invalid TOML")]
#[case::layout("rustfmt.toml", "hard_tabs = true\ntab_spaces = 0")]
#[case::layout_type("rustfmt.toml", "max_width = '80'")]
#[case::edition("rustfmt.toml", "edition = 'invalid'")]
fn inspection_and_formatting_report_identical_resolution_errors(
    #[case] name: &str,
    #[case] contents: &str,
) {
    let project = new_project();
    fs::write(project.path().join(name), contents).unwrap();
    let inspected = command(project.path()).arg("config").output().unwrap();
    let formatted = command(project.path())
        .arg("stdin")
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(inspected.status.code(), Some(1), "{inspected:?}");
    assert_eq!(formatted.status.code(), Some(1), "{formatted:?}");
    assert!(inspected.stdout.is_empty());
    assert_eq!(inspected.stderr, formatted.stderr);
}

#[test]
fn directory_lookup_does_not_recurse_and_stops_at_the_repository_root() {
    let parent = new_project();
    fs::write(parent.path().join("rustfmt.toml"), "max_width = 80").unwrap();
    let root = parent.path().join("nested");
    fs::create_dir(&root).unwrap();
    fs::write(root.join(".git"), "gitdir: ../.git").unwrap();
    let child = root.join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("rustfmt.toml"), "invalid TOML").unwrap();
    let output = report(&root, &[]);
    assert_setting(&output, "max_width", "100", "built-in default");
    assert!(
        output.contains(&format!(
            "stopped at {:?} (Git repository root)",
            root.canonicalize().unwrap()
        )),
        "{output}"
    );
    let inspected = command(&root).args(["config", "child"]).output().unwrap();
    assert_eq!(inspected.status.code(), Some(1));
    assert!(inspected.stdout.is_empty());
}

#[test]
fn paths_with_quotes_and_newlines_are_escaped() {
    let project = new_project();
    let child = project.path().join("quoted\"\npath");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("rustfmt.toml"), "max_width = 80").unwrap();
    let canonical = child.canonicalize().unwrap();
    let output = report(project.path(), &[child.to_str().unwrap()]);
    assert!(
        output.contains(&format!("target: {canonical:?}\n")),
        "{output}"
    );
    assert_setting(
        &output,
        "max_width",
        "80",
        &format!("from {:?}: max_width", canonical.join("rustfmt.toml")),
    );
    assert!(!output.contains("quoted\"\npath"));
}

#[test]
fn missing_target_is_an_error_without_a_partial_report() {
    let project = new_project();
    let output = command(project.path())
        .args(["config", "missing.rs"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("inspect configuration target")
    );
}

#[cfg(unix)]
#[rstest]
#[case::symlink(false)]
#[case::fifo(true)]
fn special_targets_are_rejected_without_opening_them(#[case] fifo: bool) {
    let project = new_project();
    let path = project.path().join("special.rs");
    if fifo {
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
    } else {
        fs::write(project.path().join("source.rs"), "").unwrap();
        std::os::unix::fs::symlink("source.rs", &path).unwrap();
    }
    let output = command(project.path())
        .args(["config", "special.rs"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("expected a regular file or directory")
    );
}

#[cfg(unix)]
#[test]
fn unreadable_source_and_symlinked_parent_do_not_require_opening_source() {
    use std::os::unix::fs::PermissionsExt;

    let project = new_project();
    let root = project.path().canonicalize().unwrap();
    let child = root.join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("rustfmt.toml"), "max_width = 80").unwrap();
    let source = child.join("source.rs");
    fs::write(&source, "invalid Rust").unwrap();
    let permissions = fs::metadata(&source).unwrap().permissions();
    fs::set_permissions(&source, fs::Permissions::from_mode(0o000)).unwrap();
    std::os::unix::fs::symlink(&child, root.join("link")).unwrap();
    let inspected = command(&root)
        .args(["config", "link/source.rs"])
        .output()
        .unwrap();
    fs::set_permissions(&source, permissions).unwrap();
    assert!(inspected.status.success(), "{inspected:?}");
    let output = String::from_utf8(inspected.stdout).unwrap();
    assert!(
        output.contains(&format!("target: {source:?}\n")),
        "{output}"
    );
    assert!(
        output.contains(&format!("format lookup directory: {child:?}\n")),
        "{output}"
    );
    assert_eq!(fs::read_to_string(&source).unwrap(), "invalid Rust");
}
