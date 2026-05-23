# Markdownlector High-Level Design

## Assumptions

- The app is primarily a terminal reading experience, not a general Markdown conversion tool.
- It should run well in Ghostty, but should not depend on Ghostty-only behavior for core use.
- The app cannot reliably set the terminal font. Legibility must come from layout, contrast, spacing, wrapping, and a documented recommended terminal profile.
- The first implementation should favor local files and stdin. Remote fetching, cloud features, editing, and rich customization are out of scope.
- The app targets Unix-like terminals only. Windows support is explicitly out of scope.
- The existing repository license is AGPL-3.0. Keep that unless the project owner explicitly changes it.

## Glow Study

Glow is strong because it combines three related jobs in one binary:

- Render Markdown from a file, stdin, HTTP URL, GitHub, or GitLab source.
- Launch a TUI browser when run without arguments or pointed at a directory.
- Provide a full-screen pager for reading rendered Markdown.

Its implementation leans on the Charm ecosystem: Bubble Tea for the TUI loop, Bubbles for widgets, Glamour for Markdown rendering, Lip Gloss for styling, plus Cobra/Viper/env packages for CLI/config, fsnotify for reloads, clipboard support, fuzzy matching, reflow helpers, and terminal utilities.

Relative to this project's goals, the important lessons are:

- The two-state TUI model is good: file list -> document pager.
- The viewport/pager abstraction is good: only the visible region needs to repaint during navigation.
- Glow's visual identity is intentionally playful and colorful. Markdownlector should be quieter, sharper, and more reading-focused.
- Glow exposes a broad customization surface: config file, styles, JSON style sheets, width, pager choice, mouse, line numbers, preserve-newlines, and more. Markdownlector should avoid becoming a theme/config platform.
- Glow renders through a general-purpose renderer and then pages rendered text. Markdownlector should own its Markdown-to-terminal layout so it can optimize reading, resize behavior, and viewport rendering directly.

## Product Stance

Markdownlector should be a "reading appliance" for Markdown in the terminal:

- Fast enough that opening a README feels instant.
- Opinionated enough that users do not tune themes, keymaps, or renderer internals.
- Attractive without being decorative.
- Vim-native in navigation rather than "less with some vim keys".
- Small enough that maintainers can audit the dependency graph and rendering pipeline.

The default experience:

```text
markdownlector README.md   # opens the full-screen reader
markdownlector .           # opens the file picker
markdownlector             # opens current-directory README.md when obvious; otherwise picker
markdownlector -           # reads Markdown from stdin
```

When stdout is not a TTY, it should emit plain text only. It should not force a TUI into pipelines or emit ANSI styling into text-only output.

No-argument behavior:

- If there is one obvious `README.md` in the current directory, open it directly.
- If multiple Markdown files are present in the current directory, show the simple file picker.
- If exactly one Markdown file exists and no README is present, open that file directly.
- If no Markdown file exists, show an empty file picker with a concise status message.

## UX Model

### Reader View

The reader view is the primary product.

- Full-screen alternate-screen interface by default.
- No large logo or decorative header.
- Bottom status line only: path, current heading, scroll percent, search state.
- Body width capped for legibility, centered or left-guttered depending on terminal width.
- Headings get spacing, weight through color/intensity, and clear hierarchy.
- Paragraphs use readable wrapping and blank-line rhythm.
- Code blocks use subtle framing and preserve indentation. Syntax highlighting is not part of v1.
- Blockquotes use a left rule and muted text.
- Tables render as aligned plain terminal tables, with graceful fallback for narrow screens.
- Links are visible but not noisy. Link target preview can appear in the status line on cursor/focus later, not in v1 unless easy.

### File Picker

The file picker should be utilitarian and quick.

- Markdown discovery is non-recursive and scoped to the current directory or supplied directory.
- README files rank first, then alphabetical filename/path order.
- `/` filters files.
- `Enter` opens.
- `q` exits.
- No cloud stash, no document editing, no clipboard actions, no file watcher in v1.

### Navigation

Default keymap should be vim-based and fixed:

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

File picker:

