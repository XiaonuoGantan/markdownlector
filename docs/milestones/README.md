# Markdownlector Milestones

This roadmap takes Markdownlector from an empty repository to a v1 reading appliance, in dependency order. It implements the v1 scope in `HIGH_LEVEL_DESIGN.md` under the editorial restraint of `DESIGN.md`.

## Sequence

| # | Milestone | One-line goal |
|---|---|---|
| 01 | [Foundation, Plain Output, and CI](milestone-01-foundation-and-plain-output.md) | Smallest Rust CLI: file/stdin input, input-boundary safety, plain non-TTY output, and a CI gate that runs fmt/test/clippy. |
| 02 | [Markdown IR and Layout Engine](milestone-02-markdown-ir-and-layout.md) | `bytes → IR → layout lines`, with IR-stable anchors and correct wide-char width — pure, no terminal. |
| 02b | [Render Spike](milestone-02b-render-spike.md) | Throwaway end-to-end crossterm slice to confirm the IR/layout contract renders, before the reader is built on it. |
| 03a | [Reader Core](milestone-03a-reader-core.md) | Panic-safe terminal lifecycle, visible-only render, scrolling, resize, minimal vim motions, status line. |
| 03b | [Reader Navigation, Search, and Theme](milestone-03b-reader-navigation-and-search.md) | Full vim keymap, search with stable positions, heading jumps, help overlay, adaptive theme. |
| 04 | [File Picker and Default Open](milestone-04-file-picker-and-default-open.md) | Directory discovery, no-arg defaults, the picker, and the picker ⇄ reader loop. |
| 05 | [Polish, Performance, and Release Readiness](milestone-05-polish-performance-and-release-readiness.md) | Visual polish, Ghostty profile doc, wall-clock perf confirmation, release docs, dependency audit. |

## Why this order

You cannot render lines you have not laid out, and the file picker is a consumer of the reader — so input/parse/layout come first, the reader next, the picker after, and release polish last. Milestone 01's plain non-TTY path is the cheapest possible walking skeleton for the input/output boundary; it is foundation, not a user-facing feature. The first thing a human can *touch* lands at Milestone 03a.

## Cross-cutting decisions (the constitution)

These hold across every milestone:

- **The CI gate is real from M01.** `make ci` runs `cargo fmt --check`, `cargo test`, `cargo clippy -- -D warnings`; a thin workflow runs it on push/PR. The gate is reproducible locally byte-for-byte and not locked to one forge.
- **"No full repaint per keypress" is an architectural invariant, asserted at M03a** — not a release-time benchmark. Only the noisy wall-clock "<50 ms cold open" number lives in M05.
- **Terminal teardown is RAII + panic hook + signal-safe (M03a).** A panic must never leave the user's terminal wrecked.
- **Search and heading positions anchor to stable IR offsets (defined in M02), not layout lines** — so they survive resize.
- **Rendering targets `impl Write` (M03a)** — the seam that makes the reader unit-testable.
- **Editorial restraint (`DESIGN.md`) governs every visual choice.** Quiet print, not neon dashboard: spacing over color, bold only for `Strong`, color reserved for `Error`/`Success` in the status line.
- **The dependency policy stays tight.** v1 runtime deps are `pulldown-cmark` and `crossterm`; `unicode-width` is proposed in M02 for correct wide-char width (decision recorded there and in `DEPENDENCIES.md`). Nothing else without justification.

## Provenance

The original five-milestone roadmap (M01, M02, a single large M03, M04, M05) was pressure-tested in a roundtable across architecture, product, engineering, and test-architecture perspectives. The changes captured here: the overloaded M03 was split into **03a (core)** and **03b (navigation & search)**; a **02b render spike** was inserted to de-risk the IR/layout contract early; **CI** moved into M01; and the **perf invariant**, **input-boundary**, **IR-anchor**, and **wide-char** tests were pulled forward to where the relevant code is born — leaving M05 to *confirm* rather than *discover*.
