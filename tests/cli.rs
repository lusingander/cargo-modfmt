pub mod common;

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use assert_cmd::cargo::cargo_bin_cmd;
use rstest::rstest;

use crate::common::{command, copy_directory};

fn execute(args: &[&str], input: &[u8]) -> Output {
    command(Path::new(env!("CARGO_MANIFEST_DIR")))
        .args(args)
        .write_stdin(input.to_vec())
        .output()
        .unwrap()
}

fn assert_source(actual: &[u8], expected: &[u8], case: &str) {
    if actual == expected {
        return;
    }
    let actual = String::from_utf8_lossy(actual);
    let expected = String::from_utf8_lossy(expected);
    let actual_lines: Vec<_> = actual.split_inclusive('\n').collect();
    let expected_lines: Vec<_> = expected.split_inclusive('\n').collect();
    let mut diff = String::new();
    for index in 0..actual_lines.len().max(expected_lines.len()) {
        let old = expected_lines.get(index).copied().unwrap_or("");
        let new = actual_lines.get(index).copied().unwrap_or("");
        if old != new {
            diff.push_str(&format!(
                r#"line {}:
- {old:?}
+ {new:?}
"#,
                index + 1
            ));
        }
    }
    panic!(
        r#"{case}: source differs
{diff}"#
    );
}

