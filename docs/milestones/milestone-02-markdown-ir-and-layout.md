# Milestone 02: Markdown IR and Layout Engine

## Goal

Build the core reading pipeline:

```text
bytes -> pulldown-cmark events -> compact document IR -> layout lines
```

This milestone creates the semantic foundation needed for efficient rendering, heading navigation, search, resize behavior, and future viewport drawing.

## User-Visible Behavior

There may still be no full-screen UI. The key outcome is testable internal behavior:

- Markdown is parsed into a compact document representation.
- A document can be laid out to a requested terminal width.
- Layout output is deterministic and snapshot-testable.

## Scope

Implement:

- Markdown parser adapter built on the existing `pulldown-cmark` dependency.
- Compact document IR.
- Layout engine for common Markdown blocks.
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

3. Implement parser adapter.
   - Convert parser events into project-owned IR.
   - Avoid exposing `pulldown-cmark` event types outside the adapter.
   - Start with readable, explicit handling for common blocks.

4. Implement layout.
   - Input: document IR, width, theme-neutral style roles.
   - Output: `Vec<LayoutLine>`.
   - Each line contains spans plus plain searchable text.
   - Wrap paragraphs to width.
   - Preserve code block indentation.
   - Add spacing around headings and major blocks.
   - Degrade tables to aligned plain columns when width permits; otherwise stack rows conservatively.

5. Add heading index generation.
   - Store heading title, level, block index, and first layout line after layout.
   - This supports `]]` and `[[` later.

6. Connect plain output to the IR where practical.
   - Reuse Markdown parsing instead of maintaining a separate ad hoc plain renderer if the code stays simpler.
   - If reuse complicates Milestone 01 behavior, keep the plain path separate and note the reason.

## Tests

Unit tests:

- Parser builds heading, paragraph, list, code, quote, link, and table blocks.
- Layout wraps paragraphs at given widths.
- Layout preserves fenced code content and indentation.
- Heading index line positions remain stable after layout.
- Table layout has deterministic fallback for narrow width.

Fixture tests:

- Compare layout line plain text against committed expected output.
- Test at narrow, normal, and wide widths.

## Verification

Run:

```text
cargo fmt --check
cargo test
cargo clippy -- -D warnings
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
- Make width handling explicit and tested.
- Keep style roles semantic, such as `Heading1`, `Muted`, `Code`, instead of hard-coded color names.

## Done Criteria

- Markdown source parses into project-owned IR.
- Layout works without terminal access.
- Heading index exists.
- Common Markdown fixtures are covered.
- No full-document ANSI string is required for layout or search.
