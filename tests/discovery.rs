pub mod common;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use rstest::rstest;
use tempfile::{Builder, TempDir};

use crate::common::{command, fixture};

fn project() -> TempDir {
    Builder::new()
        .prefix("modfmt-discovery-")
        .tempdir()
        .unwrap()
}

fn put(root: &Path, name: &str, bytes: impl AsRef<[u8]>) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

fn source(root: &Path, name: &str) {
    put(root, name, fixture("cases/mod_use_groups", "input.rs"));
}

fn execute(cwd: &Path, args: &[&str]) -> Output {
    command(cwd)
        .args(args)
        .write_stdin(b"\xff".to_vec())
        .output()
        .unwrap()
}

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, directory: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, output);
            } else {
                output.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut output = BTreeMap::new();
    collect(root, root, &mut output);
    output
}

#[test]
fn mixed_inputs_match_documented_selection_and_preserve_every_other_file() {
    let project = project();
    let root = project.path();
    fs::create_dir(root.join(".git")).unwrap();
    put(
        root,
        "modfmt.toml",
        r#"[files]
exclude = ['fixtures/**', 'vendor/**']
"#,
    );
    put(root, ".gitignore", "/generated/\n");
    put(
        root,
        "src/keep.rs",
        fixture("cases/mod_use_groups", "expected.rs"),
    );
    for name in ["src/main.rs", "tests/smoke.rs", "generated/manual.rs"] {
        source(root, name);
    }
    for name in [
        "fixtures/broken/input.rs",
        "target/build/generated.rs",
        ".hidden/file.rs",
        "src/.hidden.rs",
        "src/uppercase.RS",
        "README.md",
    ] {
        put(root, name, fixture("errors/invalid_utf8", "input.rs"));
    }
    put(root, "src/modfmt.toml", "invalid child configuration");
    let before = files(root);
    let mut args = vec![
        "files",
        "src",
        ".",
        "generated/manual.rs",
        "fixtures/broken/input.rs",
        "--exclude",
        "/tests/",
    ];
    let output = execute(root, &args);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        output.stdout,
        br#""src/keep.rs"
"src/main.rs"
"generated/manual.rs"
"#
    );
    assert!(output.stderr.is_empty());
    args[0] = "check";
    let output = execute(root, &args);
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert_eq!(
        output.stdout,
        br#"would reformat: "src/main.rs"
would reformat: "generated/manual.rs"
"#
    );
    assert!(output.stderr.is_empty());
    assert_eq!(files(root), before);
    args[0] = "write";
    let output = execute(root, &args);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        br#"reformatted: "src/main.rs"
reformatted: "generated/manual.rs"
"#
    );
    let mut after = before;
    for name in ["src/main.rs", "generated/manual.rs"] {
        after.insert(name.into(), fixture("cases/mod_use_groups", "expected.rs"));
    }
    assert_eq!(files(root), after);
    let stamp = fs::metadata(root.join("src/main.rs"))
        .unwrap()
        .modified()
        .unwrap();
    for command in ["check", "write"] {
        args[0] = command;
        let output = execute(root, &args);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
        assert_eq!(files(root), after);
        assert_eq!(
            fs::metadata(root.join("src/main.rs"))
                .unwrap()
                .modified()
                .unwrap(),
            stamp
        );
    }
}

