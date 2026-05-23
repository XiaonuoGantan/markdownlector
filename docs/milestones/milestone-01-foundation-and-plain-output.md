# Milestone 01: Foundation and Plain Output

## Goal

Create the smallest usable Rust CLI foundation for Markdownlector: local file/stdin input, Unix-only targeting, plain text output for non-TTY usage, and a dependency policy that keeps the project auditable from day one.

This milestone deliberately avoids a full-screen TUI. It proves the project can read input, parse basic arguments, produce deterministic text-only output, and stay simple.

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
- Plain text rendering path that strips Markdown syntax enough to be useful in pipelines.
- `DEPENDENCIES.md` with justification for every runtime dependency.
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
src/
  main.rs
  input.rs
  markdown.rs
  plain.rs
tests/
  fixtures/
    basic.md
    stdin.md
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

4. Implement input loading.
   - `InputSource::File(PathBuf)`.
   - `InputSource::Stdin`.
   - Preserve bytes as UTF-8 text; invalid UTF-8 should return a clear error.

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

## Tests

Unit tests:

- CLI parser accepts file, stdin, help, and version.
- CLI parser rejects too many arguments.
- Input loader reads fixture file.
- Plain renderer strips common Markdown markers via parser events.
- Plain renderer preserves fenced code content.

Integration tests:

- `markdownlector tests/fixtures/basic.md` with captured stdout.
- `markdownlector -` with piped fixture content.
- Missing file exits non-zero and writes stderr.

## Verification

Run:

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
- Tests cover CLI parsing, input loading, and plain output.
- No TUI or styling code exists yet.
- Dependency list contains only justified foundation dependencies.
