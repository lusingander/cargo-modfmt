pub mod common;

use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::{self, Permissions},
    path::{Path, PathBuf},
    process::Output,
    time::SystemTime,
};

use assert_cmd::cargo::cargo_bin_cmd;
use rstest::rstest;
use tempfile::tempdir;

use crate::common::{command, fixture};

fn execute(directory: &Path, args: &[&str]) -> Output {
    command(directory)
        .args(args)
        .write_stdin(b"\xff".to_vec())
        .output()
        .unwrap()
}

fn assert_result(output: Output, code: i32, path: &OsStr) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = if code == 3 {
        format!("would reformat: {path:?}\n").into_bytes()
    } else {
        Vec::new()
    };
    assert_eq!(output.stdout, expected);
    assert_eq!(output.stderr.is_empty(), code == 0 || code == 3);
    if code == 1 {
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("{path:?}")));
    }
}

fn assert_written(output: Output, path: &OsStr, changed: bool) {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stdout.is_empty());
    let expected = if changed {
        format!("reformatted: {path:?}\n").into_bytes()
    } else {
        Vec::new()
    };
    assert_eq!(output.stderr, expected);
}

fn tree(directory: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    fn collect(root: &Path, directory: &Path, files: &mut BTreeMap<PathBuf, Option<Vec<u8>>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap().to_owned();
            if entry.file_type().unwrap().is_dir() {
                files.insert(relative, None);
                collect(root, &path, files);
            } else {
                files.insert(relative, Some(fs::read(path).unwrap()));
            }
        }
    }
    let mut files = BTreeMap::new();
    collect(directory, directory, &mut files);
    files
}

fn stamp(path: &Path) -> (Permissions, SystemTime) {
    let metadata = fs::metadata(path).unwrap();
    (metadata.permissions(), metadata.modified().unwrap())
}

#[rstest]
#[case::project_like("compile/project_like")]
#[case::basic_spec("cases/mod_use_groups")]
#[case::skip_comments("cases/skip_comments")]
#[case::skip_attributes("cases/skip_attributes")]
#[case::unsupported_groups("cases/use_tree_root_and_empty")]
#[case::crlf_bom_eof("cases/use_layout_crlf_bom_no_final_newline")]
#[case::mixed_endings("cases/use_layout_mixed_line_endings")]
#[case::inner_header_crlf_bom_eof("cases/inner_header_crlf_bom_no_final_newline")]
#[case::inner_header_crlf_protected("cases/inner_header_crlf_protected")]
#[case::inner_header_mixed_endings("cases/inner_header_mixed_line_endings")]
#[case::inner_header_only("cases/inner_header_only")]
#[case::empty("cases/empty")]
#[case::bom_only("cases/bom_only")]
fn file_round_trip(#[case] case: &str) {
    let project = tempdir().unwrap();
    let path = project.path().join("main.rs");
    let input = fixture(case, "input.rs");
    let expected = fixture(case, "expected.rs");
    fs::write(&path, &input).unwrap();
    if case == "compile/project_like" {
        for name in ["engine.rs", "model.rs"] {
            fs::write(
                project.path().join(name),
                fixture(case, &format!("modules/{name}")),
            )
            .unwrap();
        }
    }
    let before = tree(project.path());
    let metadata = stamp(&path);
    let code = if input == expected { 0 } else { 3 };
    assert_result(
        execute(project.path(), &["check", "main.rs"]),
        code,
        OsStr::new("main.rs"),
    );
    assert_eq!(tree(project.path()), before);
    assert_eq!(stamp(&path), metadata);
    assert_written(
        execute(project.path(), &["write", "main.rs"]),
        OsStr::new("main.rs"),
        code == 3,
    );
    let mut after = before;
    after.insert("main.rs".into(), Some(expected));
    assert_eq!(tree(project.path()), after);
    let metadata = stamp(&path);
    for mode in ["check", "write"] {
        assert_result(
            execute(project.path(), &[mode, "main.rs"]),
            0,
            OsStr::new("main.rs"),
        );
        assert_eq!(tree(project.path()), after);
        assert_eq!(stamp(&path), metadata);
    }
}

#[rstest]
#[case::invalid_utf8("invalid_utf8")]
#[case::broken_item("broken_item")]
#[case::broken_use("broken_use")]
#[case::broken_macro("broken_macro")]
#[case::broken_inner_header("broken_inner_header")]
fn input_errors_preserve_file(#[case] case: &str, #[values("check", "write")] mode: &str) {
    let project = tempdir().unwrap();
    let path = project.path().join("main.rs");
    fs::write(&path, fixture(&format!("errors/{case}"), "input.rs")).unwrap();
    let before = tree(project.path());
    let metadata = stamp(&path);
    assert_result(
        execute(project.path(), &[mode, "main.rs"]),
        1,
        OsStr::new("main.rs"),
    );
    assert_eq!(tree(project.path()), before);
    assert_eq!(stamp(&path), metadata);
}

