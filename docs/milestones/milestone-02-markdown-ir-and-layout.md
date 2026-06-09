# Milestone 02: Markdown IR and Layout Engine

## Goal

Build the core reading pipeline:

```text
bytes -> pulldown-cmark events -> compact document IR -> layout lines
```

This milestone creates the semantic foundation needed for efficient rendering, heading navigation, search, resize behavior, and future viewport drawing. It also locks down two architectural bets early, where they are cheap to test: that positions (search matches, heading targets) anchor to stable IR offsets rather than layout lines, and that the layout cache stays proportional to source size rather than duplicating a full rendered copy.

> **Architectural tripwire (from the M01 review).** M01's `src/plain.rs` is a dead-end `events -> text` pass: it preserves structural markers (heading text, list bullets, nesting indentation) but does no width-aware reflow. The moment a change would make `plain.rs` aware of a column width or wrap text, it has crossed from marker preservation into layout — stop and route that work through this milestone's layout engine and IR rather than growing `plain.rs`. The two paths share only the `markdown::parse` event stream; the layout pipeline is a second consumer of it, never a consumer of `plain`.

## User-Visible Behavior

There may still be no full-screen UI. The key outcome is testable internal behavior:

- Markdown is parsed into a compact document representation.
- A document can be laid out to a requested terminal width.
- Layout output is deterministic and snapshot-testable.
- A position anchored before relayout still points at the same content after relayout at a different width.

## Scope

Implement:

- Markdown parser adapter built on the existing `pulldown-cmark` dependency.
- Compact document IR.
- A stable anchor model expressed in IR terms (block index plus character offset).
- Layout engine for common Markdown blocks.
- Correct terminal column-width measurement for wide (CJK) and combining characters in `text_width.rs`.
- Styled span model that can later render to ANSI or plain text.
- Heading index.
- Basic table fallback.
- Snapshot-style expected-output tests using committed fixtures.

Do not implement:

- Terminal raw mode.
- Interactive scrolling.
- File picker.
- Syntax highlighting.
- Custom themes.

## Suggested File Layout

```text
src/
  markdown.rs
  layout.rs
  text_width.rs
  document.rs
tests/
  fixtures/
    layout_basic.md
    layout_code.md
    layout_table.md
    layout_headings.md
    layout_wide_chars.md
  layout_snapshots.rs
```

## Implementation Plan

1. Extend the parser adapter.
   - Keep all parser-specific types behind `markdown.rs`.
   - Reuse the `pulldown-cmark` dependency introduced in Milestone 01.

2. Define the document IR.
   - Blocks: heading, paragraph, code block, blockquote, list, table, horizontal rule.
   - Inline spans: text, emphasis, strong, code, link text.
   - Keep source text or normalized text where needed for search.
   - Keep heading level and title for navigation.
   - Define a stable anchor type in IR terms (for example, block index plus character offset). Search matches and heading targets reference anchors, never layout-line numbers, so they survive relayout at a new width.

3. Implement parser adapter.
   - Convert parser events into project-owned IR.
   - Avoid exposing `pulldown-cmark` event types outside the adapter.
   - Start with readable, explicit handling for common blocks.

4. Implement text width measurement.
   - `text_width.rs` reports display columns, not byte or `char` counts.
   - Wide characters (CJK) count as two columns; zero-width and combining marks count as zero.
   - All wrapping and table alignment go through this function. (See the dependency decision below.)

5. Implement layout.
   - Input: document IR, width, theme-neutral style roles.
   - Output: `Vec<LayoutLine>`.
   - Each line carries its spans, its plain searchable text, and the stable IR anchor of its first span.
   - Wrap paragraphs to width using display columns.
   - Preserve code block indentation.
   - Add spacing around headings and major blocks. Follow the editorial-rhythm spirit in `DESIGN.md`: generous blank-line spacing before major headings (the terminal analogue of the 96px section rhythm), tighter spacing between related blocks. Exact line counts live in code; the principle is restraint, not density.
   - Degrade tables to aligned plain columns when width permits; otherwise stack rows conservatively.

6. Add heading index generation.
   - Store heading title, level, IR anchor, and first layout line after layout.
   - This supports `]]` and `[[` later, and stays correct across relayout because it stores the anchor, not just a line number.

7. Connect plain output to the IR where practical.
   - Reuse Markdown parsing instead of maintaining a separate ad hoc plain renderer if the code stays simpler.
   - If reuse complicates Milestone 01 behavior, keep the plain path separate and note the reason.

## Dependency Decision: `unicode-width`

Correct column-width handling for CJK and combining marks cannot be done reliably by hand; it needs the Unicode East Asian Width tables. This milestone proposes adding [`unicode-width`](https://crates.io/crates/unicode-width) as the third runtime dependency, after `pulldown-cmark` and `crossterm`. It is tiny, has no transitive dependencies, and is the de-facto standard for exactly this problem.

Rationale: misaligned wrapping and broken table columns directly undermine the "editorial, legible" stance in `DESIGN.md`. Width errors are visible rendering defects, not cosmetic details.

Alternative, if the owner wants to hold the strict two-dependency line: explicitly punt wide-character correctness for v1 and document in the README that CJK/emoji-heavy documents may misalign. That is a legitimate scope cut — but it must be a recorded decision, not a lurking bug.

Either way, record the choice in `DEPENDENCIES.md`.

## Tests

Unit tests:

- Parser builds heading, paragraph, list, code, quote, link, and table blocks.
- Layout wraps paragraphs at given widths.
- Layout preserves fenced code content and indentation.
- Heading index line positions remain stable after layout.
- Table layout has deterministic fallback for narrow width.
- `text_width` reports correct display columns for ASCII, CJK (double-width), and combining-mark inputs.
- Anchor stability: an anchor resolved at one width still maps to the same source content after relayout at a narrow and a wide width. (This is the test that keeps search positions from drifting on resize in the reader.)
- Plain-text round-trip: a substring match in a `LayoutLine`'s plain text resolves to an anchor that points back at the same source span.
- Layout cache for a known fixture stays within a sane multiple of source size (guards the "no duplicate full ANSI copy" bet).

Fixture tests:

- Compare layout line plain text against committed expected output.
- Test at narrow, normal, and wide widths.
- Include a wide-character fixture (`layout_wide_chars.md`).

## Verification

Run:

```text
make ci
```

Manual inspection:

```text
cargo test layout_snapshots -- --nocapture
```

Expected result: deterministic layout output with readable wrapping and no global rendered ANSI string.

## Readability and Maintainability Rules

- Own the project IR. Do not let parser library details leak across modules.
- Prefer clear block structs over clever generic node trees.
- Keep layout pure: no terminal IO, no file IO, no global state.
- Make width handling explicit and tested; never measure width with `.len()` or `.chars().count()`.
- Keep style roles semantic, such as `Heading1`, `Muted`, `Code`, instead of hard-coded color names.

## Done Criteria

- Markdown source parses into project-owned IR.
- Layout works without terminal access.
- Positions and heading targets anchor to stable IR offsets and survive relayout.
- Wide-character width is correct, or explicitly punted and documented per the dependency decision.
- Heading index exists.
- Common Markdown fixtures are covered.
- No full-document ANSI string is required for layout or search.
