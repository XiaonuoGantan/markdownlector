# Milestone 03a: Reader Core

## Goal

Build the proven, panic-safe core of the full-screen reader: terminal lifecycle, a render path that draws only the visible viewport, scrolling, resize relayout, a minimal vim motion set, and the status line. This is the milestone where the central architectural rule — keep Markdown semantics until layout time, relayout cheaply on resize, render only the visible region — is actually demonstrated.

It is the first milestone a human can *touch*: open a file, fill the screen, scroll, resize, quit. Navigation power features (the full keymap, search, heading jumps, help, adaptive theme) are layered on top in Milestone 03b, against a core that has already been shaken out.

## User-Visible Behavior

```text
markdownlector README.md
markdownlector -
```

When stdout is a TTY, the app opens a full-screen reader you can scroll and resize. When stdout is not a TTY, it remains plain text only (Milestone 01 behavior).

Available keys in 03a: `j` / `k`, `Ctrl-d` / `Ctrl-u`, `gg` / `G`, `q`. The rest of the keymap arrives in 03b.

## Scope

Implement:

- `crossterm` dependency (if not already added by the Milestone 02b spike), with documented justification.
- Raw mode and alternate screen lifecycle.
- Panic-safe teardown (see the dedicated section below).
- A render path that targets an `impl Write` buffer, not `Stdout` directly (the keystone test seam).
- Reader state machine: current document, layout lines, viewport top line, terminal size.
- Viewport scrolling.
- Visible-line-only rendering, with the no-full-repaint-per-keypress invariant asserted (see below).
- A minimal vim motion set: `j` / `k`, `Ctrl-d` / `Ctrl-u`, `gg` / `G`, `q`.
- Bottom status line: path or stdin label, scroll percent.
- Terminal resize relayout, preserving approximate reading position.
- A fixed, readable default style (one palette that works on common dark and light backgrounds). Adaptive detection is deliberately deferred to 03b.

Do not implement:

- Full keymap (`Ctrl-f` / `Ctrl-b`, `]]` / `[[`), search, heading jumps, help overlay — all 03b.
- Adaptive theme detection — 03b.
- File picker, custom themes, custom keymaps, mouse, syntax highlighting, file watching.

## Suggested File Layout

```text
src/
  app.rs
  terminal.rs
  render.rs
  keymap.rs        # minimal motion set in 03a; grows in 03b
  theme.rs         # fixed default in 03a; adaptive in 03b
tests/
  reader_state.rs
  render_snapshot.rs
  teardown.rs
```

## Panic-Safe Teardown (highest-risk surface)

Leaving the user's terminal in raw mode or on the alternate screen after a crash is the scariest defect in this project: the symptom is "my terminal is wrecked, I have to run `reset`," and it happens on users' machines, not in snapshot tests. Engineer it out rather than relying on catching it:

- Terminal restoration must be **RAII / `Drop`-based**, not a cleanup call at the end of `main`. A `Drop` guard runs during panic unwinding; a trailing teardown line does not.
- Install a **panic hook** that restores the terminal (leave alternate screen, disable raw mode) before the panic message prints.
- On a Unix-only appliance, **SIGTERM / SIGHUP** should trigger the same restore path. Decide and document SIGINT (Ctrl-C) behavior explicitly.
- Abstract the terminal behind a small trait so the guard's logic can be unit-tested without a real TTY.

## Implementation Plan

1. Land the render seam first.
   - `render.rs` writes to `impl Write`, taking `(lines, top, height, width)`.
   - This single decision is what makes the reader unit-testable: tests pump a `Vec<u8>` and assert the exact bytes / escape structure.

2. Add the terminal abstraction.
   - Wrap raw mode, alternate screen enter/leave, cursor visibility, clear, and flush.
   - Build the RAII `Drop` guard and panic hook here.
   - Keep direct `crossterm` usage inside `terminal.rs` and `render.rs`.

3. Implement reader state.
   - Current document, layout lines, viewport top line, terminal size.
   - Keep state explicit and inspectable in tests.

4. Implement the minimal keymap as a pure function.
   - `KeyEvent -> Action` with no terminal in the loop, tested exhaustively.
   - `app.rs` applies `Action -> state` (also pure).
   - 03a actions: line up/down, half-page up/down, top, bottom, quit.

5. Implement viewport rendering.
   - Render only visible layout lines plus the status line.
   - Do not rebuild Markdown or layout on scroll; reuse the Milestone 02 line cache.
   - Clear only what is necessary for correctness.

6. Implement the status line (minimal).
   - Path or stdin label; scroll percent. One line.

7. Implement resize handling.
   - Recompute layout for the new body width.
   - Preserve approximate reading position via the current top line's IR anchor (from Milestone 02).

8. Apply a fixed default style.
   - Follow `DESIGN.md` restraint: no saturated accent, no neon, spacing over color. Bold reserved for `Strong`; do not bold every heading. Reserve color for `Error` / `Success` in the status line only.

## The Perf Invariant (pulled forward from release)

"No full-document repaint per keypress" is an architectural property of the render loop built here, not a release-time polish task. Finding it violated later is a redesign, not a tune-up. So assert it where it is born:

- Instrument the render path with a counter (lines drawn) or a "full-repaint" flag.
- Unit test: a `j` / `k` keypress in a large (5,000-line) fixture touches O(viewport) lines, not O(document). The fixture may be generated inline or committed as `tests/fixtures/large.md` (shared with the Milestone 05 wall-clock benchmark).
- This is a correctness test wearing a perf costume. The noisy wall-clock "<50 ms cold open" benchmark stays in Milestone 05; this structural assertion lives here.

## Tests

Unit tests:

- `KeyEvent -> Action` table for the 03a motion set.
- `Action -> state` viewport math: scroll clamps at top and bottom.
- Resize recomputes layout without panicking and keeps position via anchor.
- Render to a `Vec<u8>` buffer matches expected structure for a small viewport.
- The O(viewport)-not-O(document) repaint assertion.
- Teardown guard logic restores state (via the terminal trait), including on a forced panic.

Manual terminal tests:

- Open a long README; scroll with `j` / `k`, `Ctrl-d` / `Ctrl-u`, `gg`, `G`.
- Resize the terminal and confirm the layout stays readable and the position is roughly preserved.
- Force a panic in a debug build and confirm the terminal is restored.
- Confirm piped (non-TTY) output is still plain text.

## Verification

```text
make ci
cargo run -- README.md
printf '# Title\n\nBody\n' | cargo run -- -
```

Expected result: TTY opens a scrollable, resizable reader; piped output stays plain text.

## Readability and Maintainability Rules

- Keep terminal side effects at the edge (`terminal.rs`, `render.rs`).
- Render must be deterministic from state plus terminal size.
- Keep `KeyEvent -> Action` and `Action -> state` pure and separately tested.
- Avoid clever diff rendering until full repaint is demonstrably too slow — but do keep visible-only rendering, which is the invariant above, not an optimization.
- Terminal teardown logic should be boring and easy to audit.

## Done Criteria

- The full-screen reader opens for file and stdin and renders the visible viewport only.
- Minimal vim motions and quit work.
- Resize relayout works and preserves approximate position.
- Panic and signal teardown restore the terminal (RAII guard + panic hook), with a test for the guard logic.
- The no-full-repaint-per-keypress invariant is asserted in tests.
- Rendering targets `impl Write` and is snapshot-tested.