#[rstest]
fn edition_selection(
    #[values("2015", "2018", "2021", "2024", "default")] edition: &str,
    #[values("check", "write")] mode: &str,
) {
    let project = tempdir().unwrap();
    let path = project.path().join("main.rs");
    let input = fixture("cases/edition_keyword", "input.rs");
    fs::write(&path, &input).unwrap();
    let mut args = vec![mode, "main.rs"];
    if edition != "default" {
        args.extend(["--edition", edition]);
    }
    let error = edition == "2024" || edition == "default";
    let code = if error {
        1
    } else if mode == "check" {
        3
    } else {
        0
    };
    let output = execute(project.path(), &args);
    if mode == "write" && !error {
        assert_written(output, OsStr::new("main.rs"), true);
    } else {
        assert_result(output, code, OsStr::new("main.rs"));
    }
    let expected = if !error && mode == "write" {
        fixture("cases/edition_keyword", "expected.rs")
    } else {
        input
    };
    assert_eq!(fs::read(&path).unwrap(), expected);
    assert_eq!(fs::read_dir(project.path()).unwrap().count(), 1);
}

#[rstest]
#[case::spaces("source file.rs", false)]
#[case::unicode("ソース.rs", false)]
#[case::hyphen("-sample.rs", true)]
#[case::dash("-", true)]
#[case::extension("source.txt", false)]
#[case::relative("./source.rs", false)]
#[case::escaped("line\nbreak.rs", false)]
fn file_names(#[case] name: &str, #[case] separator: bool) {
    let project = tempdir().unwrap();
    let input = fixture("cases/mod_use_groups", "input.rs");
    let expected = fixture("cases/mod_use_groups", "expected.rs");
    fs::write(project.path().join(name), &input).unwrap();
    for (mode, code) in [("check", 3), ("write", 0), ("check", 0)] {
        let args = if separator {
            vec![mode, "--", name]
        } else {
            vec![mode, name]
        };
        let output = execute(project.path(), &args);
        if mode == "write" {
            assert_written(output, OsStr::new(name), true);
        } else {
            assert_result(output, code, OsStr::new(name));
        }
        assert_eq!(
            fs::read(project.path().join(name)).unwrap().as_slice(),
            if code == 3 { &input } else { &expected }
        );
    }
}

#[rstest]
fn missing_file(#[values("check", "write")] mode: &str) {
    let name = "missing.rs";
    let project = tempdir().unwrap();
    fs::create_dir(project.path().join("directory")).unwrap();
    let before = tree(project.path());
    assert_result(execute(project.path(), &[mode, name]), 1, OsStr::new(name));
    assert_eq!(tree(project.path()), before);
}

#[cfg(unix)]
mod unix {
    use std::{
        ffi::OsString,
        os::unix::{
            ffi::OsStringExt,
            fs::{MetadataExt, PermissionsExt, symlink},
        },
        process::Command,
    };

    use super::*;

    struct RestorePermissions(PathBuf, Permissions);

    impl RestorePermissions {
        fn set(path: &Path, mode: u32) -> Self {
            let previous = fs::metadata(path).unwrap().permissions();
            fs::set_permissions(path, Permissions::from_mode(mode)).unwrap();
            Self(path.to_owned(), previous)
        }
    }

    impl Drop for RestorePermissions {
        fn drop(&mut self) {
            let _ = fs::set_permissions(&self.0, self.1.clone());
        }
    }

    #[test]
    fn non_utf8_argument_is_a_path() {
        let project = tempdir().unwrap();
        let name = OsString::from_vec(b"missing-\xff.rs".to_vec());
        for mode in ["check", "write"] {
            let output = cargo_bin_cmd!()
                .arg("modfmt")
                .current_dir(project.path())
                .arg(mode)
                .arg(&name)
                .output()
                .unwrap();
            assert_result(output, 1, &name);
        }
        assert_eq!(fs::read_dir(project.path()).unwrap().count(), 0);
    }

