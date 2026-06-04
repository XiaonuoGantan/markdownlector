# Dependencies

Every runtime dependency is justified here. A crate earns a row by being
load-bearing; it loses its row the moment its "Removal trigger" fires. Keeping
this list short and auditable by hand is a project goal, not an afterthought.

Dev- and build-only dependencies are listed separately and held to a lighter
standard.

## Runtime

| Crate | Version | Role | Replaces | Removal trigger |
|-------|---------|------|----------|-----------------|
| `pulldown-cmark` | 0.13 | CommonMark pull-parser: `bytes → events` | a hand-written Markdown tokenizer | we vendor our own parser, need source spans it does not expose, or it pulls in a transitive dependency we are unwilling to audit |

`pulldown-cmark` is confined to `src/markdown.rs`. Every other module consumes
its event stream and never names the crate, so replacing the parser is a
one-file change.

With `default-features = false` (the `html` renderer is unused), `pulldown-cmark`
pulls three small, widely-used transitive crates: `bitflags`, `memchr`, and
`unicase`. The entire runtime graph is four crates — auditable in one
`cargo tree`.

## Dev / build

_(none)_

CLI integration tests use `std::process::Command` with the cargo-provided
`CARGO_BIN_EXE_markdownlector` path instead of an assertion crate such as
`assert_cmd`, so the test suite adds no dependencies to audit.
