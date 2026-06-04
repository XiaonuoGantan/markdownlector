.PHONY: ci

# The single source of truth for the verification gate. CI runs exactly this,
# so the build is reproducible locally byte-for-byte. Each line is its own
# shell; make aborts on the first non-zero exit.
ci:
	cargo fmt --check
	cargo test
	cargo clippy -- -D warnings
