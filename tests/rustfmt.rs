pub mod common;

use std::{ffi::OsString, fs, path::Path, process::Command};

use rstest::rstest;

use crate::common::{command, copy_directory, fixture_path};

fn organize(cwd: &Path, source: &[u8], style: &str) -> Vec<u8> {
    let output = command(cwd)
        .args(["stdin", "--edition", "2024", "--style-edition", style])
        .write_stdin(source.to_vec())
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    assert!(output.stderr.is_empty(), "{:?}", output);
    output.stdout
}

fn format(formatter: &OsString, version: &str, config: &Path, source: &Path, check: bool) {
    let mut command = Command::new(formatter);
    command.arg("--config-path").arg(config);
    if check {
        command.arg("--check");
    }
    let output = command.arg(source).output().unwrap();
    assert!(
        output.status.success(),
        "{version}, check={check}, {source:?}: {output:?}"
    );
    assert!(output.stderr.is_empty(), "{version}: {output:?}");
}

#[rstest]
#[case::groups("cases/mod_use_groups/input.rs", &["alpha", "beta", "zebra", "utils"])]
#[case::raw("cases/use_raw_order/input.rs", &["alpha", "beta", "gamma", "match", "zeta"])]
#[case::merge("cases/use_layout_merge/input.rs", &["alpha", "zebra"])]
#[case::aliases("cases/use_alias_order/input.rs", &[])]
#[case::normalization("cases/use_normalization/input.rs", &[])]
#[case::visibility("cases/use_merge_visibility/input.rs", &[])]
#[case::nested("cases/use_tree_nested/input.rs", &[])]
#[case::width("cases/use_layout_width/input.rs", &[])]
#[case::unicode("cases/use_layout_unicode_tabs/input.rs", &[])]
#[case::atomic_paths("cases/use_layout_atomic/input.rs", &[])]
#[case::protected_scopes("compile/project_like/input.rs", &[])]
#[case::self_namespace("compile/self_namespace/expected.rs", &[])]
#[case::style_order("cases/style_order/input.rs", &[])]
#[case::inner_headers("compile/inner_headers/input.rs", &[])]
fn organizing_formatted_source_preserves_stable_check(
    #[case] fixture: &str,
    #[case] modules: &[&str],
    #[values("2015", "2018", "2021", "2024")] style: &str,
) {
    check_compatibility(fixture, modules, style, "", false);
}

#[rstest]
#[case::narrow(80, 4, false)]
#[case::default_width(100, 4, false)]
#[case::wide(120, 4, false)]
#[case::spaces(80, 2, false)]
#[case::tabs(80, 4, true)]
#[case::two_column_tabs(80, 2, true)]
fn custom_layout_preserves_stable_check(
    #[case] width: usize,
    #[case] tab_spaces: usize,
    #[case] hard_tabs: bool,
    #[values(
        "cases/use_layout_width/input.rs",
        "cases/use_layout_nested/input.rs",
        "cases/use_layout_unicode_tabs/input.rs",
        "cases/use_layout_merge/input.rs",
        "cases/use_layout_atomic/input.rs",
        "cases/use_layout_indentation/input.rs"
    )]
    fixture: &str,
    #[values("2015", "2018", "2021", "2024")] style: &str,
    #[values(false, true)] crlf: bool,
) {
    let settings = format!(
        r#"max_width = {width}
tab_spaces = {tab_spaces}
hard_tabs = {hard_tabs}
"#
    );
    let modules: &[&str] = match fixture {
        "cases/use_layout_merge/input.rs" => &["alpha", "zebra"],
        _ => &[],
    };
    check_compatibility(fixture, modules, style, &settings, crlf);
}

fn check_compatibility(fixture: &str, modules: &[&str], style: &str, settings: &str, crlf: bool) {
    let formatter = std::env::var_os("RUSTFMT").unwrap_or_else(|| "rustfmt".into());
    let version = Command::new(&formatter).arg("--version").output().unwrap();
    assert!(version.status.success(), "{version:?}");
    let version = String::from_utf8(version.stdout).unwrap();
    assert!(
        version.contains("-stable"),
        "stable rustfmt required: {version}"
    );
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("rustfmt.toml");
    fs::write(
        &config,
        format!(
            r#"edition = "2024"
style_edition = "{style}"
{settings}"#
        ),
    )
    .unwrap();
    let fixture = fixture_path(fixture);
    let module_directory = fixture.parent().unwrap().join("modules");
    if module_directory.exists() {
        copy_directory(&module_directory, directory.path());
    }
    for module in modules {
        fs::write(
            directory.path().join(format!("{module}.rs")),
            "// Module stub for formatting checks.\n",
        )
        .unwrap();
    }
    let source = directory.path().join("main.rs");
    let mut contents = fs::read_to_string(fixture).unwrap();
    if crlf {
        contents = contents.replace("\r\n", "\n").replace('\n', "\r\n");
    }
    fs::write(&source, contents).unwrap();
    format(&formatter, &version, &config, &source, false);
    format(&formatter, &version, &config, &source, true);

    let organized = organize(directory.path(), &fs::read(&source).unwrap(), style);
    fs::write(&source, &organized).unwrap();
    format(&formatter, &version, &config, &source, true);
    assert_eq!(
        organize(directory.path(), &organized, style),
        organized,
        "tool must be idempotent"
    );

    format(&formatter, &version, &config, &source, false);
    assert_eq!(fs::read(&source).unwrap(), organized, "{version}");
    let checked = command(Path::new(env!("CARGO_MANIFEST_DIR")))
        .arg("check")
        .arg(&source)
        .output()
        .unwrap();
    assert!(checked.status.success(), "{checked:?}");
    assert!(checked.stdout.is_empty() && checked.stderr.is_empty());
}
