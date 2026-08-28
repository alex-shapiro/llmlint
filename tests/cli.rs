//! End-to-end tests for the binary: exit status and output shape.

use std::process::{Command, Output};

fn llmlint(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_llmlint"))
        .args(args)
        .output()
        .expect("failed to run llmlint")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// 0 clean, 1 cliches found, 2 error — the clippy and ruff convention.
#[test]
fn exit_status_is_zero_if_clean() {
    let out = llmlint(&["tests/fixtures/clean.md"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(stdout(&out).contains("All clear"));
}

#[test]
fn exit_status_is_one_if_cliches_found() {
    let out = llmlint(&["tests/fixtures/example.txt"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).contains("Found 38 cliches in 1 file."));
}

#[test]
fn exit_status_is_two_on_usage_error() {
    assert_eq!(llmlint(&[]).status.code(), Some(2), "no paths");
    assert_eq!(
        llmlint(&["tests/fixtures/clean.md", "--select", "nope"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        llmlint(&["tests/fixtures/clean.md", "--output-format", "pretty"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        llmlint(&["no/such/file.md"]).status.code(),
        Some(2),
        "missing file"
    );
}

#[test]
fn concise_is_path_line_column_id_message() {
    let out = llmlint(&["tests/fixtures/example.txt", "-q"]);
    let first = stdout(&out).lines().next().unwrap().to_owned();
    assert_eq!(
        first,
        "tests/fixtures/example.txt:1:43: no-chain “No X, no Y” chains"
    );
}

#[test]
fn full_underlines_the_match() {
    let out = llmlint(&[
        "tests/fixtures/example.txt",
        "-q",
        "--output-format",
        "full",
    ]);
    let text = stdout(&out);
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some("warning: “No X, no Y” chains"));
    assert_eq!(lines.next(), Some(" --> tests/fixtures/example.txt:1:43"));
    let caret = text.lines().find(|l| l.contains('^')).unwrap();
    assert_eq!(caret, format!("  | {}{}", " ".repeat(42), "^".repeat(36)));
}

#[test]
fn select_narrows_and_ignore_subtracts() {
    let selected = llmlint(&["tests/fixtures/example.txt", "-q", "--select", "no-chain"]);
    assert_eq!(stdout(&selected).lines().count(), 1);

    let ignored = llmlint(&["tests/fixtures/example.txt", "-q", "--ignore", "no-chain"]);
    assert_eq!(stdout(&ignored).lines().count(), 37);
}

#[test]
fn quiet_suppresses_only_the_summary() {
    let loud = llmlint(&["tests/fixtures/example.txt"]);
    let quiet = llmlint(&["tests/fixtures/example.txt", "-q"]);
    assert_eq!(
        stdout(&loud).lines().count(),
        stdout(&quiet).lines().count() + 1
    );
}
