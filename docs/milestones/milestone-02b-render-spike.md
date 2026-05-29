# Milestone 02b: Render Spike

## Goal

A deliberately tiny, throwaway vertical slice that proves the whole pipeline survives contact with a real terminal **before** the full reader is built on top of it:

```text
bytes -> pulldown-cmark events -> IR -> layout lines -> crossterm -> alternate screen
```

This is insurance, not a feature. Milestones 01 and 02 validate the IR and layout only against snapshot tests of an abstraction we invented. This spike confirms that abstraction renders correctly on an actual alternate screen — so if the `LayoutLine` / styled-span shape is wrong, we find out with ~200 lines of throwaway code instead of discovering it underneath the reader state machine, search, and resize in Milestone 03a.

## User-Visible Behavior

```text
markdownlector --spike README.md
```

(A temporary entry path; this flag does not survive into v1.)

- Opens the alternate screen.
- Renders the first screenful of a laid-out document, styled.
- `q` quits and restores the terminal cleanly.
- No scrolling, no search, no keymap beyond `q`, no resize handling.

## Scope

Implement:

- The first real `crossterm` usage (enter/leave alternate screen, raw mode, write, flush).
- A minimal render of `&[LayoutLine]` to the screen using the Milestone 02 layout output.
- Clean teardown on `q` and on error.

Do not implement:

- Scrolling, viewport math, or a reader state machine (that is Milestone 03a).
- Any navigation beyond `q`.
- Search, headings, help, status line, theme detection.
- Anything that makes this slice grow. If it starts accreting features, stop and start Milestone 03a.

## Implementation Plan

1. Add `crossterm` and justify it in `DEPENDENCIES.md`.
2. Render a fixed first-screen slice of layout lines onto the alternate screen.
3. Wait for `q`; restore the terminal.
4. Confirm the styled output matches what the layout and style roles intended. Eyeball it against `DESIGN.md` restraint — quiet, spacing-led, no neon.

## Outcome

This spike answers one question: **is the IR + layout-line + styled-span contract the right shape to render from?**

- If yes: the spike code may be deleted, or it may graduate into the skeleton of `terminal.rs` / `render.rs` in Milestone 03a — but it carries no features forward beyond "open, draw a slice, restore."
- If no: fix the IR / layout shape back in Milestone 02 now, while it is cheap and nothing depends on it.

Record the finding (one or two sentences) in the Milestone 03a notes or the commit message so the decision is traceable.

## Verification

```text
make ci
cargo run -- --spike README.md
```

Expected result: the alternate screen shows a styled first screenful; `q` returns you to a normal, working shell with no leftover raw mode.

## Done Criteria

- A real document renders to the alternate screen via the Milestone 02 pipeline.
- `q` restores the terminal cleanly.
- The IR / layout contract is confirmed, or amended back in Milestone 02.
- No feature creep: the slice does exactly open -> draw -> restore.
