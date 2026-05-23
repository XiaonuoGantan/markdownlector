# Milestone 03: Terminal Reader

## Goal

Implement the primary full-screen reading experience: alternate screen, viewport rendering, vim-based navigation, search, heading jumps, help overlay, status line, and resize support.

This is the first milestone where Markdownlector feels like the product described in the high-level design.

## User-Visible Behavior

```text
markdownlector README.md
markdownlector -
```

When stdout is a TTY, the app opens a full-screen reader. When stdout is not a TTY, it remains plain text only.

## Scope

Implement:

- `crossterm` dependency with documented justification.
- Raw mode and alternate screen lifecycle.
- Reader state machine.
- Viewport scrolling.
- Visible-line-only rendering.
- Built-in adaptive terminal style.
- Fixed vim keymap.
- Search.
- Heading jumps.
- Help overlay on `?`.
- Bottom status line.
- Terminal resize relayout.

Do not implement:

- File picker.
- Custom themes.
- Custom keymaps.
- Mouse support.
- Syntax highlighting.
- File watching.

## Suggested File Layout

```text
src/
  app.rs
  terminal.rs
  render.rs
  keymap.rs
  search.rs
  theme.rs
tests/
  keymap.rs
  search.rs
  render_snapshot.rs
```

## Implementation Plan

1. Add terminal abstraction.
   - Wrap raw mode, alternate screen enter/leave, cursor visibility, clear, and flush.
   - Ensure cleanup runs on normal exit and errors.
   - Keep direct `crossterm` usage inside `terminal.rs` and `render.rs`.

2. Implement reader state.
   - Current document.
   - Layout lines.
   - Viewport top line.
   - Terminal size.
   - Search query and selected match.
   - Help overlay state.
   - Status message state if needed.

3. Implement fixed key handling.
   - `j` / `k`: line down/up.
   - `Ctrl-d` / `Ctrl-u`: half-page down/up.
   - `Ctrl-f` / `Ctrl-b`: page down/up.
   - `gg` / `G`: top/bottom.
   - `/`: search prompt.
   - `n` / `N`: next/previous match.
   - `]]` / `[[`: next/previous heading.
   - `?`: help overlay.
   - `q`: quit.
   - `Esc`: clear prompt/help/search transient state.

4. Implement viewport rendering.
   - Render only visible layout lines plus status line.
   - Do not rebuild Markdown or layout on scroll.
   - Use a line cache from Milestone 02.
   - Clear only what is necessary for correctness.

5. Implement theme.
   - Follow the editorial restraint defined in `DESIGN.md`: no saturated brand accent, no rainbow syntax tones, no developer-tools neon. The reading surface should feel like quiet print, not a debugger.
   - Style-role to terminal-attribute mapping (no hard-coded hex; map roles to terminal attributes that respect user theme):
     - `Body` → default foreground (terminal "ink").
     - `Muted` → dim attribute (the analogue of `body`/`muted` in `DESIGN.md`).
     - `MutedSoft` → dim + faint, used for hairline-style separators and least-important status text.
     - `Heading1`–`Heading6` → unstyled or single attribute (e.g. underline for H1, plain for the rest). Do not bold every heading. `DESIGN.md` keeps display weight at 300 deliberately; the terminal analogue is to avoid bold-everywhere and let spacing carry hierarchy.
     - `Emphasis` → italic. `Strong` → bold (this is the only place bold belongs).
     - `Code` → distinct but quiet (faint background or alternate foreground if the terminal supports it).
     - `Error` and `Success` → reserved for the status line; never used to color body content.
   - Built-in dark/light adaptive palette if terminal background detection is simple and reliable.
   - Otherwise default to a readable palette that works against both common backgrounds and leave adaptive detection for polish.
   - Provide 256-color fallback only if it does not complicate the renderer.

6. Implement status line.
   - Show path or stdin label.
   - Show current heading when available.
   - Show scroll percent.
   - Show search query or match count while searching.
   - Keep it one line.

7. Implement resize handling.
   - On resize, recompute layout for the new body width.
   - Preserve approximate reading position using current top line or nearest block.

## Tests

Unit tests:

- Key sequences map to expected actions.
- Search finds plain-text matches across layout lines.
- Heading navigation moves to correct line.
- Viewport clamps at top and bottom.
- Resize recomputes layout without panicking.

Render tests:

- Render a small viewport to a string buffer and compare expected ANSI structure.
- Verify no ANSI escapes are produced by non-TTY plain output.

Manual terminal tests:

- Open a long README and scroll with all navigation keys.
- Search with `/`, `n`, and `N`.
- Jump headings with `]]` and `[[`.
- Resize terminal and confirm layout remains readable.
- Press `?` and confirm help overlay appears.

## Verification

Run:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Manual:

```text
cargo run -- README.md
printf '# Title\n\nBody\n' | cargo run -- -
```

Expected result: TTY opens the reader; piped non-TTY output remains plain text only.

## Readability and Maintainability Rules

- Keep terminal side effects at the edge.
- Keep app state explicit and serializable enough to inspect in tests.
- Keep rendering deterministic from state plus terminal size.
- Avoid clever diff rendering until full repaint is demonstrably too slow.
- Any terminal cleanup logic should be boring and easy to audit.

## Done Criteria

- Full-screen reader works for file and stdin.
- Fixed vim keymap works.
- Search, heading jumps, help, and status line work.
- Resize relayout works.
- Tests cover state transitions and search/navigation behavior.
