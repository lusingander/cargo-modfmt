pub mod common;

use std::{fs, path::Path};

use rstest::rstest;
use tempfile::{TempDir, tempdir};

use crate::common::command;

const INPUT: &str = "use vendor::{Zulu, Gamma, Beta, Alpha};\n";
const INLINE: &str = "use vendor::{Alpha, Beta, Gamma, Zulu};\n";
const WRAPPED: &str = "use vendor::{\n    Alpha, Beta,\n    Gamma, Zulu,\n};\n";

fn project(config: &str) -> TempDir {
    let directory = tempdir().unwrap();
    fs::create_dir(directory.path().join(".git")).unwrap();
    fs::write(directory.path().join("rustfmt.toml"), config).unwrap();
    directory
}

fn organize(cwd: &Path, args: &[&str], source: &str) -> String {
    let output = command(cwd)
        .arg("stdin")
        .args(args)
        .write_stdin(source)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

#[rstest]
#[case::defaults("", INLINE)]
#[case::wide("max_width = 120", INLINE)]
#[case::narrow("max_width = 20", WRAPPED)]
#[case::spaces(
    "max_width = 20\ntab_spaces = 2",
    "use vendor::{\n  Alpha, Beta,\n  Gamma, Zulu,\n};\n"
)]
#[case::tabs(
    "max_width = 20\nhard_tabs = true",
    "use vendor::{\n\tAlpha, Beta,\n\tGamma, Zulu,\n};\n"
)]
#[case::two_column_tabs(
    "max_width = 20\ntab_spaces = 2\nhard_tabs = true",
    "use vendor::{\n\tAlpha, Beta,\n\tGamma, Zulu,\n};\n"
)]
#[case::zero_indent(
    "max_width = 20\ntab_spaces = 0",
    "use vendor::{\nAlpha, Beta, Gamma,\nZulu,\n};\n"
)]
fn rustfmt_layout_is_used_even_with_explicit_editions(
    #[case] config: &str,
    #[case] expected: &str,
) {
    let directory = project(config);
    let args = ["--edition", "2024", "--style-edition", "2024"];
    let organized = organize(directory.path(), &args, INPUT);
    assert_eq!(organized, expected);
    assert_eq!(organize(directory.path(), &args, &organized), organized);
}

#[test]
fn nearest_rustfmt_file_is_used_without_merging_and_hidden_name_wins() {
    let directory = project("max_width = 10\ntab_spaces = 2\nhard_tabs = true");
    let child = directory.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::write(child.join("rustfmt.toml"), "max_width = 20").unwrap();
    assert_eq!(organize(&child, &[], INPUT), WRAPPED);
    fs::write(child.join(".rustfmt.toml"), "").unwrap();
    assert_eq!(organize(&child, &[], INPUT), INLINE);
}

#[test]
fn rustfmt_lookup_still_stops_at_repository_root() {
    let directory = project("max_width = 20");
    let child = directory.path().join("child");
    fs::create_dir(&child).unwrap();
    fs::create_dir(child.join(".git")).unwrap();
    assert_eq!(organize(&child, &[], INPUT), INLINE);
}

#[test]
fn each_file_uses_its_own_layout_independently_of_cwd_and_modfmt_config() {
    let directory = project("max_width = 120");
    fs::write(directory.path().join("modfmt.toml"), "").unwrap();
    let narrow = directory.path().join("narrow");
    let wide = directory.path().join("wide");
    for (path, config) in [(&narrow, "max_width = 20"), (&wide, "max_width = 120")] {
        fs::create_dir(path).unwrap();
        fs::write(path.join("rustfmt.toml"), config).unwrap();
        for name in ["first.rs", "second.rs"] {
            fs::write(path.join(name), INPUT).unwrap();
        }
    }
    let other = project("max_width = 10");
    let output = command(other.path())
        .arg("write")
        .arg(&narrow)
        .arg(&wide)
        .arg("--config")
        .arg(directory.path().join("modfmt.toml"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    for (path, expected) in [(&narrow, WRAPPED), (&wide, INLINE)] {
        for name in ["first.rs", "second.rs"] {
            assert_eq!(fs::read_to_string(path.join(name)).unwrap(), expected);
        }
    }
}

#[rstest]
#[case::syntax("invalid TOML")]
#[case::negative_width("max_width = -1")]
#[case::string_width("max_width = '80'")]
#[case::float_width("max_width = 80.5")]
#[case::negative_tab_spaces("tab_spaces = -1")]
#[case::string_tab_spaces("tab_spaces = '2'")]
#[case::string_hard_tabs("hard_tabs = 'true'")]
#[case::zero_tab_width("tab_spaces = 0\nhard_tabs = true")]
fn invalid_layout_is_checked_before_listing_or_processing_any_files(
    #[case] config: &str,
    #[values("files", "check", "write")] mode: &str,
) {
    let directory = project("");
    for name in ["first", "second"] {
        let child = directory.path().join(name);
        fs::create_dir(&child).unwrap();
        fs::write(child.join("source.rs"), INPUT).unwrap();
    }
    fs::write(directory.path().join("second/rustfmt.toml"), config).unwrap();
    let output = command(directory.path())
        .args([
            mode,
            "first",
            "second",
            "--edition",
            "2024",
            "--style-edition",
            "2024",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("rustfmt.toml"));
    for name in ["first", "second"] {
        assert_eq!(
            fs::read_to_string(directory.path().join(name).join("source.rs")).unwrap(),
            INPUT
        );
    }
}

#[test]
fn zero_width_does_not_panic() {
    let directory = project("max_width = 0\ntab_spaces = 0");
    let organized = organize(directory.path(), &[], INPUT);
    assert_eq!(organize(directory.path(), &[], &organized), organized);
}
