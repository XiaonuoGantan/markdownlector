# Milestone 03b: Reader Navigation, Search, and Theme

## Goal

Layer the power-user features onto the proven reader core: the complete vim keymap, document search with stable match positions, heading jumps, the help overlay, and adaptive theming. Because Milestone 03a already shook out the render loop, resize, and the layout contract, these features build on solid ground — and search and heading positions rely on the IR anchors locked down in Milestone 02.

## User-Visible Behavior

The full reader keymap from the high-level design is now live:

```text
j / k       line down / up
Ctrl-d/u    half-page down / up
Ctrl-f/b    page down / up
gg / G      top / bottom
/           forward search
n / N       next / previous match
]] / [[     next / previous heading
?           help overlay
q           quit
Esc         clear search/help/prompt
```

## Scope

Implement:

- The remaining vim keys: `Ctrl-f` / `Ctrl-b`, `]]` / `[[`, `/`, `n` / `N`, `?`, `Esc`.
- Search over layout-line plain text, resolving matches to IR anchors (stable across resize).
- Heading jumps using the Milestone 02 heading index.
- Help overlay on `?`, exactly matching the implemented keymap.
- Search state in the status line (query, match count / position) and the current heading.
- Adaptive dark/light theme selection from terminal background where detection is simple and reliable; otherwise keep the 03a fixed default.

Do not implement:

- Custom themes or keymaps, mouse, syntax highlighting, file watching.

## Suggested File Layout

```text
src/
  keymap.rs        # extended from 03a
  search.rs
  theme.rs         # adaptive detection added here
  app.rs           # extended state: search, help overlay
tests/
  keymap.rs
  search.rs
  heading_nav.rs
```

## Implementation Plan

1. Extend the keymap.
   - Add the remaining `KeyEvent -> Action` mappings; keep the function pure and exhaustively tested.

2. Implement search.
   - Search the plain text carried on each `LayoutLine`.
   - Resolve each match to its IR anchor so positions survive resize / relayout (the anchor model is from Milestone 02).
   - `/` opens the prompt; `n` / `N` cycle matches; `Esc` clears.

3. Implement heading jumps.
   - `]]` / `[[` move to the next / previous heading via the heading index.

4. Implement the help overlay.
   - `?` toggles an overlay that exactly matches the implemented keymap. Keep it quiet and legible.

5. Extend the status line.
   - Show the current heading when available, the search query, and the match count while searching. Still one line.

6. Implement adaptive theme (the only genuinely deferred 03a item).
   - Detect terminal background where it is simple and reliable; map style roles to terminal attributes following the `DESIGN.md` restraint already encoded in 03a's theme:
     - `Body` -> default foreground (terminal "ink").
     - `Muted` -> dim attribute.
     - `MutedSoft` -> dim + faint, for hairline-style separators and least-important status text.
     - `Heading1`-`Heading6` -> unstyled or a single attribute (e.g. underline for H1, plain for the rest). Do not bold every heading; let spacing carry hierarchy.
     - `Emphasis` -> italic. `Strong` -> bold (the only place bold belongs).
     - `Code` -> distinct but quiet (faint background or alternate foreground if supported).
     - `Error` / `Success` -> reserved for the status line; never used to color body content.
   - If detection is unreliable on a terminal, fall back to the fixed default. Provide a 256-color fallback only if it does not complicate the renderer.

## Tests

Unit tests:

- Full `KeyEvent -> Action` table, including the keys added here.
- Search finds matches across layout lines and resolves them to correct anchors.
- A match's anchor still points at the same content after a resize (exercises the Milestone 02 anchor guarantee end-to-end).
- Heading navigation moves to the correct line.
- Help overlay content matches the keymap (a test asserting the documented keys equal the handled keys).

Manual terminal tests:

- Search with `/`, `n`, `N`; resize mid-search and confirm the current match stays put.
- Jump headings with `]]` / `[[`.
- Press `?`; confirm the overlay matches reality.
- Confirm theming is legible on a dark and a light terminal.

## Verification

```text
make ci
cargo run -- README.md
```

## Readability and Maintainability Rules

- Keep `KeyEvent -> Action` pure; never read events inside the mapping.
- Search positions are anchors, never raw layout-line numbers.
- The help overlay is generated from, or tested against, the real keymap so the two cannot drift.
- Keep theme role mapping semantic and restrained per `DESIGN.md`.

## Done Criteria

- The full vim keymap from the high-level design works.
- Search works, and match positions are stable across resize.
- Heading jumps and the help overlay work; help matches the keymap.
- Adaptive theme works where reliable, with a clean fixed fallback.
- Tests cover the full keymap, search / anchor stability, and heading navigation.