```text
j / k       move selection
Ctrl-d/u    half-page down / up
gg / G      first / last file
/           filter
Enter / l   open file
h           back to file picker from reader
q           quit
Esc         clear filter
```

No keymap customization in v1.

## Visual Direction

The default style should be "editorial terminal", not neon dashboard:

- Adaptive dark/light theme selected from terminal background where possible.
- Truecolor palette with 256-color fallback.
- Warm near-white or near-black base, restrained accent color, muted secondary text.
- Avoid saturated fuchsia/green as the dominant brand signal.
- Preserve terminal-native sharpness. Do not simulate cards heavily.
- Use spacing and hierarchy more than color.

Because a terminal app cannot control the font, ship a documented Ghostty profile recommendation instead of runtime font controls. The app itself should render well in common monospace fonts.

## Architecture

```text
src/
  main.rs          CLI entry and mode selection
  app.rs           top-level state machine
  terminal.rs      raw mode, input, screen writes
  input.rs         file/stdin/directory source loading
  discover.rs      Markdown file discovery
  markdown.rs      parser adapter and semantic IR
  layout.rs        block layout, wrapping, line cache
  render.rs        ANSI styling and viewport drawing
  keymap.rs        fixed vim key handling
  search.rs        document search state
  theme.rs         built-in adaptive palette
```

Core pipeline:

```text
bytes -> pulldown-cmark events -> compact document IR -> layout lines -> viewport render
```

The important architectural choice is to keep Markdown semantics until layout time. Do not convert Markdown into one giant ANSI string and then treat it as opaque text. A compact IR allows:

- Efficient resize relayout.
- Heading navigation.
- Search with stable match positions.
- Better narrow-screen behavior.
- Rendering only the visible viewport on scroll.

## Rust Dependency Policy

Recommended v1 dependencies:

- `pulldown-cmark` for Markdown parsing.
- `crossterm` for portable raw-mode input and terminal control.

Avoid in v1:

- `ratatui`, unless direct terminal drawing becomes too costly.
- `syntect` or other large syntax-highlighting stacks.
- `clap`, unless manual argument parsing becomes meaningfully painful.
- Config-file libraries.
- Fuzzy-search libraries. Simple substring filter is enough.
- HTTP clients.
- Clipboard, editor integration, and file-watch dependencies.

Every dependency should have an entry in a small `DEPENDENCIES.md` explaining why it exists, what it replaces, and what would trigger removal.

## Performance Strategy

- Read source once.
- Parse once per file load.
- Layout once per terminal resize or width-affecting state change.
- Render only visible lines on scroll.
- Avoid per-key Markdown re-rendering.
- Keep line cache as styled spans, not concatenated global ANSI strings.
- Avoid syntax highlighting in v1 to prevent large startup and dependency cost.
- Use `cargo bench` later for render/layout benchmarks, but start with simple fixture timing tests.

Initial success criteria:

- Open a typical README under 50 ms on a modern laptop after cold process start.
- Smooth navigation in a 5,000-line Markdown file with no full-document repaint per keypress.
- Memory usage remains proportional to source size plus layout cache, with no duplicate full rendered ANSI copy.
- Dependency graph remains small enough to audit manually.

## V1 Scope

Must have:

- File, directory, and stdin input.
- Full-screen reader.
- File picker.
- Fixed vim navigation.
- Search.
- Heading jump.
- Built-in adaptive style.
- Plain fallback when not attached to a TTY.

Should have:

- Good table fallback.
- Terminal resize support.
- Snapshot tests for renderer output.
- Fixture-based performance checks.

Not v1:

- Custom themes.
- Custom keymaps.
- Persistent config file.
- Remote URL fetching.
- GitHub/GitLab README fetching.
- File watching.
- Editing integration.
- Clipboard integration.
- Mouse support.
- Syntax highlighting.

## Open Questions

Closed:

- No args opens an obvious `README.md` in the current directory; if multiple Markdown files exist, show the simple file picker.
- Unix-like terminals only. Windows support is not planned.
- `?` opens TUI help.
- Keep AGPL-3.0.
- Non-TTY output is plain text only.
- File discovery is non-recursive and scoped to the current directory or supplied directory.

Still open:

- None at this level.
