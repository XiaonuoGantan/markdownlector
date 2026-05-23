# Milestone 05: Polish, Performance, and Release Readiness

## Goal

Harden Markdownlector into a coherent v1 candidate: readable default presentation, documented terminal recommendations, performance checks, packaging basics, and maintainability guardrails.

This milestone should not add broad new product features. It makes the existing scope reliable, fast, and explainable.

## User-Visible Behavior

Markdownlector should feel ready for daily use on Unix-like terminals:

- Reader view is legible in common terminal fonts.
- Ghostty recommendation is documented.
- Large Markdown files remain responsive.
- Error messages are clear.
- Help text matches the implemented keymap.
- Non-TTY output remains plain text only.

## Scope

Implement:

- Visual polish for reader and picker.
- Ghostty profile recommendation documentation.
- Fixture-based performance checks.
- Renderer snapshots for key Markdown elements.
- Error message pass.
- Release documentation.
- Dependency audit checklist.

Do not implement:

- Custom themes.
- Custom keymaps.
- Config files.
- Syntax highlighting.
- Mouse support.
- Windows support.
- Remote fetching.

## Suggested File Layout

```text
docs/
  ghostty-profile.md
  release-checklist.md
DEPENDENCIES.md
tests/
  performance_smoke.rs
  render_snapshots.rs
fixtures/
  large.md
```

## Implementation Plan

1. Polish visual hierarchy.
   - Tune heading spacing.
   - Tune paragraph wrapping and margins.
   - Tune code block framing.
   - Tune blockquote left rule.
   - Tune table fallback.
   - Keep colors restrained and readable.

2. Finish help and status text.
   - Help overlay exactly matches fixed keymap.
   - Status line remains one line.
   - Search status is understandable.
   - Empty picker status is concise.

3. Document terminal recommendations.
   - Add Ghostty profile guidance.
   - Recommend font and line-height settings without requiring them.
   - Explain that the app cannot set terminal fonts.

4. Add performance smoke tests.
   - Use a committed large Markdown fixture or generated fixture inside the test.
   - Measure parse + layout time separately from terminal drawing where practical.
   - Keep thresholds loose enough for CI variability but strict enough to catch accidental full re-render loops.

5. Add render snapshots.
   - Headings.
   - Paragraph wrapping.
   - Lists.
   - Code blocks.
   - Blockquotes.
   - Tables.
   - Search highlight if implemented through renderer.

6. Audit dependencies.
   - Confirm every dependency is listed in `DEPENDENCIES.md`.
   - Confirm no optional feature pulled in large unnecessary transitive dependencies without reason.
   - Run `cargo tree` and commit the findings summary if useful.

7. Prepare release documentation.
   - README with scope, install/build instructions, keymap, and non-goals.
   - Release checklist.
   - License notice.

## Tests

Automated:

- Existing unit and integration tests.
- Performance smoke test for a large Markdown fixture.
- Renderer snapshots.
- Plain output test confirming no ANSI escapes in non-TTY mode.

Manual:

- Open a long README.
- Use every documented key.
- Resize terminal repeatedly.
- Verify Ghostty and a second Unix terminal both render acceptably.
- Pipe output to `cat`, `grep`, and a file to confirm plain text behavior.

## Verification

Run:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
cargo tree
```

Manual:

```text
cargo run -- .
cargo run -- README.md
printf '# Title\n\nBody\n' | cargo run -- - > /tmp/markdownlector.txt
```

Expected result: the redirected file contains no ANSI escape sequences.

## Readability and Maintainability Rules

- Polish must not introduce configuration surfaces.
- Performance work should start with measurement, not speculative complexity.
- Snapshot changes require human-readable explanation.
- Keep release docs factual and explicit about non-goals.
- Do not add dependencies for cosmetic improvements unless the tradeoff is documented.

## Done Criteria

- V1 scope is implemented and documented.
- Reader and picker are visually coherent.
- Performance smoke tests pass.
- Dependency policy is current.
- Release checklist exists.
- No out-of-scope features slipped into v1.