#[rstest]
#[case::enabled(true, br#""src/keep.rs"
"src/nested/child.rs"
"#.as_slice())]
#[case::disabled(false, br#""src/keep.rs"
"src/nested/child.rs"
"src/nested/other.rs"
"src/skip.rs"
"#.as_slice())]
fn nested_gitignore_and_disable_switch(#[case] enabled: bool, #[case] expected: &[u8]) {
    let project = project();
    let root = project.path();
    fs::create_dir(root.join(".git")).unwrap();
    put(
        root,
        "modfmt.toml",
        format!(
            r#"[files]
respect_gitignore = {enabled}
exclude = ['src/config-skip.rs']
"#
        ),
    );
    put(
        root,
        ".gitignore",
        r#"*.rs
!keep.rs
!.hidden.rs
!target/inside.rs
!config-skip.rs
"#,
    );
    put(root, "src/nested/.gitignore", "!child.rs\n");
    for name in [
        "src/keep.rs",
        "src/skip.rs",
        "src/nested/child.rs",
        "src/nested/other.rs",
        "src/config-skip.rs",
        "src/.hidden.rs",
        "src/target/inside.rs",
    ] {
        source(root, name);
    }
    let output = execute(root, &["files", "src"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn ignored_ancestor_cannot_be_reincluded_or_read_from_an_explicit_directory() {
    let project = project();
    let root = project.path();
    fs::create_dir(root.join(".git")).unwrap();
    put(root, ".gitignore", "/blocked/\n");
    put(root, "blocked/.gitignore", "!child/\n");
    put(root, "blocked/child/.gitignore", "[z-a]\n");
    source(root, "blocked/child/main.rs");
    let output = execute(root, &["files", "blocked/child"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    let output = execute(root, &["files", "blocked/child/main.rs"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"\"blocked/child/main.rs\"\n");
}

#[rstest]
#[case::repository(true, b"\"src/keep.rs\"\n".as_slice())]
#[case::outside_repository(false, br#""src/keep.rs"
"src/skip.rs"
"#.as_slice())]
fn parent_gitignore_is_bounded_by_the_repository(
    #[case] repository: bool,
    #[case] expected: &[u8],
) {
    let project = project();
    let root = project.path();
    if repository {
        fs::create_dir(root.join(".git")).unwrap();
    }
    put(root, ".gitignore", "/src/skip.rs\n");
    source(root, "src/keep.rs");
    source(root, "src/skip.rs");
    let output = execute(root, &["files", "src"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, expected);
}

#[rstest]
#[case::directory(false)]
#[case::worktree(true)]
fn repository_boundary_and_other_tools_ignore_files_do_not_change_selection(
    #[case] worktree: bool,
) {
    let outer = project();
    put(outer.path(), ".gitignore", "[z-a]\n");
    let root = outer.path().join("project");
    if worktree {
        put(&root, ".git", "gitdir: git-data\n");
        put(&root, "git-data/info/exclude", "*.rs\n");
    } else {
        put(&root, ".git/info/exclude", "*.rs\n");
    }
    put(&root, ".ignore", "*.rs\n");
    put(&root, "modfmt.toml", "");
    source(&root, "src/main.rs");
    let output = execute(&root, &["files", "src"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"\"src/main.rs\"\n");
    assert!(output.stderr.is_empty());
}

#[rstest]
#[case::glob(b"[z-a]\n")]
#[case::utf8(b"\xff\n")]
fn invalid_gitignore_prevents_all_updates(#[case] ignore: &[u8]) {
    let project = project();
    let root = project.path();
    source(root, "first.rs");
    source(root, "src/last.rs");
    put(root, "src/.gitignore", ignore);
    let before = files(root);
    let output = execute(root, &["write", "first.rs", "src"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains(".gitignore"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("reformatted:"));
    assert_eq!(files(root), before);
    put(
        root,
        "modfmt.toml",
        r#"[files]
respect_gitignore = false
"#,
    );
    let output = execute(root, &["files", "src"]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"\"src/last.rs\"\n");
}

#[test]
fn configuration_and_cli_exclusions_use_the_configuration_directory() {
    let project = project();
    let root = project.path();
    put(
        root,
        "modfmt.toml",
        r#"[files]
exclude = ['/fixtures/']
"#,
    );
    source(root, "fixtures/source.rs");
    source(root, "src/generated/file.rs");
    source(root, "keep.rs");
    let cwd = root.join("src");
    let absolute = root.join("keep.rs");
    let output = execute(
        &cwd,
        &[
            "files",
            "../fixtures/source.rs",
            "generated/file.rs",
            absolute.to_str().unwrap(),
            "--exclude",
            "/src/generated/",
        ],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, format!("{absolute:?}\n").as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn explicit_configuration_can_exclude_targets_outside_its_directory() {
    let project = project();
    let root = project.path();
    put(
        root,
        "settings/custom.toml",
        r#"[files]
exclude = ['/../work/skip.rs']
"#,
    );
    source(root, "work/skip.rs");
    source(root, "work/keep.rs");
    let output = execute(
        &root.join("work"),
        &["files", ".", "--config", "../settings/custom.toml"],
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"\"./keep.rs\"\n");
}

#[rstest]
fn empty_directory_is_success(#[values("files", "check", "write")] command: &str) {
    let project = project();
    let output = execute(project.path(), &[command, "."]);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

#[cfg(unix)]
mod unix {
    use std::{
        os::unix::fs::{PermissionsExt, symlink},
        process::Command,
    };

    use super::*;

    #[rstest]
    #[case::directory(false)]
    #[case::fifo(true)]
    fn non_regular_ignore_file_is_rejected_without_opening_it(#[case] fifo: bool) {
        let project = project();
        let root = project.path();
        source(root, "main.rs");
        let path = root.join(".gitignore");
        if fifo {
            assert!(
                Command::new("mkfifo")
                    .arg(&path)
                    .status()
                    .unwrap()
                    .success()
            );
        } else {
            fs::create_dir(&path).unwrap();
        }
        let output = execute(root, &["write", "."]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("expected a regular ignore file"));
        assert_eq!(
            fs::read(root.join("main.rs")).unwrap(),
            fixture("cases/mod_use_groups", "input.rs")
        );
    }

    #[test]
    fn discovery_does_not_follow_links_or_open_special_files() {
        let project = project();
        let root = project.path();
        source(root, "real/main.rs");
        symlink("real", root.join("linked")).unwrap();
        symlink(".", root.join("real/cycle")).unwrap();
        symlink("main.rs", root.join("real/alias.rs")).unwrap();
        symlink("absent.rs", root.join("broken.rs")).unwrap();
        assert!(
            Command::new("mkfifo")
                .arg(root.join("pipe.rs"))
                .status()
                .unwrap()
                .success()
        );
        let output = execute(root, &["files", "."]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stdout, b"\"./real/main.rs\"\n");
        let output = execute(root, &["write", "real/main.rs", "linked"]);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(
            fs::read(root.join("real/main.rs")).unwrap(),
            fixture("cases/mod_use_groups", "input.rs")
        );
    }

    #[rstest]
    #[case::unreadable(false)]
    #[case::excluded(true)]
    fn unreadable_directory_errors_before_updates_unless_excluded(#[case] excluded: bool) {
        let project = project();
        let root = project.path();
        source(root, "main.rs");
        source(root, "denied/source.rs");
        if excluded {
            put(
                root,
                "modfmt.toml",
                r#"[files]
exclude = ['/denied/']
"#,
            );
        }
        let denied = root.join("denied");
        let permissions = fs::metadata(&denied).unwrap().permissions();
        fs::set_permissions(&denied, fs::Permissions::from_mode(0o000)).unwrap();
        let output = execute(root, &["write", "main.rs", "denied"]);
        fs::set_permissions(&denied, permissions).unwrap();
        assert_eq!(
            output.status.code(),
            Some(if excluded { 0 } else { 1 }),
            "{output:?}"
        );
        assert!(output.stdout.is_empty());
        assert_eq!(
            fs::read(root.join("main.rs")).unwrap(),
            fixture(
                "cases/mod_use_groups",
                if excluded { "expected.rs" } else { "input.rs" }
            )
        );
        assert_eq!(
            fs::read(root.join("denied/source.rs")).unwrap(),
            fixture("cases/mod_use_groups", "input.rs")
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn discovered_non_utf8_filename_is_listed_and_updated_without_loss() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let project = project();
        let name = OsString::from_vec(b"source-\xff.rs".to_vec());
        fs::write(
            project.path().join(&name),
            fixture("cases/mod_use_groups", "input.rs"),
        )
        .unwrap();
        let display = Path::new(".").join(&name);
        let output = execute(project.path(), &["files", "."]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stdout, format!("{display:?}\n").as_bytes());
        let output = execute(project.path(), &["write", "."]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(
            output.stderr,
            format!("reformatted: {display:?}\n").as_bytes()
        );
        assert_eq!(
            fs::read(project.path().join(&name)).unwrap(),
            fixture("cases/mod_use_groups", "expected.rs")
        );
    }
}