fn check_source(
    input: &[u8],
    expected: Option<&[u8]>,
    case: &str,
    edition: &str,
    style: &str,
) -> Vec<u8> {
    let args = ["stdin", "--edition", edition, "--style-edition", style];
    let output = execute(&args, input);
    assert!(
        output.status.success(),
        "{case}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "{case}: unexpected stderr");
    if let Some(expected) = expected {
        assert_source(&output.stdout, expected, case);
    }

    let repeated = execute(&args, &output.stdout);
    assert!(repeated.status.success(), "{case}: second run failed");
    assert!(repeated.stderr.is_empty(), "{case}: second run stderr");
    assert_source(&repeated.stdout, &output.stdout, case);
    output.stdout
}

#[rstest]
#[case::style_order("style_order")]
#[case::mod_basic("mod_basic")]
#[case::item_boundaries("item_boundaries")]
#[case::skip_comments("skip_comments")]
#[case::skip_attributes("skip_attributes")]
#[case::adjacent_docs_attrs("adjacent_docs_attrs")]
#[case::independent_runs("independent_runs")]
#[case::mod_names("mod_names")]
#[case::mod_visibility("mod_visibility")]
#[case::inline_modules("inline_modules")]
#[case::inline_comments("inline_comments")]
#[case::inline_attributes("inline_attributes")]
#[case::inline_scope_headers("inline_scope_headers")]
#[case::inline_declaration_runs("inline_declaration_runs")]
#[case::inline_same_line("inline_same_line")]
#[case::inline_crlf("inline_crlf")]
#[case::inline_mixed_line_endings("inline_mixed_line_endings")]
#[case::same_line_indent("same_line_indent")]
#[case::crlf_bom_no_final_newline("crlf_bom_no_final_newline")]
#[case::mixed_line_endings("mixed_line_endings")]
#[case::empty("empty")]
#[case::bom_only("bom_only")]
#[case::no_declarations("no_declarations")]
#[case::one_declaration("one_declaration")]
#[case::already_sorted("already_sorted")]
#[case::no_final_newline("no_final_newline")]
#[case::same_line_without_newlines("same_line_without_newlines")]
#[case::first_indentation("first_indentation")]
#[case::same_line_items("same_line_items")]
#[case::crlf_fallback("crlf_fallback")]
#[case::mixed_fallback("mixed_fallback")]
#[case::mixed_last_ending("mixed_last_ending")]
#[case::inner_attribute("inner_attribute")]
#[case::inner_documentation("inner_documentation")]
#[case::inner_header_guards("inner_header_guards")]
#[case::inner_header_comments("inner_header_comments")]
#[case::inner_header_crlf_bom_no_final_newline("inner_header_crlf_bom_no_final_newline")]
#[case::inner_header_crlf_protected("inner_header_crlf_protected")]
#[case::inner_header_mixed_line_endings("inner_header_mixed_line_endings")]
#[case::inner_header_only("inner_header_only")]
#[case::body_and_attribute_comments("body_and_attribute_comments")]
#[case::previous_item_comment("previous_item_comment")]
#[case::use_groups("use_groups")]
#[case::use_path_order("use_path_order")]
#[case::use_visibility("use_visibility")]
#[case::mod_use_groups("mod_use_groups")]
#[case::use_scopes("use_scopes")]
#[case::use_mixed_forms("use_mixed_forms")]
#[case::use_comments_attributes("use_comments_attributes")]
#[case::use_source_layout("use_source_layout")]
#[case::use_crlf("use_crlf")]
#[case::use_mixed_line_endings("use_mixed_line_endings")]
#[case::use_path_boundaries("use_path_boundaries")]
#[case::use_tree_basic("use_tree_basic")]
#[case::use_normalization("use_normalization")]
#[case::use_tree_nested("use_tree_nested")]
#[case::use_tree_order("use_tree_order")]
#[case::use_tree_groups("use_tree_groups")]
#[case::use_tree_layout("use_tree_layout")]
#[case::use_tree_scopes("use_tree_scopes")]
#[case::use_tree_comments_attributes("use_tree_comments_attributes")]
#[case::use_tree_forms("use_tree_forms")]
#[case::use_tree_root_and_empty("use_tree_root_and_empty")]
#[case::use_tree_crlf_bom_no_final_newline("use_tree_crlf_bom_no_final_newline")]
#[case::use_tree_mixed_line_endings("use_tree_mixed_line_endings")]
#[case::use_merge_basic("use_merge_basic")]
#[case::use_merge_nested("use_merge_nested")]
#[case::use_merge_groups("use_merge_groups")]
#[case::use_merge_visibility("use_merge_visibility")]
#[case::use_merge_conflicts("use_merge_conflicts")]
#[case::use_merge_qualifiers("use_merge_qualifiers")]
#[case::use_merge_layout_order("use_merge_layout_order")]
#[case::use_merge_scopes("use_merge_scopes")]
#[case::use_merge_guards("use_merge_guards")]
#[case::use_merge_crlf_bom_no_final_newline("use_merge_crlf_bom_no_final_newline")]
#[case::use_merge_mixed_line_endings("use_merge_mixed_line_endings")]
#[case::use_self_order("use_self_order")]
#[case::use_alias_order("use_alias_order")]
#[case::use_self_merge("use_self_merge")]
#[case::use_alias_merge("use_alias_merge")]
#[case::use_alias_merge_guards("use_alias_merge_guards")]
#[case::use_alias_visibility_layout("use_alias_visibility_layout")]
#[case::use_alias_scopes("use_alias_scopes")]
#[case::use_alias_guards("use_alias_guards")]
#[case::use_alias_crlf_bom_no_final_newline("use_alias_crlf_bom_no_final_newline")]
#[case::use_alias_mixed_line_endings("use_alias_mixed_line_endings")]
#[case::use_glob_order("use_glob_order")]
#[case::use_glob_merge("use_glob_merge")]
#[case::use_glob_guards("use_glob_guards")]
#[case::use_absolute_roots("use_absolute_roots")]
#[case::use_raw_order("use_raw_order")]
#[case::use_raw_merge("use_raw_merge")]
#[case::use_raw_guards("use_raw_guards")]
#[case::use_glob_raw_visibility_layout("use_glob_raw_visibility_layout")]
#[case::use_glob_raw_scopes("use_glob_raw_scopes")]
#[case::use_glob_raw_guards("use_glob_raw_guards")]
#[case::use_glob_crlf_bom_no_final_newline("use_glob_crlf_bom_no_final_newline")]
#[case::use_glob_raw_mixed_line_endings("use_glob_raw_mixed_line_endings")]
#[case::use_layout_short("use_layout_short")]
#[case::use_layout_width("use_layout_width")]
#[case::use_layout_nested("use_layout_nested")]
#[case::use_layout_indentation("use_layout_indentation")]
#[case::use_layout_same_line("use_layout_same_line")]
#[case::use_layout_trivia("use_layout_trivia")]
#[case::use_layout_atomic("use_layout_atomic")]
#[case::use_layout_unicode_tabs("use_layout_unicode_tabs")]
#[case::use_layout_merge("use_layout_merge")]
#[case::use_layout_guards("use_layout_guards")]
#[case::use_layout_crlf_bom_no_final_newline("use_layout_crlf_bom_no_final_newline")]
#[case::use_layout_mixed_line_endings("use_layout_mixed_line_endings")]
#[case::use_layout_no_newlines("use_layout_no_newlines")]
#[case::use_layout_crlf_fallback("use_layout_crlf_fallback")]
fn fixture_sources(#[case] case: &str) {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/cases")
        .join(case);
    let input = std::fs::read(directory.join("input.rs")).unwrap();
    let expected = std::fs::read(directory.join("expected.rs")).unwrap();
    check_source(&input, Some(&expected), case, "2024", "2024");
}

#[rstest]
#[case::mod_visibility("mod_visibility", "2024", &[], b"2047\n")]
#[case::mod_visibility_test_cfg("mod_visibility", "2024", &["--cfg", "test"], b"8191\n")]
#[case::simple_use("simple_use", "2024", &[], b"1028\n")]
#[case::nested_use("nested_use", "2024", &[], b"284\n")]
#[case::merged_use("merged_use", "2024", &[], b"296\n")]
#[case::self_alias_2015("self_alias", "2015", &[], b"264\n")]
#[case::self_alias_2018("self_alias", "2018", &[], b"264\n")]
#[case::self_alias_2021("self_alias", "2021", &[], b"264\n")]
#[case::self_alias_2024("self_alias", "2024", &[], b"264\n")]
#[case::self_namespace_2015("self_namespace", "2015", &[], b"111\n")]
#[case::self_namespace_2018("self_namespace", "2018", &[], b"111\n")]
#[case::self_namespace_2021("self_namespace", "2021", &[], b"111\n")]
#[case::self_namespace_2024("self_namespace", "2024", &[], b"111\n")]
#[case::glob_semantics_2015("glob_semantics", "2015", &[], b"119\n")]
#[case::glob_semantics_2018("glob_semantics", "2018", &[], b"119\n")]
#[case::glob_semantics_2021("glob_semantics", "2021", &[], b"119\n")]
#[case::glob_semantics_2024("glob_semantics", "2024", &[], b"119\n")]
#[case::raw_symbols_2015("raw_symbols", "2015", &[], b"78\n")]
#[case::raw_symbols_2018("raw_symbols", "2018", &[], b"78\n")]
#[case::raw_symbols_2021("raw_symbols", "2021", &[], b"78\n")]
#[case::raw_symbols_2024("raw_symbols", "2024", &[], b"78\n")]
#[case::absolute_roots_2015("absolute_roots", "2015", &[], b"48\n")]
#[case::absolute_roots_2018("absolute_roots", "2018", &[], b"27\n")]
#[case::absolute_roots_2021("absolute_roots", "2021", &[], b"27\n")]
#[case::absolute_roots_2024("absolute_roots", "2024", &[], b"27\n")]
#[case::multiline_use_2015("multiline_use", "2015", &[], b"66\n")]
#[case::multiline_use_2018("multiline_use", "2018", &[], b"66\n")]
#[case::multiline_use_2021("multiline_use", "2021", &[], b"66\n")]
#[case::multiline_use_2024("multiline_use", "2024", &[], b"66\n")]
#[case::project_like_2015("project_like", "2015", &[], b"103\n")]
#[case::project_like_2018("project_like", "2018", &[], b"103\n")]
#[case::project_like_2021("project_like", "2021", &[], b"103\n")]
#[case::project_like_2024("project_like", "2024", &[], b"103\n")]
#[case::inner_headers("inner_headers", "2024", &[], b"19\n")]
#[case::inner_headers_test_cfg("inner_headers", "2024", &["--cfg", "test"], b"24\n")]
fn compiled_sources_preserve_behavior(
    #[case] case: &str,
    #[case] edition: &str,
    #[case] cfg_args: &[&str],
    #[case] expected_stdout: &[u8],
) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/compile")
        .join(case);
    let input = fs::read(fixture.join("input.rs")).unwrap();
    let expected = fs::read(fixture.join("expected.rs")).unwrap();
    let legacy = check_source(
        &input,
        None,
        &format!("{case}/{edition}/2021"),
        edition,
        "2021",
    );
    let current = check_source(
        &input,
        Some(&expected),
        &format!("{case}/{edition}/2024"),
        edition,
        "2024",
    );
    let project = tempfile::Builder::new()
        .prefix(&format!("cargo-modfmt-{case}-"))
        .tempdir()
        .unwrap();
    copy_directory(&fixture.join("modules"), project.path());
    let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let vendor_source = fixture.join("dependencies/vendor.rs");
    let vendor_library = project.path().join("libvendor.rlib");
    if vendor_source.exists() {
        let compiled = Command::new(&compiler)
            .args([
                "--edition",
                edition,
                "--crate-type",
                "rlib",
                "--crate-name",
                "vendor",
            ])
            .arg(&vendor_source)
            .arg("-o")
            .arg(&vendor_library)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{case}/{edition}/vendor: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
    }

    for (version, source) in [
        ("original", input.as_slice()),
        ("style_2021", legacy.as_slice()),
        ("style_2024", current.as_slice()),
    ] {
        fs::write(project.path().join("main.rs"), source).unwrap();
        let executable = project
            .path()
            .join(format!("{version}{}", std::env::consts::EXE_SUFFIX));
        let mut command = Command::new(&compiler);
        command
            .current_dir(project.path())
            .args(["--edition", edition, "--crate-name", case])
            .args(cfg_args);
        if vendor_source.exists() {
            command
                .arg("--extern")
                .arg(format!("vendor={}", vendor_library.display()));
        }
        let compiled = command
            .arg("main.rs")
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{case}/{edition}/{version}: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let output = Command::new(executable).output().unwrap();
        assert!(
            output.status.success(),
            "{case}/{edition}/{version}: execution failed"
        );
        assert!(
            output.stderr.is_empty(),
            "{case}/{edition}/{version}: stderr"
        );
        assert_source(&output.stdout, expected_stdout, version);
    }
}

