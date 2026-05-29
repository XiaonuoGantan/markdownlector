# Milestone 01: Foundation, Plain Output, and CI

## Goal

Create the smallest usable Rust CLI foundation for Markdownlector: local file/stdin input, Unix-only targeting, plain text output for non-TTY usage, clean handling of the input boundary (invalid UTF-8, empty, and binary files), and a dependency policy that keeps the project auditable from day one.

A continuous-integration gate lands here too. The same `cargo fmt --check` / `cargo test` / `cargo clippy -- -D warnings` block is repeated in every later milestone; this milestone makes a machine enforce it instead of trusting memory.

This milestone deliberately avoids a full-screen TUI. It proves the project can read input, parse basic arguments, produce deterministic text-only output, fail cleanly at the input boundary, and stay simple.

## User-Visible Behavior

```text
markdownlector README.md
markdownlector -
```

When stdout is not a TTY, output is plain text only. No ANSI styling is emitted.

## Scope

Implement:

- `Cargo.toml` with AGPL-3.0 package metadata.
- `pulldown-cmark` dependency for parser-based plain text output.
- Basic `src/main.rs` entrypoint.
- Manual argument parsing for:
  - `markdownlector <file>`
  - `markdownlector -`
  - `markdownlector --help`
  - `markdownlector --version`
- Unix-only compile guard.
- Input loading from file and stdin.
- Input boundary handling: invalid UTF-8, empty input, and obviously-binary files each produce a clear error (or, for empty input, clean empty output) and never panic.
- Plain text rendering path that strips Markdown syntax enough to be useful in pipelines.
- `DEPENDENCIES.md` with justification for every runtime dependency.
- A `make ci` target wrapping the verification gate.
- A minimal CI workflow (`.github/workflows/ci.yml`) that runs `make ci` on push and pull request.
- Initial test fixtures.

Do not implement:

- Full-screen terminal UI.
- File picker.
- Directory discovery.
- ANSI styling.
- Custom config.
- Remote URL input.

## Suggested File Layout

```text
Cargo.toml
DEPENDENCIES.md
Makefile
.github/
  workflows/
    ci.yml
src/
  main.rs
  input.rs
  markdown.rs
  plain.rs
tests/
  fixtures/
    basic.md
    stdin.md
    empty.md
    invalid_utf8.bin
  cli_plain_output.rs
```

## Implementation Plan

1. Scaffold the Rust package.
   - Set package license to `AGPL-3.0-only`.
   - Use Rust edition 2024.
   - Add only `pulldown-cmark` as the first runtime dependency.
   - Document `pulldown-cmark` in `DEPENDENCIES.md`.

2. Add Unix-only target guard.
   - Use `#[cfg(not(unix))] compile_error!(...)`.
   - Keep the message direct: Windows is not supported.

3. Implement manual CLI parsing.
   - Keep parsing small enough that `clap` is unnecessary.
   - Return structured command enum values from a tiny parser function.
   - Unit-test the parser directly.

4. Implement input loading and validate the input boundary.
   - `InputSource::File(PathBuf)`.
   - `InputSource::Stdin`.
   - Preserve bytes as UTF-8 text; invalid UTF-8 returns a clear error, never a panic.
   - Empty input is valid and renders to empty output.
   - Detect obviously-binary input (for example, a NUL byte in the first chunk) and refuse it with a readable error and a non-zero exit, rather than dumping control bytes into the terminal.

5. Implement plain text output.
   - Start with a conservative parser-based Markdown-to-text pass.
   - Preserve paragraph text.
   - Strip heading markers while preserving heading text.
   - Preserve code block content without styling.
   - Preserve list text in readable plain form.
   - Do not attempt full Markdown fidelity here; this path exists for text-only display.

6. Add error handling.
   - Human-readable errors to stderr.
   - Non-zero exit code on input or parse errors.
   - No panic for user input mistakes.

7. Add the CI gate.
   - Write a `make ci` target that runs `cargo fmt --check`, `cargo test`, and `cargo clippy -- -D warnings` in sequence. This is the shell-of-record, so the gate is reproducible locally byte-for-byte.
   - Add `.github/workflows/ci.yml` that installs the toolchain and runs `make ci` on push and pull request.
   - Keep it thin: no coverage gate, no build matrix, no caching cleverness yet. The forge wrapper stays small precisely so the project is not locked to one CI provider.

## Tests

Unit tests:

- CLI parser accepts file, stdin, help, and version.
- CLI parser rejects too many arguments.
- Input loader reads fixture file.
- Invalid UTF-8 input returns an error rather than panicking.
- Empty input produces empty output.
- Binary input (a NUL byte) is refused with a non-zero exit.
- Plain renderer strips common Markdown markers via parser events.
- Plain renderer preserves fenced code content.

Integration tests:

- `markdownlector tests/fixtures/basic.md` with captured stdout.
- `markdownlector -` with piped fixture content.
- Missing file exits non-zero and writes stderr.
- `tests/fixtures/invalid_utf8.bin` exits non-zero with a readable error.

## Verification

Run:

```text
make ci
```

which runs:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Manual check:

```text
printf '# Title\n\nHello **world**.\n' | cargo run -- -
```

Expected result: readable plain text without ANSI escape sequences.

## Readability and Maintainability Rules

- Keep modules small and named by responsibility.
- Prefer explicit enums over stringly typed mode handling.
- Do not add an abstraction until two call sites need it.
- Add comments only where control flow or terminal behavior is non-obvious.
- Every added dependency must be justified in `DEPENDENCIES.md`.

## Done Criteria

- The binary builds on Unix-like systems.
- Text-only pipeline output works.
- The input boundary (invalid UTF-8, empty, binary) is handled without panic.
- `make ci` passes locally, and the CI workflow runs the same gate on push and pull request.
- Tests cover CLI parsing, input loading, the input boundary, and plain output.
- No TUI or styling code exists yet.
- Dependency list contains only justified foundation dependencies.
