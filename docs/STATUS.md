# Markdownlector — Status

A living progress overlay on the roadmap in [`milestones/README.md`](milestones/README.md): what is built, what is next, and the invariants that hold across the work. Update it as milestones land.

**Current state:** Milestone 01 complete and in review ([PR #3](https://github.com/XiaonuoGantan/markdownlector/pull/3) → `main`). Six milestones remain. The first user-facing build lands at Milestone 03a.

_Last updated: 2026-06-09._

Legend: ✅ done · 🔜 next · ⬜ planned.

## The destination

A quiet, opinionated **terminal reading appliance** for Markdown — a full-screen, vim-navigable reader with a file picker, search, and an adaptive theme — small enough to audit by hand. Unix-only, AGPL-3.0. Everything hangs off one pipeline:

```text
bytes → pulldown-cmark events → compact IR → layout lines → viewport render
```

## Build status on the core pipeline

```text
bytes → events → plain text              ✅ M01   shipped (dead-end diagnostic branch)
              ↘  IR → layout lines        🔜 M02   next — pure, no terminal
                        → crossterm slice  ⬜ M02b  throwaway spike to de-risk the contract
                        → viewport render  ⬜ M03a  the reader — first thing a human can touch
```

Milestone 01 deliberately built only the **plain-text dead-end** (`bytes → events → text`) — enough to prove the input/output boundary without a TUI. The real render path (IR → layout → viewport) begins at Milestone 02. The plain branch is a sibling of that path, never a foundation for it.

## Milestone ledger

| # | Milestone | Delivers | Status |
|---|-----------|----------|--------|
| **01** | Foundation, Plain Output, CI | CLI, input-boundary safety, plain non-TTY output, `make ci` gate | ✅ **Done** (PR #3 open) |
| **02** | Markdown IR & Layout Engine | `bytes→IR→layout lines`, stable anchors, wide-char width — **pure, no terminal** | 🔜 **Next** |
| **02b** | Render Spike | ~200-line throwaway crossterm slice to confirm the IR/layout shape renders | ⬜ Planned |
| **03a** | Reader Core | panic-safe terminal lifecycle, visible-only render, scroll, resize, minimal vim, status line | ⬜ Planned — **first touchable** |
| **03b** | Reader Nav, Search, Theme | full vim keymap, search on stable anchors, heading jumps, help overlay, adaptive theme | ⬜ Planned |
| **04** | File Picker & Default Open | directory discovery, no-arg defaults, picker ⇄ reader loop | ⬜ Planned |
| **05** | Polish, Performance, Release | visual polish, Ghostty profile, <50 ms cold-open confirmation, release docs, dep audit | ⬜ Planned |

## ✅ Done — Milestone 01 (in review)

- **CLI**: `markdownlector <file> | - | --help | --version`; manual parser (no clap); Unix compile guard; pipe-safe `emit()` (a closed pipe such as `| head` is a clean stop, not a panic).
- **Input boundary**: NUL byte → binary error (checked before UTF-8), invalid UTF-8 → error, empty input valid, leading UTF-8 BOM stripped. Never panics on bad input.
- **Plain output**: conservative `events → text` — heading/inline markers stripped, list markers + 2-space nesting kept, link URLs dropped, exactly one trailing newline. Deterministic.
- **Infra**: `make ci` (`cargo fmt --check`, `cargo test`, `cargo clippy -- -D warnings`) as the shell-of-record + a thin GitHub workflow that runs exactly that; `DEPENDENCIES.md`; **41 tests** (28 unit + 13 integration).
- **Open thread**: PR #3 not yet merged. Merging it also closes the one review nit that is hygiene rather than code — `.gitattributes` only governs committed bytes.

## 🔜 Next — Milestone 02 (Markdown IR & Layout)

The largest conceptual step, and **still pure** — no terminal, fully snapshot-testable:

- **Compact document IR** — blocks (heading / paragraph / code / blockquote / list / table / rule) + inline spans. Lives behind `markdown.rs`; pulldown-cmark types never leak past the adapter.
- **Stable anchors** (block index + character offset) — search matches and heading targets reference these, never layout-line numbers, so they survive relayout at a new width. This bet is locked here because everything downstream depends on it.
- **`text_width.rs`** — display-column width (CJK = 2 columns, combining marks = 0). All wrapping and table alignment route through it. Proposes the **`unicode-width`** dependency (decision to be recorded in `DEPENDENCIES.md`).
- **Layout engine** — IR + width → `Vec<LayoutLine>`, each line carrying its styled spans, its plain searchable text, and the IR anchor of its first span. Wrap to columns, editorial spacing, conservative table fallback.
- **Heading index** for later jumps.
- New files: `layout.rs`, `text_width.rs`, `document.rs`; layout fixtures + `layout_snapshots.rs`.
- ⚠️ **Tripwire** (recorded in the M02 doc): the moment `plain.rs` would need a column width or text wrapping, it has crossed into layout — freeze it and route that work through this engine instead.

## Cross-cutting invariants (the constitution)

These hold across every milestone:

- **The CI gate is real from M01** — reproducible locally byte-for-byte, not locked to one forge. ✅ live.
- **"No full repaint per keypress"** is an architectural invariant *asserted at M03a*, not a release-time benchmark. Only the wall-clock `<50 ms cold-open` number waits for M05.
- **Terminal teardown is panic-safe** (RAII + panic hook + signal-safe), landing with the first raw-mode code in M03a. A panic must never wreck the user's terminal.
- **Positions anchor to stable IR offsets** (defined in M02), so search and heading targets survive resize.
- **Rendering targets `impl Write`** (M03a) — the seam that keeps the reader unit-testable.
- **Editorial restraint** (`DESIGN.md`) governs every visual choice: spacing over color, bold only for `Strong`, color reserved for `Error`/`Success` in the status line.
- **The dependency policy stays tight.** v1 runtime deps: `pulldown-cmark` (✅ M01), `crossterm` (M02b / M03a), `unicode-width` (proposed M02). Nothing else without written justification.

## Honest read on sequencing

The first three milestones (01 ✅, 02, 02b) are **all foundation or internal** — nothing a user can *touch* until **M03a** (open a file, scroll, resize, quit). That is intentional: you cannot render lines you have not laid out, and the file picker is a consumer of the reader. The M02b spike exists precisely to catch a wrong IR / layout shape with throwaway code *before* the reader, search, and resize are built on top of it — so a contract mistake costs ~200 lines, not a rewrite.