    #[test]
    #[cfg_attr(
        target_os = "macos",
        ignore = "local macOS filesystem rejects non-UTF-8 filenames (EILSEQ); argument handling is tested separately"
    )]
    fn non_utf8_path() {
        let project = tempdir().unwrap();
        let name = OsString::from_vec(b"source-\xff.rs".to_vec());
        fs::write(
            project.path().join(&name),
            fixture("cases/mod_use_groups", "input.rs"),
        )
        .unwrap();
        for (mode, code) in [("check", 3), ("write", 0)] {
            let output = cargo_bin_cmd!()
                .arg("modfmt")
                .current_dir(project.path())
                .arg(mode)
                .arg(&name)
                .write_stdin(b"\xff".to_vec())
                .output()
                .unwrap();
            if mode == "write" {
                assert_written(output, &name, true);
            } else {
                assert_result(output, code, &name);
            }
        }
        assert_eq!(
            fs::read(project.path().join(name)).unwrap(),
            fixture("cases/mod_use_groups", "expected.rs")
        );
    }

    #[rstest]
    #[case::symlink("symlink")]
    #[case::dangling("dangling")]
    #[case::fifo("fifo")]
    fn special_files(#[case] kind: &str, #[values("check", "write")] mode: &str) {
        let project = tempdir().unwrap();
        let path = project.path().join("source");
        let input = fixture("cases/mod_use_groups", "input.rs");
        fs::write(project.path().join("target.rs"), &input).unwrap();
        match kind {
            "symlink" => symlink("target.rs", &path).unwrap(),
            "dangling" => symlink("absent.rs", &path).unwrap(),
            "fifo" => assert!(
                Command::new("mkfifo")
                    .arg(&path)
                    .status()
                    .unwrap()
                    .success()
            ),
            _ => unreachable!(),
        }
        let before = fs::symlink_metadata(&path).unwrap();
        assert_result(
            execute(project.path(), &[mode, "source"]),
            1,
            OsStr::new("source"),
        );
        assert_eq!(fs::symlink_metadata(&path).unwrap().ino(), before.ino());
        assert_eq!(fs::read(project.path().join("target.rs")).unwrap(), input);
        assert_eq!(fs::read_dir(project.path()).unwrap().count(), 2);
    }

    #[test]
    fn parent_symlink_and_hardlink() {
        let project = tempdir().unwrap();
        let directory = project.path().join("real");
        fs::create_dir(&directory).unwrap();
        symlink("real", project.path().join("linked")).unwrap();
        let path = directory.join("main.rs");
        let alias = directory.join("alias.rs");
        let input = fixture("cases/mod_use_groups", "input.rs");
        fs::write(&path, &input).unwrap();
        fs::hard_link(&path, &alias).unwrap();
        let inode = fs::metadata(&alias).unwrap().ino();
        assert_result(
            execute(project.path(), &["check", "linked/main.rs"]),
            3,
            OsStr::new("linked/main.rs"),
        );
        assert_eq!(fs::read(&alias).unwrap(), input);
        assert_written(
            execute(project.path(), &["write", "linked/main.rs"]),
            OsStr::new("linked/main.rs"),
            true,
        );
        assert_eq!(
            fs::read(&path).unwrap(),
            fixture("cases/mod_use_groups", "expected.rs")
        );
        assert_ne!(fs::metadata(&path).unwrap().ino(), inode);
        assert_eq!(fs::metadata(&alias).unwrap().ino(), inode);
        assert_eq!(fs::read(&alias).unwrap(), input);
        assert_result(
            execute(project.path(), &["check", "linked/main.rs"]),
            0,
            OsStr::new("linked/main.rs"),
        );
        assert_result(
            execute(project.path(), &["check", "real/alias.rs"]),
            3,
            OsStr::new("real/alias.rs"),
        );
        assert_eq!(fs::read_dir(directory).unwrap().count(), 2);
    }

    #[rstest]
    #[case::read_only(false, 0o444)]
    #[case::executable(false, 0o754)]
    #[case::unwritable_parent(true, 0o555)]
    fn permissions(
        #[case] parent: bool,
        #[case] mode: u32,
        #[values(false, true)] unchanged: bool,
    ) {
        let project = tempdir().unwrap();
        let directory = project.path().join("source");
        fs::create_dir(&directory).unwrap();
        let path = directory.join("main.rs");
        let input = fixture(
            "cases/mod_use_groups",
            if unchanged { "expected.rs" } else { "input.rs" },
        );
        fs::write(&path, &input).unwrap();
        let _restore = RestorePermissions::set(if parent { &directory } else { &path }, mode);
        let before = tree(&directory);
        let metadata = stamp(&path);
        assert_result(
            execute(&directory, &["check", "main.rs"]),
            if unchanged { 0 } else { 3 },
            OsStr::new("main.rs"),
        );
        assert_eq!(tree(&directory), before);
        assert_eq!(stamp(&path), metadata);
        let fail = !unchanged && (parent || mode == 0o444);
        let output = execute(&directory, &["write", "main.rs"]);
        if fail {
            assert_result(output, 1, OsStr::new("main.rs"));
        } else {
            assert_written(output, OsStr::new("main.rs"), !unchanged);
        }
        if fail || unchanged {
            assert_eq!(tree(&directory), before);
            assert_eq!(stamp(&path), metadata);
        } else {
            assert_eq!(
                fs::read(&path).unwrap(),
                fixture("cases/mod_use_groups", "expected.rs")
            );
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                mode
            );
        }
    }

    #[rstest]
    fn unreadable_source(#[values("check", "write")] mode: &str) {
        let project = tempdir().unwrap();
        let path = project.path().join("main.rs");
        let input = fixture("cases/mod_use_groups", "input.rs");
        fs::write(&path, &input).unwrap();
        {
            let _restore = RestorePermissions::set(&path, 0o000);
            assert_result(
                execute(project.path(), &[mode, "main.rs"]),
                1,
                OsStr::new("main.rs"),
            );
        }
        assert_eq!(fs::read(&path).unwrap(), input);
        assert_eq!(fs::read_dir(project.path()).unwrap().count(), 1);
    }
}