#[rstest]
#[case::invalid_utf8("invalid_utf8", "error: invalid utf-8")]
#[case::broken_item("broken_item", "error: syntax error at 3:11:")]
#[case::broken_use("broken_use", "error: syntax error at 3:11:")]
#[case::broken_macro("broken_macro", "error: syntax error at 3:11:")]
#[case::broken_inner_header("broken_inner_header", "error: syntax error at 4:24:")]
fn input_errors(#[case] case: &str, #[case] diagnostic: &str) {
    let input = common::fixture(&format!("errors/{case}"), "input.rs");
    let output = execute(
        &["stdin", "--edition", "2024", "--style-edition", "2024"],
        &input,
    );
    assert_eq!(output.status.code(), Some(1), "{case}: {output:?}");
    assert!(output.stdout.is_empty(), "{case}: unexpected stdout");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with(diagnostic), "{case}: {stderr}");
}

#[rstest]
#[case::legacy_stdin(&["--stdin"])]
#[case::legacy_check(&["--check", "missing.rs"])]
#[case::legacy_write(&["--write", "missing.rs"])]
#[case::missing_mode(&[])]
#[case::unknown_argument(&["stdin", "--unknown"])]
#[case::invalid_edition(&["stdin", "--edition", "2025"])]
#[case::invalid_style_edition(&["stdin", "--style-edition", "2027"])]
#[case::missing_style_edition(&["stdin", "--style-edition"])]
#[case::missing_edition_value(&["stdin", "--edition"])]
#[case::edition_without_mode(&["--edition", "2021"])]
#[case::path_without_mode(&["missing.rs"])]
#[case::stdin_check(&["stdin", "check", "missing.rs"])]
#[case::stdin_write(&["stdin", "write", "missing.rs"])]
#[case::missing_check_file(&["check"])]
#[case::missing_write_file(&["write"])]
#[case::missing_files_input(&["files"])]
#[case::stdin_exclude(&["stdin", "--exclude", "*.rs"])]
#[case::invalid_exclude(&["write", "missing.rs", "--exclude", "[z-a]"])]
#[case::negated_exclude(&["files", "missing.rs", "--exclude", "!keep.rs"])]
#[case::check_invalid_edition(&["check", "missing.rs", "--edition", "2025"])]
#[case::write_missing_edition(&["write", "missing.rs", "--edition"])]
fn argument_errors(#[case] args: &[&str]) {
    let output = execute(args, b"\xff");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[rstest]
#[case::stdin(&["stdin"])]
#[case::check(&["check", "missing.rs"])]
#[case::write(&["write", "missing.rs"])]
#[case::files(&["files", "missing.rs"])]
fn modes_require_cargo_subcommand(#[case] args: &[&str]) {
    let output = cargo_bin_cmd!()
        .args(args)
        .write_stdin(b"\xff".to_vec())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("Usage: cargo ")
    );
}

