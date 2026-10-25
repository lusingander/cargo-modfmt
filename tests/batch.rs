pub mod common;

use std::{fs, path::Path, process::Output};

use rstest::rstest;
use tempfile::tempdir;

use crate::common::{command, fixture};

fn input() -> Vec<u8> {
    fixture("cases/mod_use_groups", "input.rs")
}
fn expected() -> Vec<u8> {
    fixture("cases/mod_use_groups", "expected.rs")
}

fn execute(cwd: &Path, args: &[&str]) -> Output {
    command(cwd)
        .args(args)
        .write_stdin(b"\xff".to_vec())
        .output()
        .unwrap()
}

#[test]
fn explicit_files_share_selection_order_exclusions_and_deduplication() {
    let project = tempdir().unwrap();
    for name in ["b.rs", "a.txt", "skip.rs", "also-skip.rs"] {
        fs::write(project.path().join(name), input()).unwrap();
    }
    fs::write(
        project.path().join("modfmt.toml"),
        r#"[files]
exclude = ['/skip.rs']
"#,
    )
    .unwrap();
    let absolute = project.path().join("b.rs");
    let arguments = [
        "./b.rs",
        "a.txt",
        absolute.to_str().unwrap(),
        "skip.rs",
        "also-skip.rs",
        "--exclude",
        "/also-skip.rs",
    ];
    let mut args = vec!["files"];
    args.extend(arguments);
    let output = execute(project.path(), &args);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        output.stdout,
        br#""./b.rs"
"a.txt"
"#
    );
    assert!(output.stderr.is_empty());
    args[0] = "check";
    let output = execute(project.path(), &args);
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert_eq!(
        output.stdout,
        br#"would reformat: "./b.rs"
would reformat: "a.txt"
"#
    );
    assert!(output.stderr.is_empty());
    args[0] = "write";
    let output = execute(project.path(), &args);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        br#"reformatted: "./b.rs"
reformatted: "a.txt"
"#
    );
    for name in ["b.rs", "a.txt"] {
        assert_eq!(fs::read(project.path().join(name)).unwrap(), expected());
    }
    for name in ["skip.rs", "also-skip.rs"] {
        assert_eq!(fs::read(project.path().join(name)).unwrap(), input());
    }
    let before = fs::metadata(&absolute).unwrap().modified().unwrap();
    let output = execute(project.path(), &args);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    assert_eq!(fs::metadata(&absolute).unwrap().modified().unwrap(), before);
}

#[rstest]
fn missing_explicit_path_prevents_all_processing(
    #[values("files", "check", "write")] command: &str,
) {
    let project = tempdir().unwrap();
    let path = project.path().join("valid.rs");
    fs::write(&path, input()).unwrap();
    let before = fs::metadata(&path).unwrap().modified().unwrap();
    let output = execute(
        project.path(),
        &[command, "valid.rs", "missing.rs", "--exclude", "missing.rs"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing.rs"));
    assert!(!stderr.contains("reformatted:"));
    assert_eq!(fs::read(&path).unwrap(), input());
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), before);
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 1);
}

#[rstest]
#[case::utf8("invalid_utf8")]
#[case::syntax("broken_item")]
fn individual_error_does_not_stop_other_files(
    #[case] error: &str,
    #[values("check", "write")] command: &str,
) {
    let project = tempdir().unwrap();
    for name in ["first.rs", "last.rs"] {
        fs::write(project.path().join(name), input()).unwrap();
    }
    fs::write(project.path().join("unchanged.rs"), expected()).unwrap();
    let broken = fixture(&format!("errors/{error}"), "input.rs");
    fs::write(project.path().join("broken.rs"), &broken).unwrap();
    let output = execute(
        project.path(),
        &[command, "first.rs", "broken.rs", "unchanged.rs", "last.rs"],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("broken.rs"));
    if command == "write" {
        assert!(output.stdout.is_empty());
        assert!(stderr.starts_with("reformatted: \"first.rs\"\n"));
        assert!(stderr.contains("reformatted: \"last.rs\"\n"));
        assert!(stderr.contains("2 files updated; 1 unchanged; 1 failed"));
    } else {
        assert_eq!(
            output.stdout,
            br#"would reformat: "first.rs"
would reformat: "last.rs"
"#
        );
    }
    for name in ["first.rs", "last.rs"] {
        assert_eq!(
            fs::read(project.path().join(name)).unwrap(),
            if command == "write" {
                expected()
            } else {
                input()
            }
        );
    }
    assert_eq!(fs::read(project.path().join("broken.rs")).unwrap(), broken);
    assert_eq!(
        fs::read(project.path().join("unchanged.rs")).unwrap(),
        expected()
    );
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 4);
}

#[test]
fn listing_does_not_parse_sources_or_select_input_configs() {
    let project = tempdir().unwrap();
    let child = project.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("modfmt.toml"), "invalid TOML").unwrap();
    fs::write(
        child.join("broken.rs"),
        fixture("errors/invalid_utf8", "input.rs"),
    )
    .unwrap();
    let output = execute(project.path(), &["files", "child/broken.rs"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"\"child/broken.rs\"\n");
    assert!(output.stderr.is_empty());
}

#[rstest]
fn all_excluded_is_silent_success(#[values("files", "check", "write")] command: &str) {
    let project = tempdir().unwrap();
    fs::write(
        project.path().join("broken.rs"),
        fixture("errors/invalid_utf8", "input.rs"),
    )
    .unwrap();
    let output = execute(project.path(), &[command, "broken.rs", "--exclude", "*.rs"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

#[cfg(unix)]
mod unix {
    use std::os::unix::fs::{MetadataExt, symlink};

    use super::*;

    #[test]
    fn parent_symlink_alias_is_deduplicated_but_hardlink_names_are_both_updated() {
        let project = tempdir().unwrap();
        let real = project.path().join("real");
        fs::create_dir(&real).unwrap();
        symlink("real", project.path().join("linked")).unwrap();
        fs::write(real.join("main.rs"), input()).unwrap();
        fs::hard_link(real.join("main.rs"), real.join("alias.rs")).unwrap();
        let output = execute(
            project.path(),
            &["write", "linked/main.rs", "real/main.rs", "real/alias.rs"],
        );
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(
            output.stderr,
            br#"reformatted: "linked/main.rs"
reformatted: "real/alias.rs"
"#
        );
        for name in ["main.rs", "alias.rs"] {
            assert_eq!(fs::read(real.join(name)).unwrap(), expected());
        }
        assert_ne!(
            fs::metadata(real.join("main.rs")).unwrap().ino(),
            fs::metadata(real.join("alias.rs")).unwrap().ino()
        );
    }
}
