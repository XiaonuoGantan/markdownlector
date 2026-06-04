# markdownlector

A reading appliance for Markdown in the terminal.

> **Status:** Milestone 01 — plain-text output only. The full-screen reader
> arrives in a later milestone.

`markdownlector` reads Markdown from a file or standard input and prints it as
clean plain text — the foundation the terminal reader will build on.

## Usage

```
markdownlector <file>      render a file
markdownlector -           render Markdown from stdin
markdownlector --help
markdownlector --version
```

## Build and test

```
cargo build
make ci        # cargo fmt --check, cargo test, cargo clippy -- -D warnings
```

`make ci` is the single source of truth for the verification gate; the GitHub
Actions workflow runs exactly that command, so the build is reproducible
locally byte-for-byte.

## Dependencies

This project keeps its dependency graph small enough to audit by hand. Every
runtime dependency is justified in [`DEPENDENCIES.md`](DEPENDENCIES.md).

## License

[AGPL-3.0-only](LICENSE).