#[rstest]
#[case(&["--help"])]
#[case(&["stdin", "--help"])]
#[case(&["check", "missing.rs", "--help"])]
#[case(&["write", "missing.rs", "--help"])]
fn help_does_not_read_source(#[case] args: &[&str]) {
    let output = execute(args, b"\xff");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("--edition"));
    assert!(help.contains("--style-edition"));
    assert!(help.contains("Usage: cargo modfmt "));
}

#[rstest]
#[case("2015")]
#[case("2018")]
#[case("2021")]
fn edition_selection(#[case] edition: &str) {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/cases/edition_keyword");
    check_source(
        &fs::read(directory.join("input.rs")).unwrap(),
        Some(&fs::read(directory.join("expected.rs")).unwrap()),
        "edition_keyword",
        edition,
        "2024",
    );
}

#[rstest]
#[case(&["stdin"])]
#[case(&["stdin", "--edition", "2024"])]
fn edition_2024_keyword_error(#[case] args: &[&str]) {
    let input = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/cases/edition_keyword/input.rs"),
    )
    .unwrap();
    let output = execute(args, &input);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[cfg(unix)]
#[rstest]
#[case::source_read_error(&["stdin"], 1)]
#[case::help(&["--help"], 0)]
#[case::missing_mode(&[], 2)]
#[case::invalid_edition(&["stdin", "--edition", "2025"], 2)]
fn unreadable_stdin(#[case] args: &[&str], #[case] expected_code: i32) {
    let directory = std::fs::File::open(env!("CARGO_MANIFEST_DIR")).unwrap();
    let output = std::process::Command::new(assert_cmd::cargo::cargo_bin!("cargo-modfmt"))
        .arg("modfmt")
        .args(args)
        .stdin(directory)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(expected_code));
    if expected_code == 0 {
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    } else {
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[rstest]
#[case("2015", "legacy.rs")]
#[case("2018", "legacy.rs")]
#[case("2021", "legacy.rs")]
fn legacy_style_edition_selection(#[case] style: &str, #[case] expected: &str) {
    check_source(
        &common::fixture("cases/style_order", "input.rs"),
        Some(&common::fixture("cases/style_order", expected)),
        "style_order",
        "2024",
        style,
    );
}
