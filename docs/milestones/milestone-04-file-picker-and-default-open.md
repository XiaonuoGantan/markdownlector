# Milestone 04: File Picker and Default Open Behavior

## Goal

Implement Markdownlector's directory behavior: current-directory-scoped Markdown discovery, no-argument defaults, and a simple file picker.

This milestone completes the main product loop:

```text
file picker -> reader -> file picker
```

## User-Visible Behavior

```text
markdownlector
markdownlector .
markdownlector docs/
```

Rules:

- No args opens an obvious `README.md` in the current directory.
- If multiple Markdown files are present, show a simple file picker.
- If exactly one Markdown file exists and no README exists, open it directly.
- If no Markdown file exists, show an empty picker with a concise status message.
- Discovery is non-recursive and scoped to the current directory or supplied directory only.

## Scope

Implement:

- Directory source handling.
- Markdown file discovery.
- README ranking.
- Direct-open decision logic.
- File picker state and rendering.
- Picker filter via `/`.
- Picker navigation.
- Reader-to-picker return via `h`.

Do not implement:

- Git repository root discovery.
- Recursive search outside the supplied directory scope.
- Fuzzy matching.
- File watching.
- Remote sources.
- Editing.

## Suggested File Layout

```text
src/
  discover.rs
  picker.rs
  app.rs
tests/
  discover.rs
  default_open.rs
  picker_state.rs
tests/
  fixtures/
    dirs/
      one-readme/
      multiple-md/
      empty/
```

## Implementation Plan

1. Add source mode selection.
   - File path -> reader.
   - `-` -> stdin reader.
   - Directory path -> discovery/default decision.
   - No args -> discovery/default decision in current directory.

2. Implement discovery.
   - Scope to the current or supplied directory.
   - Do not recurse into child directories.
   - Include common Markdown extensions: `.md`, `.markdown`, `.mdown`, `.mkdn`, `.mkd`.
   - Ignore hidden files only if this keeps behavior simple; otherwise include them and document it.

3. Implement ranking.
   - README names rank first, case-insensitive.
   - Then alphabetical path order.
   - Keep ranking deterministic.

4. Implement default-open decision.
   - One obvious README -> open reader.
   - One Markdown file and no README -> open reader.
   - Multiple Markdown files -> picker.
   - None -> empty picker.

5. Implement picker UI.
   - Minimal header/status.
   - List visible files.
   - Highlight selected row.
   - Show count.
   - Show filter prompt when active.

6. Implement picker keymap.
   - `j` / `k`: selection down/up.
   - `Ctrl-d` / `Ctrl-u`: half-page down/up.
   - `gg` / `G`: first/last.
   - `/`: filter.
   - `Enter` / `l`: open selected file.
   - `q`: quit.
   - `Esc`: clear filter.

7. Integrate reader return.
   - `h` returns from reader to picker only when the document was opened from picker.
   - If a file was opened directly from CLI, `h` should either do nothing or show help status. Prefer doing nothing in v1.

## Tests

Unit tests:

- Discovery returns deterministic Markdown files.
- README ranking is case-insensitive.
- Default-open decision handles README, single file, multiple files, and empty directory.
- Picker filter is simple case-insensitive substring matching.
- Picker navigation clamps correctly.

Integration tests:

- Fixture directory with only README opens reader mode.
- Fixture directory with multiple Markdown files starts picker mode.
- Empty fixture directory starts empty picker state.

Manual terminal tests:

- `cargo run --`
- `cargo run -- .`
- `cargo run -- tests/fixtures/dirs/multiple-md`
- Filter with `/`, open with `Enter`, return with `h`.

## Verification

Run:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```

Manual:

```text
cargo run -- .
```

Expected result: default-open behavior follows the documented rules.

## Readability and Maintainability Rules

- Keep discovery pure and testable.
- Keep direct-open rules in one function with exhaustive tests.
- Do not mix picker rendering with filesystem scanning.
- Avoid fuzzy matching and ranking heuristics beyond README-first and alphabetical order.
- Keep picker UI quieter than Glow: no large logo, no cloud/stash concepts.

## Done Criteria

- No-argument behavior matches the product decision.
- Directory arguments work.
- File picker works and opens documents.
- Tests cover discovery, ranking, filtering, and default decisions.
- No Git-root or remote discovery exists.
