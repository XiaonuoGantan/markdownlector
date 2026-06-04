//! End-to-end tests that run the built binary as a subprocess.
//!
//! No assertion crate is used: `std::process::Command` plus the
//! `CARGO_BIN_EXE_markdownlector` path cargo provides covers every case. Binary
//! fixtures are generated at runtime in `CARGO_TARGET_TMPDIR` rather than
//! committed, so their intent stays readable in source.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn run(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_markdownlector"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn markdownlector");
    // Write stdin and drop the handle so the child sees EOF before we wait. A
    // BrokenPipe here just means the child did not read stdin (e.g. --help) —
    // not a test failure.
    let mut stdin_pipe = child.stdin.take().expect("stdin handle");
    let _ = stdin_pipe.write_all(stdin);
    drop(stdin_pipe);
    child.wait_with_output().expect("wait for markdownlector")
}

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    path.push(name);
    std::fs::write(&path, bytes).expect("write temp fixture");
    path
}

#[test]
fn renders_file_to_plain_text() {
    let out = run(&[&fixture("basic.md")], b"");
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Markdownlector"));
    assert!(stdout.contains("A plain-text rendering of the docs."));
    assert!(stdout.contains("- read files"));
    assert!(stdout.contains("  - via a dash"));
    assert!(stdout.contains("1. first"));
    assert!(stdout.contains("# not a heading"));
    // Inline markup and link targets are stripped.
    assert!(!stdout.contains("**"));
    assert!(!stdout.contains("https://example.com"));
}

#[test]
fn reads_from_stdin_with_dash() {
    let piped = std::fs::read(fixture("stdin.md")).unwrap();
    let out = run(&["-"], &piped);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, "Stdin\n\nPiped content with no trailing newline.\n");
    // Exactly one trailing newline, even though the input had none.
    assert!(stdout.ends_with('\n'));
    assert!(!stdout.ends_with("\n\n"));
}

#[test]
fn file_and_stdin_produce_identical_output() {
    let from_file = run(&[&fixture("basic.md")], b"");
    let piped = std::fs::read(fixture("basic.md")).unwrap();
    let from_stdin = run(&["-"], &piped);
    assert_eq!(from_file.stdout, from_stdin.stdout);
}

#[test]
fn empty_file_produces_empty_output() {
    let out = run(&[&fixture("empty.md")], b"");
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
}

#[test]
fn missing_file_exits_nonzero_with_stderr() {
    let out = run(&[&fixture("does-not-exist.md")], b"");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("cannot read"));
    assert!(out.stdout.is_empty());
}

#[test]
fn invalid_utf8_exits_nonzero_with_readable_error() {
    // Invalid UTF-8 with no NUL: exercises the UTF-8 path in isolation.
    let path = temp_file("invalid_utf8.bin", &[0xFF, 0xFE]);
    let out = run(&[path.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("not valid UTF-8"));
}

#[test]
fn binary_input_exits_nonzero_with_readable_error() {
    // Contains a NUL: exercises the binary path.
    let path = temp_file("binary.bin", b"ok\x00bad");
    let out = run(&[path.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("binary"));
}

#[test]
fn help_goes_to_stdout_and_exits_zero() {
    for flag in ["--help", "-h"] {
        let out = run(&[flag], b"");
        assert_eq!(out.status.code(), Some(0));
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(stdout.contains("reading appliance for Markdown"));
        assert!(stdout.contains("Usage:"));
    }
}

#[test]
fn version_goes_to_stdout_and_exits_zero() {
    for flag in ["--version", "-V"] {
        let out = run(&[flag], b"");
        assert_eq!(out.status.code(), Some(0));
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
        assert!(stdout.contains("markdownlector"));
    }
}

#[test]
fn too_many_arguments_exits_nonzero() {
    let out = run(&[&fixture("basic.md"), &fixture("empty.md")], b"");
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8(out.stderr).unwrap().contains("too many"));
}

#[test]
fn strips_utf8_bom_so_heading_still_parses() {
    // A leading BOM must be removed at the boundary, or it prints as mojibake
    // and glues to the '#' so the heading marker is never stripped.
    let out = run(&["-"], "\u{feff}# Title".as_bytes());
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "Title\n");
}

#[test]
fn crlf_input_renders_with_unix_newlines() {
    let out = run(&["-"], b"# Title\r\n\r\nbody");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8(out.stdout).unwrap(), "Title\n\nbody\n");
}

#[test]
fn file_and_stdin_parity_without_trailing_newline() {
    // basic.md ends in a newline, so it cannot expose a trailing-byte
    // divergence; stdin.md (no trailing newline) can.
    let from_file = run(&[&fixture("stdin.md")], b"");
    let piped = std::fs::read(fixture("stdin.md")).unwrap();
    let from_stdin = run(&["-"], &piped);
    assert_eq!(from_file.status.code(), Some(0));
    assert_eq!(from_file.stdout, from_stdin.stdout);
}
